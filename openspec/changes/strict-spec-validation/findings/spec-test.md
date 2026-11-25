# Findings from the spec-test review of `strict-spec-validation`

Reviewed by the `spec-test-reviewer` against the spec deltas, the live specs they
modify, issue #93 and the tests. The implementation was not read, except three
lines in `Main.qml` and `SourceState.qml` named below.

**No mutation was run.** The pod has no Qt 6 `qmltestrunner` and no C or C++
compiler (`which cc g++ qmltestrunner6 nix` finds none; `cargo` exists but cannot
link), so every verdict on "can this test fail" below is **mutation by reading,
and a prediction**. No mutation is left in the tree, and none was made. What did
run: `npx --yes @fission-ai/openspec@2.1.1 validate strict-spec-validation
--strict --no-interactive` (reports the change valid), and
`validate --specs --json` on the unarchived tree (9 passed, warnings only; the
two requirements my own length count puts over 500 that the delta leaves alone,
`A view renders settings from the reply it was given` and `Unseeding removes the
policy and is idempotent`, are not among the CLI's warnings, so the delta
covers every requirement the CLI measures as long).

## Verified clean

- **Part 1, coverage.** All 37 `ADDED` scenarios have a test, counted per
  capability from the deltas (header 1, identity 4, setup 20, state 5,
  settings 3, config 3, source-modes 1) and mapped by reading each scenario's
  GIVEN/WHEN/THEN against the test body. The tasks.md group-3 claim of "29 QML, 4
  Rust, 7 C++" matches a count of the `test_*`/`LOGOS_TEST`/`#[test]` functions
  the diff adds or changes (13 + 8 + 6 + 2, 3 + 1, 5 + 2). No scenario is
  untestable as written, after the `embedded-header` rewrite the tester's
  finding drove. Layers fit, with the one exception in the first `tester` box
  below.
- **Part 2, tests that cannot fail for the reason they name.** Read, none found
  of the three shapes. The tests that carry their own control leg (a changed
  single input, same harness) are the strongest: the blocked-home, no-start-
  control, passphrase-not-shown, `canWrite`-not-`encrypted`, header-shown-in-
  exactly-the-local-prefix-modes, serving-node-lands-on-identity, withheld-
  capabilities and resumed-embedded-step tests, and the Rust two-fields and
  dynamic-peers tests (non-default seed values, so a reset-to-defaults write is
  visible).
- **Part 3, `NO SPEC`.** None of the 29 + 4 + 7 new or changed tests carries the
  marker; the markers in the tree (`sync.yaml`, `tst_setup_wizard_view.qml:194`,
  `node_seeding.rs:143`, `node_config.rs:443`, `:463`) predate this change. No
  unmarked beyond-spec behaviour found, apart from the ordering defect below.
- **Part 4, moves.** No requirement moved between capabilities and the deltas
  carry no `REMOVED` section. Within a capability, every `MODIFIED` heading
  matches a live heading exactly, no live scenario is dropped, and every live
  scenario body is byte-identical except the two that lose their trailing
  after-scenario passage (`node-paths` "An unrecognised mode resolves nothing",
  `source-modes` "An unprovisioned embedded home is not reported as empty"):
  both removed passages are reasoning, no scenario step changes.
- **MUST/MUST NOT, per `MODIFIED` requirement.** Diffed each against its live
  counterpart sentence by sentence (a script, then by reading what it flagged).
  Every live `MUST`/`MUST NOT` is retained in the same requirement or in the
  `ADDED` requirement split from it, in shorter words, with **one** exception,
  the one the proposal declares: `node-paths`' "Socket precedence prefers the
  runtime directory over the home" no longer says on its own that `local` mode
  MUST find the home-derived socket when no runtime directory exists. I agree it
  is implied (the scenario "Without a runtime directory the socket falls back
  under the home" and the retained "a fallback that MUST be retained" pin the
  resolver that mode uses), so no box; it is the only dropped sentence.
- **`remote*` / `local*` symmetry.** The `ADDED` shapes are `getSettings`,
  `getEmbeddedIdentity`, `getNodeConfig`/`setNodeConfig`; none has a `remote*`
  twin, so nothing is fixed for one prefix and silent on the other.
- **MUST/MUST NOT only.** No uppercase SHOULD, MAY, SHALL, RECOMMENDED or
  OPTIONAL anywhere in the deltas. The lowercase exceptions are the third box.
- **Part 5, staleness against issue #93** (read fresh with `gh issue view 93`).
  The issue asks to reinstate `--strict` and split the long requirements so
  strict passes; the proposal does that. One addition the issue never stated:
  `embedded-header` now defines "has an identity" as "derives the `local` method
  prefix", which fixes the header's behaviour for a mode string the view does
  not recognise. It is declared in the proposal and agrees with `source-modes`
  and the code, so no box, but the owner should know it is more than a split.

