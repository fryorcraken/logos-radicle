## Stages

- [x] spec — `spec-writer`
- [x] design + code — `dev-writer`
- [x] tests — `tester`
- [x] review: correctness — `code-reviewer`
- [x] review: security — `code-reviewer`
- [x] review: readability — `code-reviewer`
- [x] review: architecture — `code-reviewer`
- [x] review: spec-test — `spec-test-reviewer`
- [x] review: design — `design-reviewer`
- [x] findings all ticked, `findings/` deleted — `closer`
- [x] `openspec validate --strict`, then `archive` — `closer`
- [ ] CI green, title/body checked, PR merged — `closer`

# Tasks

## 1. Bounded sync dispatch in `SourceTab.qml`

- [x] 1.1 Write `radicle-ui/tests/tst_sync_cancel.qml` against a fake node
  that serves one request at a time, and watch it fail on the unchanged
  code. Run against `origin/main`'s `SourceTab.qml`, the rows that fail are
  the overtaking test (70 of the sync's reads served ahead of the branch
  listing), the large-sync and bound-does-not-grow tests, the
  never-more-than-4-outstanding test, both rows of the late-reply slot test,
  and the three "still opens" tests; the rest pass there, as they should,
  since they pin what the old code already did.
- [x] 1.2 Queue a sync's discovered reads (`syncPending`) and dispatch them
  from `pumpSync()` only while fewer than `syncMaxOutstanding` (4) are
  outstanding; `cancelSync()`, `syncAll()` and `reset()` empty the window
  (`resetSyncDispatch()`). Verified by `tst_sync_cancel.qml`'s bound,
  never-more-than-4-outstanding, overtaking and late-reply tests.
- [x] 1.3 Mark a file in flight at dispatch rather than at discovery, and let
  a sync's file reply paint the file the user is waiting on. Verified by the
  two "still opens" tests in `tst_sync_cancel.qml`, both red before.
- [x] 1.4 The existing sync tests still pass unchanged:
  `tst_sync_epoch.qml`, `tst_repo_switch.qml`, `tst_staleness.qml`, run one
  file at a time; and `sh radicle-ui/tests/run-qml-tests.sh` gives one
  `Totals:` line per `tst_*.qml` and no `FAIL`.
- [x] 1.5 "The sync control is the seed's": `RepoView` shows the control only
  while `app.source` is `remote`, which this change did not alter. Verified by
  `tst_sync_cancel_view.qml`'s `test_the_sync_control_is_shown_for_the_seed_only`,
  one row per prefix, whose `local` row goes red when the control is shown
  for every source.
- [x] 1.6 Fix the two walk defects review found (Decision 7): queue each
  directory once per sync, and send and cache every read under the rid and
  branch the sync started on. `tst_sync_walk.qml`'s two cycle tests and its
  branch test, all three red before the fix.
- [x] 1.7 Pin that a refused read frees its slot and the sync completes
  (Decision 5): `tst_sync_walk.qml`'s refused-read test, red with the
  failure callback's `settleSyncRead()` removed.

## 2. The e2e regression test and the docs

- [x] 2.1 Put `radicle-ui/tests/ui/sync.yaml` back in the user's order
  (start, cancel, then poll), with the poll and the busy strip each given
  30 s against a backlog measured at 116 s, and remove its two cancel-related
  `NO SPEC` markers, which the `repo-sync` spec now answers. Verified by the
  `sitometres sync` CI job, not run locally (no Basecamp built here).
- [x] 2.2 Replace `docs/e2e.md`'s paragraph saying a cancel does not cancel
  its requests with what now holds and the hazard it leaves.
- [x] 2.3 `sh radicle-ui/tests/check-qml-syntax.sh` and
  `sh radicle-ui/tests/lint-qml.sh` pass; `openspec validate sync-cancel
  --strict` passes.
