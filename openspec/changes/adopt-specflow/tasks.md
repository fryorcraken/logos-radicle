## Stages

- [ ] ~~spec — `spec-writer`~~ — no behaviour change in either module; the proposal is written and the change declares `skip_specs: true`
- [x] design + code — `dev-writer`
- [ ] ~~tests — `tester`~~ — no spec requirement to test; the new `ci.yml` step is exercised by the PR's own CI run
- [x] review: correctness — `code-reviewer`
- [x] review: security — `code-reviewer`
- [x] review: readability — `code-reviewer`
- [x] review: architecture — `code-reviewer`
- [ ] ~~review: spec-test — `spec-test-reviewer`~~ — no spec delta and no tests for it to read
- [x] review: design — `design-reviewer`
- [ ] findings all ticked, `findings/` deleted — `closer`
- [ ] `openspec validate --strict`, then `archive` — `closer`
- [ ] CI green, title/body checked, PR merged — `closer`

# Tasks

No test layer can see this change: it edits prose, agent instructions and
ignore rules, plus two comments. Each task is verified by the command named in
it, run against this tree.

## 1. Move PLAN.md's durable rules, then delete it

- [x] 1.1 Add "Platform scope" and the real-home and two-home test rules to
  `docs/rust-ffi.md`, worded against `ci.yml`'s `build` matrix and
  `radicle/rust-ffi/Cargo.lock`; verified by reading the matrix legs and
  `git grep -n "uds_windows\|radicle-windows" radicle/rust-ffi/Cargo.lock`
- [x] 1.2 Add the embedded e2e lesson to `docs/e2e.md`, and fold "re-run a spec
  at the next stack bump" into its existing sitometres paragraph; verified by
  reading the section for a second copy of the sentence
- [x] 1.3 Repoint the PLAN.md references in `docs/e2e.md`,
  `radicle-ui/tests/ui/sync.yaml`, `radicle/src/radicle_impl.h` and
  `docs/M3-embedded-node-plan.md`'s header to issue #92 or to the substance
- [x] 1.4 Delete `docs/PLAN.md`; verified by
  `git grep -n "PLAN\.md" -- . ":!openspec/changes/archive" ":!openspec/changes/adopt-specflow"`
  returning nothing

## 2. Move the agent files' repo-specific rules, then delete them

- [x] 2.1 Add `## spec-writer`, `## dev-writer`, `## tester`,
  `## code-reviewer`, `## design-reviewer` and `## spec-test-reviewer` sections
  and the composer hazard to `.claude/specflow/PROJECT.md`, per design.md
  Decision 2; remove the two Hazards lines naming PLAN.md and the project agents
- [x] 2.2 Repoint `docs/OPENSPEC-ARCHIVE.md`'s two `.claude/agents/README.md`
  links to the plugin's skills
- [x] 2.3 Delete `.claude/agents/*.md` and their `.gitignore` re-admits;
  verified by `git check-ignore -v --no-index .claude/agents/code-reviewer.md`
  matching `.claude/*`, and `.claude/settings.json` still re-admitted

## 3. Slim `CLAUDE.md`

- [x] 3.1 Remove the PLAN.md, `RUNNER.md` and `README.md` rows and repoint the
  flow paragraph, per design.md Decision 3
- [x] 3.2 Cut the restated rules from "Keeping this file true", the Bash-cost
  section, the scratch section, "Working in a git worktree", "How to shape a
  change" and "Tests are part of the change", keeping this repo's evidence
- [x] 3.3 Fix the stale claims in the sections touched, per design.md
  Decision 4; verified by
  `git grep -n -E "\.claude/agents|RUNNER\.md|flow README" -- CLAUDE.md`
  returning only the specflow block's generic line

## 4. Integration checks

- [x] 4.1 `openspec validate adopt-specflow` passes
- [x] 4.2 No live file outside this change names a deleted file; verified by
  `git grep -n -E "PLAN\.md|\.claude/agents|RUNNER\.md" -- . ":!openspec/changes/archive" ":!openspec/changes/adopt-specflow"`
  returning only `CLAUDE.md`'s specflow block line

## Deferred

- `openspec validate --specs` in `ci.yml` is not in this piece. It is issue
  #93; design.md Decision 6 records why.
