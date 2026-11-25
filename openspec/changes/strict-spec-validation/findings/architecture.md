# Architecture review: strict-spec-validation

One `code-reviewer` instance covering all four dimensions; this is the
architecture file. Judged against the specflow block's "How to shape a change"
in `CLAUDE.md`. Nothing here was run against Qt, a C++ compiler or Nix (none in
this pod).

## Clean areas

- **Shape of the change.** The source change is one workflow step, two doc
  edits and one comment; there is no reshape tangled with a behaviour change.
  The 37 split-out requirements are specs and tests only, and the change
  respects its own rule that a scenario the code fails is a finding, not a quiet
  code edit: no source behaviour moved.
- **Where the gate lives.** In `Lint`, which is already a required check on
  `main`, rather than a new job that would gate nothing until branch protection
  named it. It needs no Nix and no Qt, so it belongs with the other fast
  structural checks. The pin is written once, in the one place that runs it;
  there is no second copy for `check-stack-pins.py` to reconcile, and
  `CLAUDE.md` deliberately repeats the command without the version.
- **The expected red is real and bounded.** `Lint` fails on this PR until the
  archive commit, because the gate reads `openspec/specs/` as the branch holds
  it. I reproduced both halves: 0 passed / 9 failed before, 9 passed / 0 failed
  after a trial archive. `docs/OPENSPEC-ARCHIVE.md` says so, so the next piece
  that fixes live specs through a delta will not read it as a CI fault.
- **A moved gate measuring the wrong directory:** none. Nothing was moved or
  renamed; the new step reads `openspec/specs/` from the repo root, which is
  where `openspec` takes its root (the step comment says so).
- **Interfaces.** No change to `radicle_impl.h`, the JSON contracts, the FFI
  boundary, or the write path; the tests drive existing public entry points
  (`nodeconfig::get/set`, the module's `getSettings`/`getEmbeddedIdentity`, the
  QML components). No new dependency in the product; the only new third-party
  code is the CI-time CLI (see `security.md`).
- **Stale overlay.** `.claude/specflow/PROJECT.md`'s `## CI gates` row for
  `Lint` still lists only the old checks. It is the owner's file, and
  `tasks.md`'s follow-up already records it as a proposal. Noted so the closer
  does not mistake it for an oversight; not a finding against the code.

## Findings

- [ ] **`tester`** — `radicle-ui/tests/tst_setup_wizard_view.qml:968-976` — `capabilitiesWith()` is a second, drifting copy of the fake's `getCapabilities` reply
      **Scenario:** the file's `fake.capabilities()` (line 59) builds the reply
      from `fake.mode`, `fake.startableModes` and `fake.nodeId`. The new
      `capabilitiesWith(extra)` rebuilds the same object with `mode: "embedded"`
      and the three-mode startable set written as literals, then overlays
      `extra`. Three tests use it. A test that sets `fake.mode = "local"` and
      then reaches for `capabilitiesWith({ pathsProblem: ... })` silently
      reports `embedded`; and when `getCapabilities()` gains a field the fake is
      updated and the helper is not, so the wizard under test sees a reply the
      real backend no longer sends. The wizard-flow file solved the same need by
      giving the fake a scenario property (`capabilitiesMode`), which keeps one
      shape in one place. The cleaner shape is the same here: let `fake` carry
      `pathsProblem`, `gitProblem` and an optional `startableModes` override and
      drop the helper.
      **Severity:** low; a maintainability preference, not a present defect (no
      test is wrong today, and CI passed the three that use it).
