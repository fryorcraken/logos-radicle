# Design review: strict-spec-validation

Reviewed: `design.md` against `.github/workflows/ci.yml` (the two new `lint`
steps), `CLAUDE.md` and `docs/OPENSPEC-ARCHIVE.md` (the doc edits),
`SourceState.qml` and `tst_source.qml` (the one code-adjacent edit), issue #93
(read fresh with `gh issue view 93`), and the live specs under
`openspec/specs/` against their deltas.

**In good shape, checked:**

- Decisions 1, 2 and 5 match the code. The step is last in `lint`, is pinned
  `@2.1.1` once, sets `OPENSPEC_TELEMETRY: "0"`, runs `--specs --strict
  --no-interactive`, and `setup-node@v7` at `"24"` is the same pin `ui-specs`
  carries (`ci.yml` lines 122-124 and 281-283). The claim that `Lint` is a
  required check and a new job name would not be holds:
  `gh api .../required_status_checks` lists `Lint` and no other candidate name.
- Both of issue #93's asks are met: `--strict` is reinstated (the step), and
  all nine capabilities carry a delta. The issue says "split"; the change
  mostly cuts reasoning out instead. Decision 4 argues that departure
  (a spec is a behaviour contract), so it is justified, not silent.
- The blind spots design.md lists (missing `spec.md`, declined promotion,
  MODIFIED that shrinks but stays long, prose after the scenarios) agree with
  `docs/OPENSPEC-ARCHIVE.md` and with the comment beside the step. The three
  texts say the same thing.
- Reasoning carried out of the specs: I read the live text against the
  deltas for `node-paths` (all of it), and for the cut passages of
  `embedded-setup`, `source-modes` and `embedded-header`. Every cut passage
  I found has an entry under "Reasoning carried out of the specs". Not
  checked passage by passage: `embedded-identity`, `embedded-state`,
  `module-settings`, `node-config`, `node-seeding`.
- Not verified here: the CLI is not installed in this tree, so the claims
  about `validator.js`, `opt-out.js` and the 73-warning count are the
  `dev-writer`'s measurements, not mine.

## Findings

