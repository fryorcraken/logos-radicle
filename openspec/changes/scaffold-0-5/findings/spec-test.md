# Spec-test review: scaffold-0-5

Scope. This change declares `skip_specs: true` and moves no requirement, so
sections 1 (scenario coverage), 4 (requirements moved between capabilities) and
6 (PLAN.md shedding) have nothing to check: there is no delta spec, and
`openspec/specs/` holds nothing about the sync button, the file tree or the
README (`git grep` for `syncedOnce`, `Download All`, `Re-sync`, `lastSyncedCommit`
and `README` in `openspec/specs` finds only unrelated "staleness guard" lines).
The tests the change touches are `radicle-ui/tests/ui/source.yaml` and
`sync.yaml`, both changed in `bc24ded`. Both are in `ui-tests.yml`'s matrix
(`spec: [browse, branches, source, sync, write, local]`), so they do run.

Mutations: **none run, none left in the tree.** A sitometres spec needs the
Nix-built Basecamp 0.3.0 `#app` and a live seed, which is not a cheap layer, and
the three QML/Rust suites do not execute these files. Every "cannot fail" or
"can fail" below is a judgement from reading, not a measurement.

- [ ] **`tester`** — `radicle-ui/tests/ui/sync.yaml:189-209` — the cancel half
      still depends on the first repository being large enough
      **Scenario:** the reorder in `bc24ded` removed one dependence on the
      seed's first repository (the poll no longer queues behind a backlog) and
      kept another. Between `clicking it starts a real sync` and
      `click the button again to cancel` there are three more steps. If the seed
      lists a small repository first (one with few entries, which
      `scaffold-0-5`'s own commit title says the spec must now tolerate), the
      sync completes in that window. `finishSyncIfDone()` then sets
      `syncedOnce = true`, the button reads `Re-sync`, and the "cancel" click
      starts a second sync instead. `the sync stops` may pass by the second one
      finishing, and `a cancelled sync does not claim to have completed` then
      fails on `syncedOnce === false`. That is a red run on a healthy build,
      and it is the same shape of defect `bc24ded` set out to remove. The header
      argues only the upper bound (a sync can take over five minutes), never
      the lower one.
      **Fix:** say in the header that the spec needs a repository whose sync
      outlasts those steps, or make the dependence checkable, for example by
      asserting `root.syncing === true` immediately before the cancel click so a
      too-small repository fails there and names itself.
      **Severity:** medium.

- [ ] **`tester`** — `radicle-ui/tests/ui/source.yaml:142-161` — the new scroll
      step throws when the first repository has no root `README.md`, while the
      comment above it claims the spec holds "for any repository"
      **Scenario:** the header of the README section (lines 48-58) says a
      repository without a README "is not a failure". The scroll step does
      `indexOf("README.md")` and throws `no README.md in …` on `i < 0`, and the
      click step after it selects `text: "README.md"`. A first repository with
      no README, or one named `README` or `README.adoc`, fails the spec at
      step 13. Before `bc24ded` it failed there too (the click found nothing),
      so this is not a regression, but the commit and the comment now say the
      dependence is gone and it is not.
      **Fix:** either state the dependence ("needs a root `README.md`") in the
      comment and in `docs/e2e.md:56-63`, or pick the file to open from
      `entryNames` (the first non-directory entry) and drop the dependence.
      **Severity:** medium.

- [x] **`dev-writer`** — `docs/e2e.md:44-46` — now false: it says "no spec here
      has yet relied on a `calls:` assertion"
      **Scenario:** `sync.yaml:123-125` (`expect: calls:
      ["radicle.remoteListBranches"]`, added in `bc24ded`) is exactly such an
      assertion. CLAUDE.md's rule is to prune a claim the same change
      falsifies, and task 4.3 edited this very paragraph.
      **Severity:** low, but it is a stale claim a later reader will trust.
      **Fixed** in the commit that ticks this box: the sentence now names `sync.yaml`'s staleness
      step as the first spec to assert `calls:`, and says its power to fail on its own has not been
      shown. When the tester answers the `calls:` box below, that sentence is the one to update with
      the result.

- [ ] **`tester`** — `radicle-ui/tests/ui/sync.yaml:123-125` — the first
      `calls:` assertion in the repo has not been shown able to fail
      **Scenario:** `docs/e2e.md:39-46` records that `calls:` reported "no
      backend calls at all" on the old stack while data plainly arrived, and
      calls 0.2.0's output "observed, not a proven gate". The step also holds
      `wait_for: state: updateAvailable === true`, which cannot become true
      without the call, so the `calls:` line adds no assertion the state line
      lacks unless it can fail on its own. Nothing records that it was watched
      failing (for example with a name no call carries). Task 5.2's "sync 19/19"
      shows it passes, not that it discriminates.
      **Fix:** run it once with a wrong method name, record the failure in
      `tasks.md`, and fix the `docs/e2e.md` sentence above to say the result.
      **Severity:** low.

- [ ] **`tester`** — `radicle-ui/tests/ui/sync.yaml:208-209` — a newly pinned
      behaviour with no scenario and no `NO SPEC:` marker
      **Scenario:** the added assertion `sourceTabItem.lastSyncedCommit ===
      '000…0'` after start-then-cancel pins that neither `syncAll()` nor
      `cancelSync()` touches `lastSyncedCommit`. The input is the spec's own
      stand-in value, so it is independent of the implementation and can fail,
      which is good. But no requirement says a sync start must preserve it, and
      an implementation that cleared it on `syncAll()` would break this spec for
      a defensible reason. The comment justifies it from the code ("recorded
      only there too"), not from a requirement. Grep for `NO SPEC` across
      `radicle-ui/tests/ui` finds nothing, so nobody has marked it.
      **Fix:** mark it `NO SPEC:` so the spec-writer can decide it, as the repo's
      convention asks. The same holds for "the Update state never gates the
      click" and "a cancelled sync does not claim completion", which the comments
      describe as "the explicit requirement" with no spec behind them.
      **Severity:** low.

- [ ] **`tester`** — `radicle-ui/tests/ui/sync.yaml:150-158` — the in-progress
      label is asserted only by exclusion
      **Scenario:** `syncLabel !== 'Download All' / 'Re-sync' / 'Update'` also
      holds for an empty or `undefined` label, so a button whose Text lost its
      percentage binding and rendered nothing would pass. The comment says
      "the label becomes a percentage".
      **Fix:** add a positive check, for example `/%$/.test(root.syncLabel)`.
      **Severity:** low.

Clean, in prose. The stand-in design in `sync.yaml` is sound: the spec writes
`lastSyncedCommit` itself and the head comes from the real seed, so
`updateAvailable === true` could not hold with `checkForUpdate()` deleted, and
dropping `syncedOnce` before the cancel is what lets the cancel assertions tell
"cancel claimed completion" from the stand-in. `root.syncLabel` is read off the
rendered Text item, and `root.sourceTabItem`, `entryNames` and `entryCount` all
exist in the QML (checked for existence only), so none of the new `eval` or
`state` expressions is comparing against a missing property. The completion
transition the spec hands off is pinned in `tst_repo_switch.qml`
(`test_a_completed_sync_flips_syncedOnce`) and the staleness comparison in
`tst_staleness.qml`, so the hand-off holds. `design.md`'s Risks entry and
`docs/PLAN.md`'s "Known defects" entry for the cancel backlog are consistent
with each other and with the spec comments.
