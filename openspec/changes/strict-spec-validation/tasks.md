## Stages

- [x] spec — `spec-writer`
- [x] design + code — `dev-writer`
- [x] tests — `tester`
- [ ] review: correctness — `code-reviewer`
- [ ] review: security — `code-reviewer`
- [ ] review: readability — `code-reviewer`
- [ ] review: architecture — `code-reviewer`
- [ ] review: spec-test — `spec-test-reviewer`
- [ ] review: design — `design-reviewer`
- [ ] findings all ticked, `findings/` deleted — `closer`
- [ ] `openspec validate --strict`, then `archive` — `closer`
- [ ] CI green, title/body checked, PR merged — `closer`

# Tasks

The validation command below is the one the `Lint` step runs, from the
worktree root: `npx --yes @fission-ai/openspec@<pin> validate --specs --strict
--no-interactive`, with `<pin>` as `ci.yml` writes it.

## 1. The CI gate (design.md, Decisions 1 to 3 and 5)

- [x] 1.1 Append `actions/setup-node` (Node 24) and a strict validation step,
      pinned exactly, to the `lint` job in `.github/workflows/ci.yml`, with a
      comment naming what a green cannot see. Verify: the workflow parses, and
      the `lint` job lists the two new steps last (parsed with the `yaml`
      package the CLI ships); `check-stack-pins.py` still passes.
- [x] 1.2 Verify the command fails on the tree as it stands: 0 passed,
      9 failed, exit 1, and every issue is the length warning (73 of them,
      counted from the `--json` report without `--strict`, which exits 0).
- [x] 1.3 Verify the command passes on the tree the archive will produce:
      trial `openspec archive strict-spec-validation --yes` in this worktree,
      then the command reports 9 passed, 0 failed; then restore with
      `git reset --hard` and `git clean` of the archived folder, and
      `git status` is clean.
- [x] 1.4 Verify the blind spot Decision 3 records rather than guards: on the
      trial-archived tree, renaming `node-seeding/spec.md` to `Spec.md` gives
      "8 passed, 0 failed (8 items)" and no failure.

## 2. Documentation

- [x] 2.1 `CLAUDE.md`'s CI section lists the gate, says it is strict and why,
      and names the pinned command without repeating the version. Verify by
      reading.
- [x] 2.2 `docs/OPENSPEC-ARCHIVE.md` says a declined promotion is not caught,
      lists what the gate cannot see (each measured or read from the CLI's
      `validator.js`), and says a change fixing the live specs through a delta
      is red on `Lint` until its archive commit. Verify by reading.

## 3. A test for each split-out scenario (the `tester`'s stage)

Each `ADDED` requirement carries a scenario no existing requirement had. The
proposal expects no code change; a scenario the code fails is a finding, not a
licence to change behaviour quietly.

- [x] 3.1 `embedded-header`: 1 scenario pinned by a test that can fail.
      The first wording contradicted `source-modes` and could not be pinned;
      the `spec-writer` restated it as the relationship the code has (a mode
      has an identity exactly when it derives the `local` prefix), and
      `test_the_identity_is_shown_in_exactly_the_modes_that_derive_local` pins
      it on `slotIdentity.visible`.
- [x] 3.2 `embedded-identity`: 4 scenarios pinned.
- [x] 3.3 `embedded-setup`: 20 scenarios pinned.
- [x] 3.4 `embedded-state`: 5 scenarios pinned.
- [x] 3.5 `module-settings`: 3 scenarios pinned; the five-keys one closes the
      gap issue #64 records.
- [x] 3.6 `node-config`: 3 scenarios pinned.
- [x] 3.7 `source-modes`: 1 scenario pinned.

"Pinned" here means a test exists that, read against the code, can fail for the
reason the scenario names. **None of the tests added in this group has been
run**: the pod that wrote them has no Qt runner, C/C++ compiler or Nix, so the
first execution is CI's.

## Workflow follow-up

- The PR's first `Lint` run fails at the strict step on the length warning
  alone. That is expected until the `closer`'s archive commit (design.md,
  Migration Plan), and is the evidence the step can fail.
- After the archive commit, `Lint` passes the step with 9 passed.
- `.claude/specflow/PROJECT.md`'s `## CI gates` row for `Lint` does not list
  this step. It is the owner's file; the change proposes the edit rather than
  making it.
