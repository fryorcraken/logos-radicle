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

- [ ] **`spec-writer`** — `embedded-state`, "Embedded's empty surface reports one
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
- [ ] **`spec-writer`** — `embedded-state`, `ADDED` "Each state names its own
      next action": the first sentence says each state MUST carry "its own named
      action", and the next sentences give states with none (`stopped` with an
      unencrypted key "names none", `blocked` "no action that writes"; `starting`
      and `running and empty` name none either, per their own requirements).
      **Scenario:** a check of "every state names an action" fails for four of
      the seven, and the same requirement then says a state whose action nothing
      can carry out MUST still name it. Carried from the live text. **Needs:**
      "each state that has an action names it" or an explicit list of the states
      that have none. **Severity:** low.
- [ ] **`spec-writer`** — optional-phrased text in requirements the change
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
- [ ] **`tester`** — `embedded-setup` `ADDED` "The setup presents no start step
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
