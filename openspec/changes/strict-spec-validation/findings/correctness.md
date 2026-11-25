# Correctness review: strict-spec-validation

Reviewed by one `code-reviewer` instance covering all four dimensions
(correctness, security, readability, architecture); this is the correctness
file. Scope: the `Lint` step in `.github/workflows/ci.yml`, the doc edits, the
`SourceState.qml` comment, and the new QML, Rust and C++ tests.

## What I ran, and what I could not

- **Ran, against this tree:** the pinned CLI (`@fission-ai/openspec@2.1.1`,
  installed with `--ignore-scripts` into `./tmp/`) as the step runs it.
  `validate --specs --strict --no-interactive` exits 1 with `0 passed, 9
  failed`, every issue the 500-character warning. A trial `openspec archive
  strict-spec-validation --yes` followed by the same command gives `9 passed, 0
  failed`; the tree was then restored with `git reset --hard` and a `git clean`
  of the archived folder. `openspec validate strict-spec-validation --strict` is
  valid. `cargo fmt --check` on `radicle/rust-ffi` is clean.
- **Read from the CLI's own source** to check the claims in
  `docs/OPENSPEC-ARCHIVE.md`: the length rule measures requirement text before
  the first `####` (`requirement-text.js`), the `MODIFIED` rule fires only when
  the new text is over the limit **and** longer than the live text
  (`validator.js` lines 680-682), and `OPENSPEC_TELEMETRY` is honoured by
  `telemetry/opt-out.js`, which its header says the version check shares. All
  three claims hold.
- **Could not run:** any QML test (no Qt 6 runner), any C++ test (no compiler,
  no Nix), the Rust tests (`cargo test` stops at "linker `cc` not found"), and
  clippy. **No mutation was run on any file.** Every statement below about what
  a test can or cannot catch is from reading, not from a red/green. The
  mutations the test comments name (`tst_source.qml`'s "every mode except
  Explore" and "only Local") I traced by hand against `SourceState.qml` and they
  do separate as the comment says, but that is an argument, not a run.

## Clean areas

- **The workflow step.** Pinned exactly, `--strict` and `--no-interactive`
  present and accepted by the CLI, `OPENSPEC_TELEMETRY: "0"` is a value the CLI
  reads as disabled, the Node version satisfies the CLI's `engines`
  (`>=22.20.0`) and matches the `ui-specs` job, and it is the last step of
  `Lint`, after `setup-node`. The step does fail on the tree as it stands and
  pass on the archived one, so a green `Lint` after the archive commit is
  evidence, and today's red is the expected one.
- **The doc claims** in `CLAUDE.md` and `docs/OPENSPEC-ARCHIVE.md` that I could
  check against the CLI source or by running it.
- **The `SourceState.qml` comment against the code**: `modeHasIdentity` is
  `current === "local"`, so "a mode has an identity exactly when it derives the
  `local` prefix" is what the code does, and "a fourth mode gains an identity in
  the same edit that adds it to `current`, and routes to the seed with none until
  then" is true of it.
- **The Rust tests** (`node_config.rs`, `profile_init.rs`): each leg moves one
  input against a control, the non-default seeding in
  `two_fields_set_at_once...` is what makes "the rest hold their values" able to
  fail, and `keys_snapshot` plus the precondition that both halves of the key
  are present means two empty maps cannot compare equal. `assert!(profile_exists
  ...)` sits before the second snapshot, so both probes are inside the window.
- **`ETXTBSY`:** none of the new C++ or Rust tests writes an executable and runs
  it, so that trap has nothing to bite on here. The unreadable-key test
  replaces `keys/radicle` with a directory rather than using `chmod`, so it does
  not depend on not running as root.
- **QML fakes that return the same for every input:** the new tests mostly pair
  each leg with a control that differs in one field (the paths problem sentence,
  `embeddedStartHosted`, `embeddedEncrypted`, `identityExists`), and the identity
  test gives every case a different DID. I found no new fake that answers
  identically for every input. The three entries below are the exceptions.

## Findings

- [ ] **`tester`** — `radicle-ui/tests/tst_setup_wizard.qml:1786` and `:1846` — the new "no `startNode` may be issued" call-log scans can never fail
      **Scenario:** the harness's `fake` has no start function by design (the
      comment at lines 259-268 says the `startNode` entries are "unreachable
      rather than merely absent"), so no call a flow makes can ever be logged as
      `startNode`. The loops added in
      `test_an_existing_identity_with_a_stopped_node_is_walked_without_a_start`
      and `test_a_failing_preflight_writes_nothing_either` therefore pass for
      any flow whatever. The first test's comment says the precondition
      `fake.callLog.length > 0` makes this "a search of something rather than of
      an empty list", which is true of the list and false of the thing searched
      for. What does pin the scenario is `compare(typeof flow.startNode,
      "undefined")` in the first test, and that holds only for a property with
      exactly that name: a flow that gained a differently named start entry
      point (`startEmbeddedNode`, an injected `fetchStart`) passes both tests.
      The `createEmbeddedIdentity` and `setSetting` scans beside them in the
      second test are real.
      **Severity:** low-medium; the scenario is pinned by one name-bound check
      and presented as pinned by two. **Measured:** not run (no Qt runner); the
      argument is from the fake's source.

- [ ] **`tester`** — `radicle/tests/test_radicle_impl.cpp:625-640` and `radicle/tests/test_settings_store.cpp:355-372` — the "before any write" leg of the two five-keys tests is only fresh on the first run
      **Scenario:** `scratchSettingsPath("five-keys")` and
      `ScratchSettings s("five-keys")` give a fixed directory under `TMPDIR`
      that nothing removes, and these tests write all five keys into it. On a
      second local run (same `TMPDIR`) the opening
      `getSettings().size() == 5` and `store.all().size() == 5` read a file that
      already holds `mode:"local"`, a radHome and a seed, not "a file that has
      never been written", which is the half the comment and the scenario claim.
      A defect that added a sixth key only on the no-file default path would be
      caught on the first run and missed on every later one. CI starts clean so
      it is not affected; a developer's rerun is. The neighbouring
      `get_settings_reports_defaults_when_nothing_has_been_persisted` does not
      have the problem only because nothing else writes its `"defaults"`
      directory. `ScopedXdgDataHome` in the same file already removes its
      directory for exactly this reason.
      **Severity:** low (local-rerun hygiene). **Measured:** not run.

- [ ] **`tester`** — `radicle-ui/tests/tst_embedded_panel.qml:1154-1164` — `test_the_passphrase_field_does_not_show_what_is_typed` restores the field only on the passing path
      **Scenario:** the control leg sets `f.echoMode = TextInput.Normal` on the
      one `TextField` shared by every test in the file. If a `verify` before the
      restore fails (the masked walk finds the secret, or the control walk does
      not), QtTest aborts the function and lines 1163-1164 never run: the field
      is left unmasked and holding `"correct horse battery"`. `init()` does not
      reset it, so any test that runs after it and asserts the mask
      (`test_an_encrypted_identity_is_offered_a_passphrase_field`, line 880,
      `verify(f.echoMode !== TextInput.Normal)`) fails too, with a message about
      the wrong test. It costs diagnosis time only when something has already
      broken. A restore in `init()` or `cleanup()` removes it.
      **Severity:** low (failure-time noise). **Measured:** not run.
