# Design

## Context

See `proposal.md` for why. The constraint that decides the approach is how a
call reaches the node:

- A view calls `app.call()` (`Main.qml`), which hands the call to the UI
  backend over QtRO and returns. The reply arrives later through
  `logos.watch`.
- The backend (`radicle_ui_backend.cpp`) forwards each method to the core
  module with a synchronous call, one per slot invocation, and the core
  answers it with a blocking HTTP request or FFI read.
- So the module serves calls one at a time, in the order they were sent. The
  issue's measurement is the same fact from outside: a `ListBranches` sent
  after a cancel came back 116 s later, behind the cancelled sync's reads.

Before this change `syncAll()` handed every read to `app.call()` the moment it
was discovered: a root listing with 60 entries became 60 calls sent at once.
`cancelSync()` only bumped `syncEpoch`, which makes the replies ignored when
they arrive. Every one of them was still served.

## Goals / Non-Goals

**Goals:**

- A cancel stops the sync's reads reaching the node, within a bound that does
  not depend on the repository.
- The same mechanism for every mode, without the core module learning what a
  sync is.

**Non-Goals:**

- Making a sync faster. The bound is chosen not to make it slower; nothing
  here is meant to change its throughput.
- Abandoning a sync by leaving the repository or switching branch without a
  cancel (out of scope in `proposal.md`). `reset()` empties the queue as a
  side effect of sharing the cancel's bookkeeping, but nothing here specifies
  or tests that path.

## Decisions

### 1. The cancel is enforced where the view dispatches, not in the core

**Chosen:** `SourceTab` holds a sync's discovered reads in its own FIFO
(`syncPending`) and hands them to `app.call()` only while fewer than
`syncMaxInFlight` are outstanding (`syncInFlight`). Each answered read frees a
slot and dispatches the next. `cancelSync()` empties the queue, so what is
still held is never sent. QML only; the module's interface is unchanged.

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

### 2. The bound is 4

**`syncMaxInFlight` is 4**, for every mode. It is the spec's fixed bound: after
a cancel at most 4 of the sync's tree and file reads reach the node, and a
request made after a cancel waits behind at most 4 of them. Tests write it as
a literal (`tst_sync_cancel.qml`'s `bound`), so changing it is a change to the
tests too.

Why 4 and not another number:

- **Not 1.** The module serves one call at a time, so a window larger than 1
  buys no parallelism; what it buys is that the module never sits idle while
  a reply travels back and the next call travels out. That round trip is two
  QtRO hops (view to backend, backend to core). Against a seed it is small
  beside the HTTP request; against the Radicle home, where a read is a local
  FFI call, it is the larger part of each read, and a window of 1 would make a
  local sync wait on it every time.
- **Not much more.** The bound is also the worst case a request after a
  cancel waits for. On the measurement in the issue (116 s for a backlog of a
  33-entry repository, about 3.5 s a read on that seed) 4 reads is about 14 s,
  inside `sync.yaml`'s 30 s wait with the poll's own request on top; 8 would
  not be.

No measurement has been made of sync throughput at 4 against the unbounded
dispatch. The argument above says it should not be slower; that is reasoning,
not a number.

### 3. A read is marked in flight when it is sent, not when it is queued

`inFlight[key]` is the map `openEntry()` and `prefetch()` consult to avoid a
duplicate request: a file already being fetched is left to the request that
is fetching it. Before, `syncBlob()` set it at discovery. With a queue that a
cancel empties, a mark set at discovery would outlive the dropped read, and
opening that file afterwards would wait for ever on a reply that is never
coming. So `fetchSyncBlob()` sets it at dispatch, and re-checks the cache and
the map there too, since a click may have fetched the file while it waited its
turn.

*Guard:* setting the mark in `syncBlob()` instead turns
`tst_sync_cancel.qml`'s `test_a_file_queued_by_the_sync_still_opens_after_the_cancel`
red.

### 4. A sync's file read paints the file if the user is waiting on it

`openEntry()` leaves a file already in flight to the request fetching it.
`prefetch()`'s reply paints the viewer for that reason; the sync's reply did
not, so opening a file whose sync read was outstanding left the pane on
"Loading…". The spec requires the file to show ("A file opened during the sync
still opens"), so `fetchSyncBlob()`'s reply paints when
`cacheKey(selectedFile)` is the key it fetched, whether or not its sync was
cancelled. Comparing keys rather than paths carries the rid and branch terms
with it.

*Guard:* removing that paint turns
`test_a_file_the_sync_is_already_fetching_still_opens` red. This was a defect
before the queue existed; the queue made it rarer (only the 4 outstanding
reads can hit it), not absent.

### 5. Only a reply under the running epoch frees a slot

`syncInFlight` is reset to 0 when a sync starts, is cancelled, or the tab is
reset, and decremented only by a reply whose epoch is still current. A late
reply from a cancelled sync therefore cannot free a slot in the sync that
followed it, which would let the new sync put a fifth read out.

*Guard:* decrementing before the epoch check turns
`test_a_late_reply_does_not_free_a_slot_in_the_next_sync` red.

One consequence worth stating: right after a cancel and a restart, the
module can hold up to 4 reads of the cancelled sync and 4 of the new one. The
spec bounds only the cancelled sync's, and a request made then waits behind at
most 8 reads, not the repository.

### 6. `pumpSync()` is the only place a sync dispatches

Every `GetTree` and `GetBlob` a sync makes is sent from `fetchSyncTree()` or
`fetchSyncBlob()`, and those are called only from `pumpSync()`. That makes
"does anything bypass the bound?" a question with an answer: the two
`app.call` sites in the sync path, both counted in `syncInFlight`. A cached
listing is walked without taking a slot, since it sends nothing.

## Risks / Trade-offs

- [A sync of a large repository could be slower than before] → Decision 2's
  window keeps the module fed; no measurement yet. If one shows a cost, the
  number is the one place to change, and the tests' literal with it.
- [The local-node scenario has no test that drives the `local*` path] → The
  bound is enforced before `Main.call()` chooses the method prefix, so the
  dispatch code is the same for every mode; the component tests cannot see
  the prefix, and `local.yaml` does not sync. Satisfied by construction, and
  said so in `tasks.md`.
- [Another view that sends many calls at once reintroduces the backlog for
  every later call] → Recorded in `docs/e2e.md`, where the sync spec's wait
  depends on it.
- [`sync.yaml`'s 30 s poll wait depends on the seed's speed per read] → It
  waits behind at most 4 reads, so it fails only if a read takes over about
  6 s; before the fix it waited behind the whole repository.
