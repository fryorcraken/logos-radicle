# spec-test review: sync-cancel

Reviewed: `specs/repo-sync/spec.md` against `tst_sync_cancel.qml`,
`tst_sync_cancel_view.qml` and `ui/sync.yaml`, and against issue #92 read
fresh with `gh issue view 92`. `openspec validate sync-cancel --strict` passes.

**Coverage by scenario.** Covered at the component layer: large sync stops
short, bound does not grow, branch listing overtakes the backlog, sync stops at
once, cancelled first sync offers `Download All` (component and view), last
synced commit survives, file opened during the sync (before and after the
cancel), sync after a cancel downloads everything (drained and not drained),
late reply ignored. Covered only by `sync.yaml`, which I could not run (no
Basecamp here): cancel-then-poll, busy indicator clears, cancel raises no
error. **No test at all: the local-node bound scenario.**

**Local-node scenario: a spec defect, not a test gap.** Verdict on the tester's
report: it holds. `RepoView.qml:347` shows the sync control only when
`app.source === "remote"`, and the spec defines a cancel as "activating the
sync control while it runs". So no user can start, let alone cancel, a sync in
Local mode. The scenario's GIVEN cannot be reached through the control. The
prefix is derived in `Main.call()`, above the component layer's `app` fake, so
a component test cannot see it either, and `local.yaml` does not sync.
`design.md`'s "satisfied by construction" is an argument, not a test.

**Areas that read clean:** `tst_sync_cancel_view.qml` can fail (a second click
that did not cancel leaves `syncing` true); every fake reply is
input-dependent; the bound tests count requests served after the cancel rather
than asking the implementation; the "sync after a cancel downloads everything"
count is independent of the implementation (51 unique reads derived from
`makeRepo`). The requirement-text length limit is met (strict validation
passes). No requirement is stale against issue #92, whose three "done when"
points are all specified. No requirements moved between capabilities
(`repo-sync` is new, nothing `REMOVED`).

**Mutations run (both restored; none left in the tree; `git status` clean).**
Command for both:
`/usr/lib64/qt6/bin/qmltestrunner -platform offscreen -input radicle-ui/tests/tst_sync_cancel.qml -import radicle-ui/src/qml`
(`-platform offscreen` stands in for the `QT_QPA_PLATFORM` the overlay says a
direct call needs).

1. `cancelSync()` also sets `lastSyncedCommit = pendingSyncHead`. **Killed:**
   `test_the_last_synced_commit_survives_a_cancel` failed (`head-on-the-node`
   against `recorded-before`) and so did
   `test_a_branch_listing_after_a_cancel_overtakes_the_backlog`, as a side
   effect of its own `updateAvailable` assertion. This also confirms the test's
   comment that the head is in hand before the cancel.
2. The sync's `GetBlob` reply paints `body.replace(/[0-9]+$/, "00")` into the
   viewer: the wrong file's content, of the same length for `f59`. **Survived**
   in `test_a_file_queued_by_the_sync_still_opens_after_the_cancel` and
   `test_a_file_the_cancelled_sync_had_queued_opens_after_the_cancel` (17
   passed). It was killed only in `test_a_file_the_sync_is_already_fetching_still_opens`,
   and only because `f0` is one character shorter than `f00`. See the finding
   below.

## Findings

