# Correctness review: sync-cancel

Dimension: correctness only (security, readability and architecture are other instances).

Reviewed `radicle-ui/src/qml/SourceTab.qml` against `origin/main`, the `repo-sync` delta, `design.md`, and the `sync.yaml` / `docs/e2e.md` changes. The Qt 6 runner (`/usr/lib64/qt6/bin/qmltestrunner`) was available and each file was run alone and read to the end: `tst_sync_cancel.qml` (18 passed), `tst_sync_epoch.qml` (5), `tst_sync_cancel_view.qml` (3), `tst_staleness.qml` (13), `tst_repo_switch.qml` (11). `check-qml-syntax.sh` passes.

Attempts to break it, and what they showed:

- **Slot accounting.** Every bump of `syncEpoch` (`reset`, `syncAll`, `cancelSync`) is paired with `dropSyncQueue()`, and every `syncInFlight++` sits in a dispatch that has exactly one settling reply under that epoch, so `syncInFlight` cannot go negative and a late reply from a cancelled sync cannot free a slot in the next one. Every queued job adds exactly one to `syncDone` (request reply, cached listing, or blob skip), so `syncDone >= syncQueued` cannot fire early or never.
- **The bound.** `pumpSync` is the only dispatch site and checks `syncInFlight < syncMaxInFlight` on each iteration; the cached-listing and blob-skip paths send nothing and take no slot. The bound holds for any listing size.
- **Re-entrancy.** A synchronous failure callback (`Main.call`'s "unsupported operation" path) re-enters `pumpSync` from inside the loop, but the queue is only populated by asynchronous successful listings, so recursion depth stays bounded. Not a finding.
- **Mode switch, repository switch and branch switch mid-sync.** All reach `reset()` through `nav.reset()` / `RepoView.onRidChanged` / `onBranchChanged`, so queued reads are dropped before they can dispatch under the new prefix, rid or branch.
- **Mutations, all restored.** `syncMaxInFlight = 1000` turns exactly the five tests `design.md` Decision 1 names red (measured: 5 failed). Removing `finishSyncIfDone()` from the cached-listing path is caught by `tst_staleness.qml` (`test_resyncing_clears_updateAvailable_and_catches_up`) and by none of `tst_sync_cancel.qml`. `origin/main`'s `SourceTab.qml` against the new test file fails 8 rows (see the finding below).
- **`sync.yaml`.** `root.navBusy` and `root.navError` exist in `Main.qml`; the `/^[0-9]+%$/` selector survives YAML quoting. The e2e job was not run (no Basecamp here).

Not defects, noted only: a blob skipped at dispatch because another read has it in flight is counted done before it is cached, so a failed read leaves a "synced" repository with that file uncached. `origin/main` behaves the same way (it skipped at discovery on the same `inFlight` mark), so this change neither introduces nor widens it.

- [ ] **`dev-writer`** — `openspec/changes/sync-cancel/tasks.md:22` — task 1.1 records a measurement the file no longer produces
      **Scenario:** 1.1 says "7 of its 13 test rows failed there". Running `tst_sync_cancel.qml` as it now stands against `origin/main`'s `SourceTab.qml` gives 8 failed and 10 passed, and the file holds 16 test rows (18 reported, less `initTestCase` and `cleanupTestCase`), not 13: later commits added rows (`c139697`, `3032449`, `b08cda1`) and the number was not re-measured. The 70-reads-after-a-cancel figure in the same sentence does reproduce (`70 of the sync's reads reached the node after the cancel; the bound is 4`). A stale count in a document that outlives the change is what the repo's "keeping documents true" rule exists to prevent; re-measure it or state the failing rows by name instead of a ratio.
      **Severity:** low, documentation accuracy; no behavioural defect found.
      **Measured:** 8 failed / 10 passed on `origin/main`'s `SourceTab.qml`; 0 failed / 18 passed on this branch.
