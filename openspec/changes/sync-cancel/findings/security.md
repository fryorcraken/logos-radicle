# Security review: sync-cancel

Dimension: security only (the other dimensions are other instances').
Scope: `radicle-ui/src/qml/SourceTab.qml` (the only source change); the
untrusted input is the seed's `GetTree`/`GetBlob` replies, per the overlay.
Method: read the diff against `origin/main`, then ran probe components with the
Qt 6 runner (`/usr/lib64/qt6/bin/qmltestrunner`, one file, read to the end),
once on this tree and once with `SourceTab.qml` restored from `origin/main`.
The probes live in `./tmp/` and are not committed; `SourceTab.qml` is restored
to this branch's version and the tree is clean.

- [x] **`dev-writer`** — `SourceTab.qml:467-473` and `:494-502` — a seed that
      lists a directory containing itself (or any cycle through already-cached
      directories) now freezes the view in an unbounded loop; before this change
      it ended in a recoverable exception.
      **Scenario:** the seed answers every `GetTree` with
      `{entries:[{kind:"tree", path:"a", name:"a"}]}`. `GetTree("")` is answered
      and cached, `walkSyncListing` queues `a`, `pumpSync` dispatches it; its
      reply is cached and lists `a` again, `walkSyncListing` queues `a`, and
      `pumpSync` pops it, finds `treeCache["a"]` set, takes the "no slot" branch
      of `fetchSyncTree`, walks the cached list, which queues `a` again, and
      `syncInFlight` stays below the bound with `syncPending` never empty, so
      the `while` in `pumpSync` never exits. The cached branch never yields to
      the event loop, so the Cancel button's click is never delivered: the only
      recovery is killing the host process. A sync started against a hostile or
      buggy seed is enough; no user action beyond pressing Sync.
      **Measured:** with a queue-backed fake serving that reply, this tree: the
      runner did not return in 30 s (`timeout` exit 124, still inside
      `syncAll`/reply handling, 50 serve steps never completed). The same probe
      against `origin/main`'s `SourceTab.qml`: returns at once with
      "Uncaught exception: Maximum call stack size exceeded", the UI stays
      alive and Cancel works. So the cycle was already a defect, but it was a
      bounded one; moving the walk from recursion to a loop made it unbounded,
      and the design's own claim that the cached walk "sends nothing" is why
      nothing stops it. Severity: medium (denial of service of the module's UI
      by an untrusted seed; the overlay names the seed as untrusted input).
      Likely remedy for the dev-writer to choose: a per-sync set of directory
      keys already queued, checked in `syncDir`, which also bounds `syncQueued`.
      The existing `tst_sync_cancel*.qml` fakes never return a cyclic or
      repeated directory, so no test of the change can see this.
      **Fixed** in the commit "Sync: list each directory once per sync, and
      keep every read on its branch", with the remedy suggested: each sync
      carries a `listed` set (`Object.create(null)`, so a directory named
      `__proto__` is an ordinary key), checked in `queueSyncTree()`. It lives
      on the per-sync `run` value rather than a tab property, so it is empty
      for each sync by construction (design Decision 7). Regression tests:
      `tst_sync_walk.qml`'s `test_a_directory_that_lists_itself_is_listed_once`
      and `test_two_directories_listing_each_other_are_each_listed_once`. The
      cyclic entry's `kind` is a getter that counts its reads and throws past
      1000, so the defect fails the test instead of hanging the runner.
      Watched failing before the fix: both "Uncaught exception: the sync
      walked … 1001 times"; after it, each directory is requested once and the
      sync completes.

- [x] **`dev-writer`** — `SourceTab.qml:495-505`, `:521-535` — queued reads are
      sent under the live `rid`/`branch` at dispatch time, not the ones the sync
      began under, so the window in which a direct property change is mixed into
      a running sync grew from "reads discovered after the change" to "every read
      not yet dispatched".
      **Scenario:** `syncAll()` on `main`; its head lookup answers `main-head`;
      the root listing (20 files) is answered; then `source.branch = "dev"` is
      assigned without `reset()` (the case `tst_staleness.qml`'s
      `test_a_head_lookup_answering_after_a_branch_switch_is_dropped` and the
      comment at `syncAll()`'s `wantBranch` say must be survivable); the rest is
      served. Four reads go out as `GetBlob main`, the other 16 as `GetBlob dev`,
      the sync finishes with `syncedOnce = true` and
      `lastSyncedCommit = "main-head"`, and `updateAvailable` stays false: the
      repository is reported synced to a commit whose files were never fetched
      under that branch.
      **Measured:** probe on this tree printed exactly that sequence
      (`GetBlob:main:f0..f3 | GetBlob:dev:f4..f19 ... syncedOnce=true
      last=main-head`). On `origin/main` every blob of that listing was sent at
      discovery, so all 20 would have gone to `main`; the mixed result needs the
      queue. Production is covered, since `RepoView.onRidChanged` and
      `onBranchChanged` call `reset()`, which bumps the epoch, so this is
      defence in depth, not a reachable defect today. Severity: low. It matters
      because `fetchSyncTree`/`fetchSyncBlob` take an `epoch` but read ambient
      state for the rest of what the request depends on, which is the shape
      `CLAUDE.md` ("pass what the function needs") and `## Hazards` (the
      staleness guard dropping a term) warn about. Remedy: capture `rid` and
      `branch` in `syncAll()` and pass them (or pin them on the queued job).
      **Fixed** in the same commit as the cycle: `syncAll()` builds one
      `run` value (epoch, rid, branch, listed set) and every step takes it in
      place of the bare epoch; reads are sent with, and cached under,
      `cacheKeyIn(run.rid, run.branch, …)`. Regression test:
      `tst_sync_walk.qml`'s
      `test_every_read_of_a_sync_goes_to_the_branch_it_started_on`, your
      probe's sequence. Watched failing before the fix with exactly your
      measurement (`GetBlob:dev:f4` … `f19`, 16 reads to `dev`); after it, 0.

## Clean areas

- **Slot accounting and epoch guard:** every path that decrements
  `syncInFlight` is behind `epoch !== syncEpoch`; a late reply from a cancelled
  or reset sync cannot free a slot in the next sync or pump a cancelled one.
  `dropSyncQueue()` is called at all three places the epoch moves. Nothing
  bypasses `pumpSync()` in the sync path (the two `app.call` sites are the
  fetchers it calls).
- **Cache poisoning across repositories:** a late reply after `reset()` writes
  into the fresh `blobCache`/`treeCache`/`inFlight` objects, but under a key that
  carries the rid and branch it was dispatched with, and the viewer paint is
  guarded by `cacheKey(tab.selectedFile) === bkey`, which also carries the
  current rid and branch. A stale reply cannot paint another repository's pane.
  (An entry whose path is the empty string would match `selectedFile === ""`;
  the content is that seed's own and `prefetch()` already behaves the same, so
  not a finding.)
- **Malformed replies** (a `null` entry, `data` null): the callback throws
  before `settleSyncRead`, so the slot and the count leak and the sync never
  finishes. That is the same stall as before the change (one lost count), and
  Cancel clears it because `dropSyncQueue()` resets the slots. Not a regression.
- **Resource use:** `syncPending` grows only with replies the seed actually
  sends, and is dropped on cancel; the cancel now reduces, rather than
  preserves, the work a hostile seed can force.
- **`inFlight` marking at dispatch:** a dropped read leaves no mark behind, so
  nothing waits for ever on a reply that is never coming.
- **No new dependency, no FFI or core change, no change to the interface, no
  new path, filesystem or credential handling, no error text that reaches the
  user.** `sync.yaml` and `docs/e2e.md` are test and prose only.