## Findings

- [x] **`spec-writer`** — `embedded-state`, "Embedded's empty surface reports one
      of seven states" (MODIFIED) and the `ADDED` "Each embedded state has one
      condition": the listed order contradicts the listed conditions, so two of
      the seven states can never be the first that holds.
      **Scenario:** a start is refused (or is outstanding) while the node still
      reports `running:false, serving:false`, which is the ordinary case. The
      spec's rule is "the first, in the order blocked, no identity, stopped,
      starting, start failed, ...  whose condition holds"; `stopped` is
      "the node reports neither running nor serving", listed before `starting`
      and `start failed`, and holds, so the literal answer is `stopped` every
      time. The tests and the code say `startFailed`
      (`tst_embedded_panel.qml:859`, `:968`, and the new `:1074`, over the
      defaults `embeddedRunning:false`, `embeddedServing:false`), so the spec,
      not the tests, is wrong; a reader implementing from the text would never
      render `start failed`, and `starting` would be reachable only with
      `running:true`. The ordering text is carried unchanged from the live spec;
      the new `ADDED` requirement restates the conditions without the exclusion
      and makes the defect easier to read. **Needs:** `stopped` stated as "and no
      start outstanding and no start refused", or `starting` and `start failed`
      moved ahead of it. **Severity:** medium (spec defect, no code defect).
      **Measured:** read only; the three tests above pin the opposite of the
      literal text.
      **Outcome (`spec-writer`): fixed for `start failed`, and fixed the other
      way for `starting`.** The live spec was wrong in the same way: its order
      is the one carried, and its own scenario "A refused automatic start is
      reported as a refusal" requires start failed over a stopped node. The fix
      is in the `ADDED` "Each embedded state has one condition" only; the order
      in the `MODIFIED` requirement is unchanged. `stopped` now reads "neither
      `running` nor `serving`, and the last `startNode` not refused". The
      half of **Needs** that excludes an outstanding start from `stopped` is
      **not** taken: it would change what the code must do.
      `EmbeddedState.qml`'s `current` renders `stopped`, not `starting`, while a
      start is outstanding and the node still reports `running:false`, and
      `tst_embedded_state.qml`'s
      `test_no_action_is_named_while_a_start_is_outstanding` and
      `test_an_outstanding_start_withholds_a_hosted_start_control` pin that
      (`compare(st.current, "stopped")` with `startPending` true). That is the
      live order read literally, so the spec now says it: `starting` reads "a
      `startNode` outstanding, with `running` or `serving`". Checked against
      `current` case by case (node stopped or loaded, start outstanding or not,
      refused or not): the stated conditions in the stated order give the
      code's answer in every case.
      That exposed three `MODIFIED` scenarios, carried byte for byte, that
      disagree with the order, and each now gets one step changed. "The reply
      clears the outstanding start, not the request" and "The start action is
      withheld while a start is outstanding" now give the node status their
      requirement assumes (`running:true`, `serving:false`) in the **GIVEN**.
      "A stopped node with an unencrypted key starts by itself" said "the
      rendered state MUST be the starting one" over a node reporting
      `running:false`; it now says "while that call is outstanding and the node
      still reports `running:false`, the rendered state MUST be the stopped
      one". No code changes. `proposal.md` records the three scenario changes.
      **For the `tester`:** `test_an_unencrypted_stopped_node_is_started_without_being_asked`
      (`tst_embedded_panel.qml:713`) is the test for that last scenario and
      asserts no state today. It needs a leg that sets
      `app.embeddedStartPending = true` after the automatic start and asserts
      `list.embedded.current === "stopped"`; it is the one changed step no test
      pins. For the two changed **GIVEN**s, every test in
      `tst_embedded_state.qml` that sets `startPending = true` and expects
      `starting` already sets `running = true` (`:170`, `:275`, `:339`, `:377`,
      `:473`, `:574`), so whichever of them the `tester` maps to those two
      scenarios needs reading against the new text, not changing. The three tests
      this box names (`tst_embedded_panel.qml:859`, `:968`, `:1074`) already
      assert what the new text says.
