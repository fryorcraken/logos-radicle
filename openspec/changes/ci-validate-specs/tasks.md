## Stages

- [ ] ~~spec — `spec-writer`~~ — no behaviour contract moves: a CI-only
      change, so `skip_specs: true` in `.openspec.yaml`. The proposal is
      written.
- [x] design + code — `dev-writer`
- [ ] tests — `tester`
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

Every "verify" below is a run of `python3 .github/scripts/validate-specs.py`
from the worktree, with the tree restored afterwards (`git status` clean
apart from this change's files). The script is what the CI step runs, so these
runs exercise the check itself; what they cannot exercise is the `ci.yml`
wiring, which group 2 covers.

## 1. The validation script (design.md, Decisions 2 and 3)

- [x] 1.1 Add `.github/scripts/validate-specs.py`, running the pinned CLI with
      `--json` from the repository root. Verify on the unmodified tree: exit 0,
      every long-requirement warning printed, and "all 9 live specs valid".
- [x] 1.2 Verify a structural error fails it: insert a `### Requirement:`
      heading in `openspec/specs/embedded-header/spec.md` above the second
      requirement's scenarios, leaving that requirement with none. Exit 1, the
      spec reported `INVALID`, and an `::error file=…/embedded-header/spec.md::`
      annotation naming `requirements.1.scenarios`.
- [x] 1.3 Verify a skipped capability fails it:
      `git mv openspec/specs/node-seeding/spec.md …/Spec.md` exits 1 naming
      `node-seeding/`, where the CLI alone exits 0 having validated 8. With the
      `expected - seen` loop disabled the same run exits 0, so the loop is the
      guard.
- [x] 1.4 Verify the other two failure paths: `openspec/specs/` moved away
      exits 1 on "no capability directories"; `OPENSPEC` set to a version that
      does not exist exits 1 on "printed no JSON report", with npm's `ETARGET`
      forwarded above it.

## 2. The CI step (design.md, Decision 4)

- [ ] 2.1 Append `actions/setup-node` (Node 24) and a step running the script
      to the `lint` job in `.github/workflows/ci.yml`, with a comment naming
      what it cannot see. Verify: the PR's `Lint` job runs the step, and its
      log ends in "all 9 live specs valid".

The step has **not** been seen red in CI. Tasks 1.2 to 1.4 show the script
failing; the wiring is a plain `run:` whose exit status Actions propagates, and
pushing a deliberately broken spec to the PR would cost a full run of both
workflows to show that. If a reviewer wants it seen, that is the experiment.

## 3. Documentation

- [x] 3.1 Add the gate to `CLAUDE.md`'s "The gates, all on every PR" table,
      naming what it is blind to, and to the CI section's list of what
      `ci.yml` runs, with the local command. Verify by reading both.
- [x] 3.2 Say where the declined promotion is discussed that CI cannot catch
      it: `docs/OPENSPEC-ARCHIVE.md`, "What archive does", and
      `.claude/agents/closer.md`, "Take the delta-merge prompt".
- [x] 3.3 Look for any passage saying spec validation happens only by hand,
      which the proposal's Impact asks to correct.
      `git grep -n -i "validate" -- .claude/agents docs CLAUDE.md` finds none:
      every mention is the closer's `validate <change> --strict`, which stays
      a manual step and is unchanged.
