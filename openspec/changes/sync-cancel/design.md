# Design

## Context

See `proposal.md` for why. The constraint that decides the approach is how a
call reaches the seed:

- A view calls `app.call()` (`Main.qml`), which hands the call to the UI
  backend over QtRO and returns. The reply arrives later through
  `logos.watch`.
- The backend (`radicle_ui_backend.cpp`) forwards each method to the core
  module with a synchronous call, one per slot invocation, and the core
  answers it with a blocking HTTP request.
- So the module serves calls one at a time, in the order they were sent. The
  issue's measurement is the same fact from outside: a `ListBranches` sent
  after a cancel came back 116 s later, behind the cancelled sync's reads.

A sync is the seed's: the `repo-sync` spec's "The sync control is the seed's"
shows the control only while the derived prefix is `remote`, so every read a
sync makes is a `remote*` call.

Before this change `syncAll()` handed every read to `app.call()` the moment it
was discovered: a root listing with 60 entries became 60 calls sent at once.
`cancelSync()` only bumped `syncEpoch`, which makes the replies ignored when
they arrive. Every one of them was still served.

## Goals / Non-Goals

**Goals:**

- A cancel stops the sync's reads reaching the seed, within the spec's bound
  of 4, whatever the size of the repository.
- The core module does not learn what a sync is.

**Non-Goals:**

- Making a sync faster. The bound is chosen not to make it slower; nothing
  here is meant to change its throughput.
- Abandoning a sync by leaving the repository or switching branch without a
  cancel (out of scope in `proposal.md`). `reset()` empties the dispatch
  window as a side effect of sharing the cancel's bookkeeping, but nothing
  here specifies or tests that path.

## Decisions

### 1. The cancel is enforced where the view dispatches, not in the core

**Chosen:** `SourceTab` holds a sync's discovered reads in its own FIFO
(`syncPending`) and hands them to `app.call()` only while fewer than
`syncMaxOutstanding` are outstanding (`syncOutstanding`). Each answered read
frees a slot and dispatches the next. After a cancel nothing pumps the queue
again, because only a reply under the running epoch does (Decision 5), so
what is still held is never sent. QML only; the module's interface is
unchanged.

*Guard:* raising `syncMaxOutstanding` to 1000 turns six tests in
`tst_sync_cancel.qml` red: the large-sync, never-more-than-4-outstanding,
bound-does-not-grow and overtaking tests, and both rows of the late-reply
slot test. Removing `resetSyncDispatch()` from `cancelSync()` turns nothing
red, and is not meant to: the epoch bump is what stops the queue being
pumped, `syncAll()` empties the window again before the next sync, and the
call in `cancelSync()` only lets go of what the queue held.

The issue put the question this way: "Orphaning replies in QML cannot shorten
the queue; the requests have to stop being served. That puts the cancel
either in how the view dispatches (bounded batches, so a cancel stops the next
batch: QML only) or in the core module (dropping queued calls: widens its
interface)."

**Ruled out — an epoch-only fix**, which is what shipped: bump the epoch, drop
the replies. It cannot shorten the queue, because by the time of the cancel
every read is already in the module's queue and the epoch lives in the view.
Any variant of it (a second epoch, a guard on more terms) has the same
defect: it decides what to do with a reply, and the cost is in serving the
request.

**Ruled out — dropping queued calls in the core.** A `cancelSync` method, or
an epoch argument on every `GetTree`/`GetBlob` with a "drop below epoch N"
call, has two problems:

- *The cancel would arrive behind the backlog.* The calls travel the same
  QtRO channel to a backend that serves them one at a time and in order, so a
  cancel call is read only after every call sent before it has been served,
  which is exactly the wait it is meant to remove. Making it overtake would
  mean serving calls concurrently in the backend or adding a second channel,
  either a larger change than the defect.
- *It widens the interface*, which `radicle_impl.h` keeps at "string in,
  string out, per method" so the core never learns about a view's workflows.
  A sync is a view's idea; the core would gain state about it in two places
  (the backend and the module) for one caller.

**Ruled out — fixed batches** (send N reads, wait for all N, send the next N),
the issue's own phrasing of the QML option. It bounds the cancel equally, but
a batch is as slow as its slowest read, and the slots of the reads that
answer early stand idle. A sliding window keeps the same bound and keeps the
module busy.

### 2. Why the spec's bound is 4