- [x] **`spec-writer`** — `embedded-state`, `ADDED` "Each state names its own
      next action": the first sentence says each state MUST carry "its own named
      action", and the next sentences give states with none (`stopped` with an
      unencrypted key "names none", `blocked` "no action that writes"; `starting`
      and `running and empty` name none either, per their own requirements).
      **Scenario:** a check of "every state names an action" fails for four of
      the seven, and the same requirement then says a state whose action nothing
      can carry out MUST still name it. Carried from the live text. **Needs:**
      "each state that has an action names it" or an explicit list of the states
      that have none. **Severity:** low.
      **Outcome (`spec-writer`): fixed, in the delta.** The live spec was wrong
      in the same way ("Each state MUST carry ... its own action naming what
      the user may do next", beside states that offer none). The requirement
      now says each state MUST carry its own sentence; names the four that
      MUST name their own action (no identity, setup; stopped with an encrypted
      key, a passphrase and a start; start failed, a start; not serving, a
      restart); and says blocked, starting, running and empty and stopped with
      an unencrypted key MUST name none. That is `EmbeddedState.qml`'s
      `actionKind` exactly, so no code changes. Two of the "none" entries were
      not stated anywhere before (running and empty; starting only as "MUST NOT
      name the start action"); both are already pinned by
      `test_each_state_says_something_different` (`tst_embedded_state.qml:146`,
      whose expected string ends `starting/ ... runningEmpty/`). The `ADDED`
      scenario is unchanged, so no test needs revisiting for this box.
- [x] **`spec-writer`** — optional-phrased text in requirements the change
      touches: `embedded-setup` `ADDED` "A resumed step behaves as one reached
      by advancing" says "what it **may** do, what it **must** state" in lower
      case, and `embedded-state` "The no-repositories wording is withheld where
      no node exists" says the placeholder "**MUST be permitted**" in the running
      and empty state, which is a MAY written as a MUST (its scenario "A serving
      node that holds nothing **may** say so" says the same). The overlay's rule
      is that an optional requirement is either a requirement or it is not.
      **Needs:** state the obligation (the sentence rendered there MUST say the
      node lists what it is seeding) and drop the permission. **Severity:** low;
      both are carried from the live text.
      **Outcome (`spec-writer`): fixed for both requirements; the scenario
      title rejected.** The live text was wrong in the same way in both. In
      `embedded-state`, "It MUST be permitted in the running and empty state,
      and the sentence rendered there MUST also say ..." is now "In the running
      and empty state, the sentence rendered MUST say that this node lists what
      it is seeding": the permission obliged nothing, so dropping it changes
      nothing an implementation must do. In `embedded-setup`, "what it may do,
      what it must state and what blocks it are unchanged" is now "it MUST offer
      the same actions, state the same text and be blocked by the same
      findings". The scenario title "A serving node that holds nothing may say
      so" is **kept**: a title is a name, not a requirement, its one step is
      already a `MUST`, and `test_a_serving_node_that_holds_nothing_may_say_so`
      (`tst_embedded_panel.qml:343`) is matched to it by that name, so renaming
      it would move a mapping and check nothing new. No scenario step changed
      for this box, so no test needs revisiting.
- [ ] **`tester`** — `embedded-header` `ADDED` "One rule decides the header
      identity for every mode": the scenario is pinned on a **hand-written copy**
      of `Main.qml`'s binding, not on `Main.qml`. `tst_source.qml:883` hosts
      `NodeIdentity` under `visible: slotState.modeHasIdentity &&
      slotIdentity.nodeId !== ""`, and `Main.qml:1052` has the same text beside
      it; the file says so (`Main.qml` cannot be instantiated in a component
      test). The new test
      `test_the_identity_is_shown_in_exactly_the_modes_that_derive_local` is
      strong against `SourceState.modeHasIdentity` (a `mode === "local"` there
      reddens its `embedded` case) but cannot see the line that actually decides
      what the header draws.
      **Scenario:** change `Main.qml:1052` to `root.mode === "local" && ...`:
      Embedded shows no DID again, which is the defect this requirement exists
      for. **Predicted, not run:** all 29 QML tests pass; `local.yaml` asserts
      only `root.nodeIdentity === ''` in an Embedded home with no identity (true
      under the mutation) and `root.nodeIdentity !== ''` in Local, so the e2e
      layer does not see it either. **Needs:** a sitometres step over an Embedded
      home that holds an identity, or the binding moved into a component the
      test hosts for real; or the owner accepts the gap and the spec's scenario
      is understood as pinning the rule, not the wiring. **Severity:**
      low to medium; the same limit applies to the existing scenario "Embedded
      shows its DID as Local does".
      **Outcome (`tester`): left OPEN, deliberately, and no test added.** The
      finding is right and I could not close it with a test. The line that
      decides what the header draws (`Main.qml:1052`) sits in a file no
      component test can instantiate, so every test at this layer is pinned on
      a copy of it; the two ways to see the real line are both outside what I
      may do. (1) Move the header's identity item into a component a test hosts
      for real: that is a source change, and this pass is tests and fixtures
      only. (2) A sitometres step over an Embedded home that holds an identity:
      `local.yaml` never runs the wizard (its own comment at the Embedded steps
      says so), so no such home exists in that spec; and it asserts the root's
      `nodeIdentity`, a property, not that the header item is visible, so a
      step added there would need a seeded embedded identity and a visibility
      probe, neither of which I can write or run in this pod. A source-text test
      that greps `Main.qml` for the binding was considered and not written: this
      repo has no test that reads a source file, it would pin spelling and not
      behaviour, and it cannot fail for the reason the scenario names. **For the
      owner:** accept the gap (the scenario then pins the rule, not the wiring,
      as the finding allows), or schedule the component extraction. The box
      stays open because that choice, and the issue that would hold it, are not
      mine to make or file.
- [x] **`tester`** — `embedded-setup` `ADDED` "The setup presents no start step
      and starts nothing": the `startNode` leg of
      `test_an_existing_identity_with_a_stopped_node_is_walked_without_a_start`
      (`tst_setup_wizard.qml`, new test; and its sibling at `:1271`) cannot fail.
      The fake documents that it has no `startNode` entry point (`:259-268`: "the
      entries ... are therefore unreachable rather than merely absent"), so a
      call-log search for a `startNode` entry reads an empty set whatever the
      flow does. The legs that can fail are the list of steps presented (a fifth
      step reddens it) and `typeof flow.startNode === "undefined"` (reddens a
      property of that exact name).
      **Scenario:** a start wired into `SetupFlow` under another property name,
      called from the network step, and the fake given no matching function:
      nothing is logged and no step is added. **Predicted, not run:** survives.
      **Needs:** either say in the test that the log leg is a tripwire for a
      future fake, or give the fake a `startNode` that logs, so that wiring one
      in anywhere is seen. **Severity:** low.
      **Outcome (`tester`): fixed, by the first of the two Needs and one more
      leg.** A fake `startNode` cannot be wired: `SetupFlow` has no such
      property, and assigning one in the harness is a QML error ("non-existent
      property"), which is why the fake has none. So the log legs in
      `test_no_step_of_the_setup_starts_a_node`,
      `test_an_existing_identity_with_a_stopped_node_is_walked_without_a_start`
      and `test_a_failing_preflight_writes_nothing_either`
      (`tst_setup_wizard.qml`) are now commented as tripwires that cannot fail
      until a start is wired into the fake, and the comment beside the fake no
      longer says they make the scenario structural. The leg that can fail is
      new: `assertNoStartMember(flow)` enumerates the flow's own members and
      fails on any whose name starts with `start` or contains `Start`
      (`startEmbeddedNode`, `fetchStart`, `canStartNode`; not `restart`), so the
      finding's own scenario, a start under another name, is seen. It first
      requires the enumeration to list `createIdentity` and `advance`, so a
      runtime that enumerated nothing fails loudly rather than passing over an
      empty list. **Predicted, not run:** adding `property var startEmbeddedNode`
      to `SetupFlow.qml` reddens `test_no_step_of_the_setup_starts_a_node`,
      `test_an_existing_identity_with_a_stopped_node_is_walked_without_a_start`
      and `test_a_failing_preflight_writes_nothing_either` at
      `assertNoStartMember`, while the previous version of all three passed.
      **Limit:** a start injected under a name with no `start` in it
      (`launchNode`) is still not seen; and whether QML's `for...in` lists a
      QML object's declared properties is a belief about Qt 6, not something I
      could run, which is what the precondition is for. If CI fails on the
      precondition, the leg should be dropped, not loosened.
- [ ] **`runner`** — **part 2's mutations were not run, by this review or by the
      tester**, so no new test has an observed failure. Where a toolchain exists
      (Qt 6 runner; `cc` for `cargo test`), the three worth running, each in its
      own single-file form from the overlay's Test layers, restoring after each:
      (1) `SourceState.qml` `modeHasIdentity`, `current === "local"` to
      `mode === "local"` against `tst_source.qml` (predicted: the new
      header test and `test_having_an_identity_is_derived_rather_than_listed`
      fail on `embedded`);
      (2) `SetupFlow.qml`, stop waiting for the capabilities reply before the
      resume lands, against `tst_setup_wizard.qml` (predicted: the withheld-
      capabilities test fails at its first `compare(flow.step, "preflight")`);
      (3) the embedded state object (`EmbeddedState.qml`), key the prompt on
      the `canWrite` the test's `app` exposes instead of `encrypted`, against
      `tst_embedded_panel.qml` (predicted:
      `test_a_key_that_cannot_sign_is_not_taken_for_an_encrypted_one` fails).
      Also worth one run: delete the derived-member refusal in the node-config
      write path and run `node_config.rs`; **predicted to survive**, harmlessly,
      because `inboundReachable` is then refused by name as an unexposed field,
      which is the same observable (an equivalent mutation, not a test defect).
      **Severity:** the same open box as the tester's own `runner` entry in
      `findings/tester.md`; this one adds the list.