- [x] **`spec-writer`** — spec, "A cancelled sync stops reaching its node" and
      "Cancel, then poll, in the order a user follows": the bound and the poll's
      wait are never given a value. The spec says "a fixed number", "the bound"
      and "that wait", and a scenario that says "MUST NOT exceed the bound"
      cannot be tested until the bound is a number.
      **Scenario:** a correct implementation with a bound of 5 satisfies the
      spec as written and fails every bound test, because `tst_sync_cancel.qml`
      hardcodes `bound: 4`, taken from `design.md` Decision 2. The same holds
      for `sync.yaml`'s `timeout: 30s`. Neither literal carries a `NO SPEC:`
      comment, and the value lives in a document the spec is not supposed to
      depend on. Either the requirement states the number (and the wait), or
      the tests carry `NO SPEC:` comments naming the chosen value.
      **Severity:** medium.
      **Outcome (`spec-writer`): fixed in the spec.** The bound is now the
      number: "A cancelled sync stops reaching its node" says at most 4 of the
      sync's requests reach the seed after a cancel, and every scenario that
      said "the bound" says 4, so `tst_sync_cancel.qml`'s `bound: 4` is the
      spec's value rather than `design.md`'s. The poll's wait is not a property
      of the module, so it stays the test's choice, and the scenario now
      constrains it instead of naming it: "a wait longer than that seed takes
      to serve 4 of the sync's requests and the poll's own", against a sync
      cancelled with more left than the seed can serve within that wait.
      `sync.yaml`'s 30 s is a value satisfying that, not an unspecified one, so
      it needs no `NO SPEC:`. The lower side of the constraint is what the
      `tester`'s "Cancel, then poll" precondition finding below checks.

- [x] **`spec-writer`** — spec, scenario "The bound holds against the local
      node", and the clause "the Radicle home for `local*` ones": untestable as
      written, and unreachable through the control the spec defines a cancel by.
      **Scenario:** nothing in the tree can start a Local-mode sync: the
      control is `remote`-only (`RepoView.qml:347`), the prefix is derived
      above the component layer's fake, and `local.yaml` has no sync step.
      `grep -n "sync" openspec/specs/source-modes/spec.md` finds no requirement
      stating the control is remote-only, so the cross-capability fact that
      makes the scenario vacuous is written nowhere. Decide: strike the
      scenario and the `local*` clause and say the control is remote-only in
      this capability or in `source-modes`; or, if a Local-mode sync is meant to
      exist, restate the scenario as something a test can observe. Do not leave
      a MUST that no layer can fail.
      **Severity:** medium.
      **Outcome (`spec-writer`): fixed in the spec — struck, and the
      remote-only control stated in this capability.** The scenario "The bound
      holds against the local node", the `local*` clause and "the same for
      every mode" are gone. A new requirement, "The sync control is the
      seed's", says the control MUST be shown while the derived prefix is
      `remote` and MUST NOT be shown while it is `local`, so a sync is made of
      `remote*` calls and the node it reads from is the seed; the cancel
      requirements now say "the seed". It sits in `repo-sync` rather than
      `source-modes` because it is a rule about the sync control, and the
      Purpose names `source-modes`' "The method prefix is derived from the
      mode" as the boundary. The `local` half is new coverage: no test shows
      the control hidden today (`tst_sync_cancel_view.qml` checks only that it
      is shown for `remote`). That makes the `tester` finding below on a
      `source: "local"` data row moot as written.

- [ ] **`tester`** — `tst_sync_cancel.qml`, the "still opens" tests: they
      assert `viewerBodyLength` against `"content of f59".length` and
      `viewerTitle` against `"f59"`. The title comes from the entry the test
      passed in, not from the reply, and the length is the same for `f10`
      through `f59`. The spec says the contents MUST be shown "matching what the
      node holds for that path".
      **Scenario:** a reply that paints another file's contents of equal length
      passes both `f59` tests. **Measured:** mutation 2 above survived both;
      only the `f0` test caught it, by one character of length. `SourceTab`
      exposes no body accessor, only `viewerBodyLength`, so a test hook is
      needed to compare the content itself (a change to `SourceTab.qml`, so
      the `dev-writer` may need to add it).
      **Severity:** medium.

- [ ] **`tester`** — "A cancel settles every request it stops" has no
      component-layer test, and the refusal path after a cancel has none at
      all. `busy clears` and `no error` are asserted only in `sync.yaml`'s last
      steps, which is a CI-only, seed-dependent run I could not execute. In
      both component fakes `onFail` is called only for an unknown method, so
      a request that is refused after a cancel is never driven.
      **Scenario:** the spec says "answered or refused" and that a stopped
      request MUST NOT surface as an error. The `GetTree` and `GetBlob`
      `onFail` handlers carry their own epoch checks (read, not mutated, so
      unproven): dropping the check in one would let a refused read from a
      cancelled sync free a slot and bump `syncDone` in the next sync, and
      nothing here would go red. Add a row to the late-reply tests that serves
      the stale reads with `onFail`, and assert no error is raised and the new
      sync's counters do not move.
      **Severity:** medium.