The `repo-sync` spec states the number: at most 4 of a sync's requests
outstanding while it runs, and at most 4 reaching the seed after a cancel.
`syncMaxOutstanding` is that 4, and the tests write it as a literal
(`tst_sync_cancel.qml`'s `bound`), so changing it is a change to the spec and
the tests together. This records why 4 was the number put to the spec.

- **Not much more.** The bound is also the worst case a request after a
  cancel waits for. On the measurement in the issue (116 s for a backlog of a
  33-entry repository, about 3.5 s a read on that seed) 4 reads is about 14 s,
  inside `sync.yaml`'s 30 s wait with the poll's own request on top; 8 would
  not be.
- **Not 1.** The module serves one call at a time, so a window larger than 1
  buys no parallelism; what it buys is that the module never sits idle while
  a reply travels back and the next call travels out, two QtRO hops (view to
  backend, backend to core). Against a seed that round trip is small beside
  the HTTP request. **This is reasoning, not a measurement**: neither the
  round trip nor a sync's throughput at 1 against 4 has been measured, so the
  case for 4 over 1 is that it costs a later request at most three more reads
  of waiting and cannot be slower. An earlier version of this argument rested
  on local reads, where the round trip would be most of each read; a sync no
  longer runs against the local node, so it does not apply.

No measurement has been made of sync throughput at 4 against the unbounded
dispatch either. The argument says it should not be slower; that is
reasoning, not a number.

### 3. A read is marked in flight when it is sent, not when it is queued

`inFlight[key]` is the map `openEntry()` and `prefetch()` consult to avoid a
duplicate request: a file already being fetched is left to the request that
is fetching it. Before, the sync set it at discovery. With a queue that a
cancel empties, a mark set at discovery would outlive the dropped read, and
opening that file afterwards would wait for ever on a reply that is never
coming. So `fetchSyncBlob()` sets it at dispatch, and re-checks the cache and
the map there too, since a click may have fetched the file while it waited its
turn.

*Guard:* setting the mark in `queueSyncBlob()` instead (and re-checking only
the cache at dispatch) turns five rows of `tst_sync_cancel.qml` red: both
"still opens after the cancel" tests, the "files outstanding" row of the
late-reply slot test, and both rows of
`test_a_sync_after_a_cancel_downloads_everything`, where the next sync skips
every file the cancelled one had queued (21 of 51 reads asked).

### 4. A sync's file read paints the file if the user is waiting on it

`openEntry()` leaves a file already in flight to the request fetching it.
`prefetch()`'s reply paints the viewer for that reason; the sync's reply did
not, so opening a file whose sync read was outstanding left the pane on
"Loading…". The spec requires the file to show ("A file opened during the sync
still opens"), so `fetchSyncBlob()`'s reply paints when
`cacheKey(selectedFile)` is the key it fetched, whether or not its sync was
cancelled. It compares keys rather than rid and path, as `openEntry()` and
`prefetch()` do, because the sync's read is pinned to the branch the sync
started on (Decision 7) and the key is what carries the branch. The three
readers share `blobBody()` for decoding the reply; their paint guards differ
on purpose and are not unified, for the reasons issue #29 gives against one
universal staleness guard.

*Guard:* removing that paint turns
`test_a_file_the_sync_is_already_fetching_still_opens` red. This was a defect
before the queue existed; the queue made it rarer (only the 4 outstanding
reads can hit it), not absent.

### 5. Only a reply under the running epoch frees a slot

`syncOutstanding` is reset to 0 when a sync starts, is cancelled, or the tab
is reset (`resetSyncDispatch()`), and decremented only by `settleSyncRead()`,
which returns at once for a run whose epoch is no longer current. `pumpSync()`
checks the same. A late reply from a cancelled sync therefore cannot free a
slot in the sync that followed it, which would let the new sync put a fifth
read out, and cannot pump the cancelled sync's queue. The check is written
once in each of those two functions rather than at each of the four reply
callbacks, so a new reply path inherits it.

*Guard:* removing the check from `settleSyncRead()` turns the "files
outstanding" row of `test_a_late_reply_does_not_free_a_slot_in_the_next_sync`
red (8 of the new sync's reads with the seed at once). The "listings
outstanding" row stays green, because the `GetTree` success callback keeps an
epoch check of its own ahead of walking the listing; that check is what stops
a stale listing reaching `settleSyncRead()` at all.

**A refused read settles exactly as an answered one.** It frees its slot and
counts as done, so the sync carries on and can finish marked as synced past a
read the seed refused. The alternatives were to stop the sync at the first
refusal, or to retry. Both are a change to what a sync promises, which the
spec does not state, and this change did not set out to make one; it keeps
what the unbounded dispatch already did. What the queue adds is that a
refusal *must* free its slot: one that did not would wedge the window after
four refusals. *Guard:* removing `settleSyncRead()` from `fetchSyncBlob()`'s
failure callback turns `tst_sync_walk.qml`'s
`test_a_refused_read_does_not_stall_the_sync` red (the read after the
refusals is never made).

One consequence worth stating: right after a cancel and a restart, the
module can hold up to 4 reads of the cancelled sync, 4 of the new one, and the
new sync's `ListBranches` (Decision 6). The spec bounds each sync's own reads,
not their sum, and a request made then waits behind at most 9 calls, not the
repository.

### 6. `pumpSync()` is the only place a sync's tree and file reads dispatch

Every `GetTree` and `GetBlob` a sync makes is sent from `fetchSyncTree()` or
`fetchSyncBlob()`, and those are called only from `pumpSync()`. That makes
"does anything bypass the bound?" a question with an answer: the two
`app.call` sites for a sync's reads, both counted in `syncOutstanding`. A
cached listing is walked without taking a slot, since it sends nothing.

The one exception is the sync's head lookup: `syncAll()` sends one
`ListBranches` directly, outside `pumpSync()` and uncounted. It is not one of
the spec's tree and file reads, it is sent before any of them, so it cannot sit
behind a cancelled sync's reads, and it is the same single request
`checkForUpdate()` makes.

### 7. Everything a sync's reads depend on is fixed when it starts

`syncAll()` builds one value, `run`, and passes it to every step of the walk
in place of the bare epoch: the epoch, the rid and branch every read is sent
for and cached under, and `listed`, the set of directories this sync has
queued. Two defects the queue introduced are closed by it.

- **A seed that lists a directory inside itself froze the view.** Once the
  directory was cached, `pumpSync()` took its job, walked the cached listing
  without taking a slot, which queued the directory again, and looped without
  yielding to the event loop, so the Cancel click was never delivered. Two
  directories listing each other did the same. The unbounded dispatch had the
  same defect in recursive form and ended in "Maximum call stack size
  exceeded", which was recoverable; the loop made it unbounded. The seed is
  untrusted input, so `queueSyncTree()` queues each directory once per sync.
- **Reads were sent under the tab's live rid and branch.** With every read
  sent at discovery that hardly mattered; with a queue, a branch assigned
  without a reset sent the rest of a running sync to the new branch and
  cached it under the new branch's keys, while the sync recorded the old
  branch's head. Production cannot reach this (`RepoView` resets the tab on
  every repository or branch change, which moves the epoch), so the pinning
  is defence in depth, and the shape the repo's staleness-guard hazard warns
  about: the fetchers took the epoch as an argument and read the rest of what
  the request depends on from ambient state.

`listed` is `Object.create(null)`, so a directory a seed names `__proto__` or
`constructor` is an ordinary key rather than an inherited property that reads
as already queued.

*Guard:* removing the `listed` check turns both cycle tests in
`tst_sync_walk.qml` red. Each listing's entry is a getter that counts its
reads and throws after 1000, so the regression fails the test rather than
hanging the runner. Building the keys and arguments from the tab's live
`rid` and `branch` instead turns
`test_every_read_of_a_sync_goes_to_the_branch_it_started_on` red (16 of the
20 file reads sent to `dev`).

**Ruled out — a tab property for the set, emptied in `resetSyncDispatch()`.**
It would be one more thing to reset at each of the three places the epoch
moves; a value built per sync is empty for each sync by construction, and a
stale run's set can never be consulted for the running one.
**Ruled out — capping how many cached listings one pump may walk.** It turns a
freeze into an arbitrary number and still re-lists the cycle. **Not done —
de-duplicating files the same way.** A listing that names one file many times
queues it many times, but each copy after the first is skipped at dispatch,
counted done without a request, so the work is bounded by what the seed sent.

## Risks / Trade-offs

- [A sync of a large repository could be slower than before] → Decision 2's
  window keeps the module fed; no measurement yet. If one shows a cost, the
  number is the spec's to change, and the tests' literal with it.
- [Another view that sends many calls at once reintroduces the backlog for
  every later call] → Said at `Main.call()`, where a view's calls start, and
  in `docs/e2e.md`, where the sync spec's wait depends on it.
- [`sync.yaml`'s 30 s poll wait depends on the seed's speed per read] → It
  waits behind at most 4 reads, so it fails only if a read takes over about
  6 s; before the fix it waited behind the whole repository.
