# Findings from the tester stage

Written by the `tester` for `strict-spec-validation`, tasks.md group 3. This is
its own file rather than `spec-test.md`, which belongs to the `spec-test-reviewer`.

- [x] **`spec-writer`** — `specs/embedded-header/spec.md`, ADDED requirement "One
      rule decides the header identity for every mode", scenario "A mode the
      header was not written for still shows its DID" — **the scenario cannot be
      satisfied by the code, and cannot be satisfied without breaking
      `source-modes`.**
      **Scenario:** GIVEN a header in a mode other than `explore`, `local` and
      `embedded` whose backend reports a distinctive DID, THEN that DID MUST be
      displayed in full. **Trace:** `SourceState.modeHasIdentity` is
      `current === "local"` (`radicle-ui/src/qml/SourceState.qml:180`), and
      `current` is `(mode === "local" || mode === "embedded") ? "local" : "remote"`
      (`:146`). A mode string outside the three derives `remote` and so has no
      identity: `modeHasIdentity` is false and the header shows nothing. That is
      what `source-modes` requires ("An unrecognised mode derives the seed
      prefix", live `openspec/specs/source-modes/spec.md:295`, and "An unknown
      mode on disk grants no local access"), and `tst_source.qml`
      `test_an_unknown_mode_routes_to_the_seed_not_the_node` pins it. A mode
      reaches the `local*` methods, and so gains an identity, only by being
      NAMED in `current` — which is the list the requirement says the rule must
      not be. **There is no component-layer input that is "a mode nobody wrote a
      rule for" and that shows a DID**, so no test of this scenario can pass
      against the code and also stay consistent with `source-modes`. This is a
      spec defect, not a code defect: the old requirement carried this as prose
      ("right for a mode added later") with no scenario, and the split-out
      scenario turned an aspiration about future edits into an input nobody can
      supply. **What is pinned instead:** `tst_source.qml`
      `test_an_unrecognised_mode_has_no_identity_to_show` (marked `NO SPEC`),
      which asserts the header's rule agrees with the routing for five mode
      strings including two unrecognised ones; and the three existing tests the
      code comment names. **Needs:** either restate the scenario as a
      relationship the code has ("a mode has an identity exactly when it routes
      to the `local*` methods"), or drop it and keep the requirement's prose.
      Task 3.1 is left unticked for this reason.
      **Outcome (`spec-writer`): fixed, by restating the scenario as the
      relationship the code has.** The finding is right: the scenario was not
      live text but one the split-out wrote, and it named an input nobody can
      supply. The requirement's two `MUST`s are unchanged. It now also states
      the term the live text left implicit: a mode has an identity exactly when
      it derives the `local` method prefix, as `source-modes`' "The method
      prefix is derived from the mode" decides it, and whether that identity
      exists yet is the reported DID. The scenario is now "The identity is
      shown in exactly the modes that derive the local prefix": with a
      distinctive DID reported, it is displayed in `local` and in `embedded`,
      and not in `explore` or in a string the view does not recognise. That
      agrees with `source-modes` and with the code, and the scenario count is
      unchanged (one). The MODIFIED requirement's no-identity sentence was
      reworded so it no longer calls an `embedded` mode with an empty DID "a
      mode with no identity", which the new definition would have
      contradicted. Its `MUST` and all three scenarios are unchanged. The
      `NO SPEC` test `test_an_unrecognised_mode_has_no_identity_to_show` should
      drop its marker, name this scenario, set a distinctive `nodeId` and
      assert `slotIdentity.visible` (what is displayed) for each mode
      alongside `slotState.current`, not `modeHasIdentity` alone. Task 3.1 can
      then be ticked.

- [x] **`dev-writer`** — `radicle-ui/src/qml/SourceState.qml:162`, comment on
      `modeHasIdentity` — "this rule is right for a fourth mode before anyone
      writes one" is not true as the code stands. A fourth mode is right only
      once it is added to the list in `current`; until then it derives `remote`
      and shows no identity, which is the safe direction and what `source-modes`
      requires. The comment is existing text that the strict-spec-validation
      change did not touch; it is the same claim as the defect above, in the
      code. Not changed by the tester (implementation).
      **Fixed** in the commit "Correct the fourth-mode claim about the header
      identity rule" (comment only; no code or test behaviour changed, so no
      test can fail without it). The `modeHasIdentity` comment now defines an
      identity as deriving the `local` prefix, citing `embedded-header`'s
      requirement, and says a fourth mode gains one in the same edit that adds
      it to `current` and not before, routing to the seed until then. The same
      claim sat in the doc comment above `tst_source.qml`'s
      `test_having_an_identity_is_derived_rather_than_listed` and is corrected
      there too (the comment only, not the test). The comment's list of tests a
      `mode === "local"` mutation reddens now names the new scenario test as a
      fourth by reading, and says that one was not run under the mutation.
      `design.md`'s `embedded-header` entry was the same claim in prose and now
      states the relationship. `Main.qml`'s "right for a mode added later
      without anyone editing this line" was left: it is scoped to the header
      line, and that is true.

- [ ] **`runner`** — **nothing the tester added or changed was executed.** The
      pod has no Qt 6 `qmltestrunner`, no C or C++ compiler (`cc`, `g++`,
      `clang++` absent; `cargo test` stops at "linker `cc` not found" while
      building `proc-macro2`'s build script), no `nix`, and no `crt1.o`. Only
      `cargo fmt --check` (clean) and a brace/paren/bracket balance check of the
      four edited `.qml` files (unchanged from `HEAD`'s own count, which the
      string-stripping regex gets wrong the same way) ran. Every new or changed
      test therefore has a **predicted** failure and no observed one; the first
      real execution is CI's. **Needs:** a CI run of `QML lint and component
      tests`, `Rust backend tests` and `Build LGX (linux-amd64)` (which runs the
      core unit tests) over this branch, read to the end, before the tests row is
      relied on; the `spec-test-reviewer` mutation list in the tester's report is
      the work to repeat where a toolchain exists.