- [x] **`spec-writer`** — `tst_sync_cancel.qml`,
      `test_a_late_reply_does_not_free_a_slot_in_the_next_sync`: it pins that a
      running sync never has more than `bound` reads with the node at once,
      which no scenario states. The spec bounds only what a cancelled sync
      sends after the cancel, and `design.md` Decision 5 concedes the module
      can hold up to 8 right after a cancel and restart. Unmarked.
      **Scenario:** an implementation that cancelled correctly but let a
      running sync keep 6 reads out would pass every scenario and fail this
      test. Either add the running sync's cap to the spec or have the test say
      `NO SPEC:`.
      **Severity:** low.
      **Outcome (`spec-writer`): fixed in the spec — the cap is now a
      requirement.** "A running sync keeps at most 4 requests outstanding"
      says a running sync MUST NOT have more than 4 of its requests
      outstanding at once (outstanding: made, and no reply or refusal yet),
      with two scenarios: "A large sync never has more than 4 requests
      outstanding", and "A late reply from a cancelled sync does not widen the
      next one", which is what this test pins. It bounds only the running
      sync's own requests, so it agrees with `design.md` Decision 5's 4 + 4
      right after a cancel and restart. The plain large-sync scenario has no
      test of its own; the late-reply test's second sync runs to completion
      under it, but with the old replies interleaved.

- [x] **`spec-writer`** — `sync.yaml:247`, `NO SPEC: "Update" is a colour and a
      label, never a gate`: still unanswered. The change moved the two
      cancel-related markers into the spec and left this one, and the new
      capability's Purpose ("started and cancelled from one control") is the
      natural home. The test pins it by clicking the control in the `Update`
      state. The owner decides whether it is in scope for this change.
      **Severity:** low.
      **Outcome (`spec-writer`): marker left in place, by decision.** It is a
      rule about starting a sync, which issue #92 does not scope, and no cancel
      requirement rests on it: "A sync started after a cancel downloads
      everything" says nothing of the control's state. The marker stays as the
      record of an unspecified choice, and the step it labels still pins the
      behaviour. If the owner wants it specified, it is one requirement in
      `repo-sync` beside "The sync control is the seed's", in this change or a
      later one; that call is the owner's, and this outcome does not pre-empt
      it.

- [ ] **`tester`** — `sync.yaml`, "Cancel, then poll": the scenario's GIVEN is
      "a sync whose full download takes longer than a staleness poll is allowed
      to wait", but the spec runs against whichever repository the seed lists
      first, and the only precondition step asserts `syncing === true` at the
      cancel. **Scenario:** if the first repository is small enough that its
      whole backlog is served inside 30 s, the poll step passes on a module
      that never cancelled anything, and the spec reports a regression test
      that cannot fail. Add a precondition step before the cancel asserting
      that enough remains (for example `syncQueued - syncDone` above the bound
      by a margin), so a short repository fails at the precondition, naming
      itself, as the existing "still running" step already does for the other
      end. **Not run here:** `sync.yaml` is CI-only; I read it, I did not
      mutate it.
      **Severity:** low.

- [ ] **`tester`** — local scenario, if the spec keeps it: the cheapest test
      available. `tst_sync_cancel.qml`'s fake carries no `source` property, so
      nothing shows that `SourceTab`'s dispatch does not branch on the mode.
      Give the fake `source: "local"` as a data row of the large-sync test.
      It would not prove the prefix, which is `Main.call()`'s, but it removes
      the only part of "the same for every mode" the component layer can see.
      Moot if the spec-writer strikes the scenario.
      **Severity:** low.
