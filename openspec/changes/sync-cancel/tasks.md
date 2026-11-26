## Stages

- [x] spec — `spec-writer`
- [x] design + code — `dev-writer`
- [x] tests — `tester`
- [ ] review: correctness — `code-reviewer`
- [ ] review: security — `code-reviewer`
- [ ] review: readability — `code-reviewer`
- [x] review: architecture — `code-reviewer`
- [ ] review: spec-test — `spec-test-reviewer`
- [x] review: design — `design-reviewer`
- [ ] findings all ticked, `findings/` deleted — `closer`
- [ ] `openspec validate --strict`, then `archive` — `closer`
- [ ] CI green, title/body checked, PR merged — `closer`

# Tasks

## 1. Bounded sync dispatch in `SourceTab.qml`

- [x] 1.1 Write `radicle-ui/tests/tst_sync_cancel.qml` against a fake node
  that serves one request at a time, and watch it fail on the unchanged
  code: 7 of its 13 test rows failed there, among them 70 reads reaching the
  node after a cancel against a bound of 4.
- [x] 1.2 Queue a sync's discovered reads (`syncPending`) and dispatch them
  from `pumpSync()` only while fewer than `syncMaxInFlight` (4) are
  outstanding; `cancelSync()`, `syncAll()` and `reset()` drop the queue.
  Verified by `tst_sync_cancel.qml`'s bound, overtaking and late-reply tests.
- [x] 1.3 Mark a file in flight at dispatch rather than at discovery, and let
  a sync's file reply paint the file the user is waiting on. Verified by the
  two "still opens" tests in `tst_sync_cancel.qml`, both red before.
- [x] 1.4 The existing sync tests still pass unchanged:
  `tst_sync_epoch.qml`, `tst_repo_switch.qml`, `tst_staleness.qml`, run one
  file at a time; and `sh radicle-ui/tests/run-qml-tests.sh` gives one
  `Totals:` line per `tst_*.qml` and no `FAIL`.
- [x] 1.5 The bound against the local node ("The bound holds against the
  local node") is **satisfied by construction**, not tested: the bound is
  enforced before `Main.call()` derives the `remote`/`local` prefix, so the
  dispatch code is the same for every mode. No component test sees the
  prefix, and no e2e spec syncs in Local mode.

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