- [x] **`dev-writer`** — `design.md:549-575`, `proposal.md:32-38` — a `MUST` was
      removed and the decision is not recorded as one. The live
      `node-paths` "Socket precedence" requirement said `local` mode MUST still
      find the home-derived socket when no runtime directory exists. The delta
      drops it; the only account is a reasoning bullet ("the precedence it
      restated already returns the home-derived socket"), plus the proposal's
      "one exception". Issue #93 asks for a split, not a weaker contract, so
      this is a departure that needs arguing in Decisions: what was chosen,
      the alternative (keep the sentence, or move it into the "Without a
      runtime directory" scenario, which already pins the resolver's answer),
      and what it costs. One gap in the argument: the dropped sentence was
      about `local` mode's outcome, while the retained scenario pins
      `resolveSocket`'s outcome. Say whether any scenario pins that `local`
      mode passes no runtime directory through, or say none does. A gap, not
      a contradiction. I did not count every `MUST` across the nine specs, so
      "the one exception" is the author's claim, not mine.
      **Fixed** in the commit "Act on the dev-writer review findings in
      strict-spec-validation": `design.md` Decision 6 records the drop as a
      decision (what was chosen, keeping the sentence and moving it into the
      "Without a runtime directory" scenario as the alternatives, and the
      cost), and the `node-paths` reasoning entry now points at it. "The one
      exception" is no longer only the author's claim: it cites the
      spec-test review's sentence-by-sentence diff of every `MODIFIED`
      requirement, which found no other. The gap is answered **none does**:
      no scenario, live or delta, pins that `local` mode passes an empty
      runtime directory through when `XDG_RUNTIME_DIR` is unset
      (`resolvePathsFromEnv`), and under `radicle/tests/` the only test that
      sets `XDG_RUNTIME_DIR` is `test_local_writer.cpp:306`, to a value
      (`grep -rn XDG_RUNTIME_DIR radicle/tests`). The live sentence had no
      scenario either. A scenario for it is observable behaviour, so it is
      reported to the `spec-writer` in the hand-back rather than written.
      Prose only; no test can fail without it.

- [x] **`dev-writer`** — `design.md:566-575` — a cross-capability trap now
      lives only in the archived design. The cut note said whoever changes the
      sanitising fallback ("unknown stored mode reads as explore") must
      re-check `node-paths`' "An unrecognised mode resolves nothing", because
      that `MUST NOT` holds by composition. The delta removes the note from the
      spec, and `grep` finds no counterpart in `radicle/src` (the only
      "sanitis" hit is a test comment in `test_settings_store.cpp:272`).
      `source-modes` does carry a scenario ("An unknown mode on disk grants no
      local access") that would catch the break, so this is softer than a
      missing guard. But the overlay's `## design-reviewer` rule is that a trap
      belongs where the next toucher of the file looks. Put a pointer at the
      fallback in the settings store, or say in the entry that the
      `source-modes` scenario is the substitute and name it.
      **Fixed** in the commit "Act on the dev-writer review findings in
      strict-spec-validation", by both. `SettingsStore::load()`
      (`radicle/src/settings_store.cpp`, beside the `isKnownMode` fallback)
      now says that `node-paths`' "An unrecognised mode resolves nothing" and
      `source-modes`' "An unknown mode on disk grants no local access" hold
      only through that line, names
      `a_persisted_mode_this_build_does_not_know_does_not_alias_the_local_profile`
      (`test_radicle_impl.cpp:1395`) as the test that sees the second, and
      says a change to the fallback means re-checking both. The `node-paths`
      entry in `design.md` points at the comment. Comment and prose only; no
      test can fail without it.

- [x] **`dev-writer`** — `design.md:131-137` — Decision 3's blind spots have no
      issue. Decision 4 states the rule "work not done goes to its issue", and
      cites #56, #62, #64, #72 for the other cut passages. But the two real
      blind spots the gate leaves open (a capability directory with no
      `spec.md` skipped silently, and a declined promotion that passes) are
      "reported to the `spec-writer`" and nothing more. The issue list holds
      nothing for either (`gh issue list`, all states, 40 newest). When #93
      closes, the only live records are a `ci.yml` comment and
      `docs/OPENSPEC-ARCHIVE.md`, which say the gap exists and do not track
      it. Open an issue for each, or record in design.md that the owner
      declined to.
      **Open, pending the runner (`dev-writer`).** Deferral to an issue is
      right, but the destination does not exist yet, and filing is not mine
      to do, so the box stays open until the runner confirms. What I would
      file, two issues on `fryorcraken/logos-radicle`:
      (1) *"Lint's live-spec validation passes when a capability directory
      has no `spec.md`"*: on `@fission-ai/openspec` 2.1.1, `validate --specs
      --strict` skips a directory under `openspec/specs/` with no `spec.md`
      without a word (renaming `node-seeding/spec.md` to `Spec.md` gave
      "8 passed, 0 failed (8 items)", exit 0), and an empty `openspec/specs/`
      passes too. The question for the issue is whether the gate must prove
      it read every directory, which is a promise about the gate for the
      `spec-writer` to state before anyone builds a guard.
      (2) *"A declined archive promotion passes every gate"*: declining
      `openspec archive`'s merge prompt moves the change folder without
      promoting its delta and leaves every live spec valid, so `Lint` is
      green exactly when the contract is missing; catching it needs a check
      that every capability an archived change's `specs/` names carries that
      change's requirements in `openspec/specs/`.
      Once filed, `design.md` Decision 3 and Risks should name both numbers;
      I added the sentence to Decision 3 saying the existing records note
      the gap and do not track it.
      **Deferred, to issues #99 and #100**, filed on the owner's decision
      ("Blind-spot issues: file them"), in the commit "Close the
      strict-spec-validation findings on the owner's decisions". #99 is
      "Lint's live-spec validation passes when a capability directory has no
      spec.md" and #100 is "A declined archive promotion passes every gate";
      each body states the gap, how it was measured (pointing at `design.md`
      Decision 3 and `docs/OPENSPEC-ARCHIVE.md`) and what a fix could look
      like. `design.md` Decision 3 and both Risks entries now cite them.

- [x] **`dev-writer`** — `design.md:76-83` — the guard list for Decision 1 is
      thin in two places. (1) `setup-node` is listed as a guard, but "which
      Node `ubuntu-latest` ships was not measured", so there is no evidence the
      step turns red without it; say it is unmeasured as a guard rather than
      listing it beside the measured `--strict` entry. (2) Two constants a
      reader might choose differently are not explained: Node 24 where the
      CLI's `engines` floor is 22.20 (the only reason given is "matches
      `ui-specs`"), and `--no-interactive` (what it prevents on a runner is not
      said). Both are suggestions.
      **Fixed** in the commit "Act on the dev-writer review findings in
      strict-spec-validation". (1) Decision 1 now lists `--strict` alone as a
      measured guard, and moves `setup-node` under "Environment pins, not
      measured guards", saying outright that whether the step fails without
      it is unknown. (2) Node 24: the floor would serve equally; 24 is what
      `ui-specs` sets up, so the workflow holds one Node version to bump,
      written in the Decision and in the comment above the step.
      `--no-interactive`: read from the 2.1.1 source, not measured. With
      `--specs` the bulk path's only interactive behaviour is the `ora`
      spinner (`runBulkValidation`, `dist/commands/validate.js`), and
      `isInteractive` already answers no when `CI` is set
      (`dist/utils/interactive.js`), so on a runner it changes nothing; it
      is kept for the reason `OPENSPEC_TELEMETRY` is. Written in the
      Decision and beside the step's `env`. Prose and comments only; no
      test can fail without it.

- [x] **`dev-writer`** — `design.md:176-198` and `radicle-ui/src/qml/SourceState.qml`
      — the `embedded-header` rule is the one entry whose test has not been
      mutated. `tasks.md:71-76` says no named mutation has been run, and the
      new `SourceState.qml` comment says of
      `test_the_identity_is_shown_in_exactly_the_modes_that_derive_local` that
      "that one has not been run under the mutation". That is honest, but the
      design entry carries no mutation evidence for this guard either, and it
      is the change's one new behavioural statement (a mode has an identity
      exactly when it derives `local`). Run the two mutations the test names
      (every mode but Explore; only Local) and record which tests go red, or
      record here that it was not run. Also: `proposal.md:75` says no code in
      `radicle-ui/` changes, and this change edits a doc comment in
      `SourceState.qml`; say so in the proposal's Impact, since the claim is
      read as "no `.qml` touched".
      **Fixed, by recording that it was not run**, in the commit "Act on the
      dev-writer review findings in strict-spec-validation". The mutations
      could not be run here either: this pod has no Qt 6 `qmltestrunner`
      (`which qmltestrunner6` finds none, and `/usr/lib64/qt6/bin/` and
      `/usr/lib/qt6/bin/` hold no `qmltestrunner`). The `embedded-header`
      entry in `design.md` now says "Mutation evidence: none for the new
      test", which tests the named mutation was run against, the predicted
      red cases of both mutations by reading, and that the test is pinned on
      a copy of `Main.qml`'s binding. `tasks.md` group 3 says the same. The
      proposal's Impact now says no behaviour in `radicle/` or `radicle-ui/`
      changes and names the two comment-only source edits, `SourceState.qml`
      and `settings_store.cpp` (the latter from the second box above);
      `git diff --stat origin/main -- radicle/src radicle-ui/src
      radicle/rust-ffi/src` lists exactly those two files. The two
      mutations remain the runner's to run where a Qt 6 runner exists.

- [x] **owner (`.claude/specflow/PROJECT.md`), via `dev-writer` to propose** —
      the trap a spec-writer meets tomorrow is not where the spec-writer reads.
      After this change a requirement whose lead text exceeds 500 characters
      fails `Lint` once archived, and `openspec validate <change> --strict` can
      pass a `MODIFIED` that shrank but stayed long. That is written in
      `CLAUDE.md`'s CI section and `docs/OPENSPEC-ARCHIVE.md`, which is the
      closer's trigger doc ("read when archiving"). The overlay's
      `## spec-writer` section, which a writer reads before its first command,
      has no 500-character rule, and its `## CI gates` row for `Lint` still
      omits the step (`tasks.md:84-86` already notes the second). Propose both
      edits to the owner, in the PR body; the file is not ours to change.
      **Proposed (`dev-writer`); left open for the owner.** PR #98's body
      now carries both proposed edits to `.claude/specflow/PROJECT.md`
      under "Proposed for the owner": a `## spec-writer` rule that a
      requirement's text before its first scenario stays at or under 500
      characters, because a longer one fails `Lint` once archived and a
      `MODIFIED` that shrank but stayed long passes `validate <change>
      --strict`; and the strict validation step added to the `Lint` row of
      `## CI gates`, with its blind spots in the "does not prove" column.
      Nothing under `.claude/` was touched. Accepting or declining is the
      owner's, so the box is theirs to tick.
      **Fixed, overlay edited at the owner's request** ("Overlay edits: the
      owner asks you to make both"), in the commit "Close the
      strict-spec-validation findings on the owner's decisions". In
      `.claude/specflow/PROJECT.md`: `## spec-writer` carries the rule that a
      requirement's text before its first scenario stays at or under 500
      characters, with why (a `MODIFIED` that shrank but stayed long passes
      `validate <change> --strict` and fails `Lint` after archive); the
      `## CI gates` `Lint` row names the `validate --specs --strict` step on
      the pinned CLI, and its "does not prove" column names a capability
      directory with no `spec.md`, a declined archive promotion and an
      in-flight change's delta. Nothing else under `.claude/` changed. The
      PR body's "Proposed for the owner" section now says what was done.
      Prose only; no test can fail without it.
