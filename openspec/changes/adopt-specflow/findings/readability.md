# Readability findings: adopt-specflow

Dimension reviewed: **readability** only (the prompt named it). Material: prose,
agent instructions, config, ignore rules and two comments; there is no
application-code diff, so the mutation tool does not apply and was not run.

**Clean, checked against the tree:**

- `git grep` for `PLAN.md`, `.claude/agents`, `RUNNER.md` and `flow README`
  outside `openspec/changes/archive/` and this change returns only the specflow
  block's generic line (`CLAUDE.md:853`), as tasks 3.3 and 4.2 claim.
- Every issue number the new prose cites (#28, #29, #59, #72, #75, #77, #79, #80,
  #82, #92, #93) exists and says what the citing sentence says. #92 carries the
  116 s measurement that `docs/e2e.md` and `sync.yaml` now defer to.
- `docs/rust-ffi.md` "Platform scope": the three `build` matrix legs, the
  `release` job's three-variant loops and `uds_windows` / `radicle-windows` in
  `Cargo.lock` all match the code. "Creating an identity" and "the Darwin
  paragraph" resolve. The real-home / probe / two-home paragraphs read cleanly
  and say why, not only what.
- `docs/M3-embedded-node-plan.md`'s header, `radicle_impl.h:551` and
  `sync.yaml`'s comment are repointed to the substance, and read well alone.
- `docs/OPENSPEC-ARCHIVE.md`'s two repoints name skills that exist and say which
  half of the job each holds.
- The overlay's per-role sections are short, each rule says why, and the
  decision table in `design.md` Decision 2 maps them one to one.
- The stage block in `tasks.md` is well formed: struck rows keep their box and
  carry a reason. `design.md` follows the Decisions / Considered / Ruled-out
  shape.
- CLAUDE.md's slimmed sections keep this repo's evidence and point at the block
  for the rules; no section was found that still restates a block rule at length.

- [x] **`spec-writer`** — `openspec/changes/adopt-specflow/proposal.md:49-51, 66-82, 87-90` — the proposal still describes a CI step that the change does not make
      **Scenario:** a reader who opens `proposal.md` first (it is the first file in
      the archive) is told "Add `openspec validate --specs --strict` to `ci.yml`"
      under What Changes, sees `ci.yml` in Impact's "Files edited", and sees open
      question 2 as undecided ("Not decided here"). `design.md` Decision 6 and
      `tasks.md` "Deferred" say the opposite: the owner moved the step to #93 and
      nothing in `ci.yml` changes. `design.md`'s Non-Goals also names it out of
      scope. The same Impact list omits `docs/OPENSPEC-ARCHIVE.md` and
      `.gitignore`, both of which this change edits (`design.md` Decisions 2 and
      5). Open question 1 is recorded as answered, which is the one place the
      proposal was brought up to date.
      **Severity:** genuine defect (documentation that contradicts the two files
      beside it), medium. The proposal is archived as the record of why; it will
      be wrong for every later reader.
      **Outcome (`spec-writer`): fixed.** What Changes no longer lists the CI
      step and says it is issue #93; Open question 2 records the owner's answer;
      Impact lists `ci.yml` as not edited and now matches
      `git diff origin/main...HEAD --name-status`: `.claude/specflow/PROJECT.md`
      as added, and `.claude/settings.json`, `.gitignore` and
      `docs/OPENSPEC-ARCHIVE.md` among the edited files. What Changes also gains
      the `docs/OPENSPEC-ARCHIVE.md` repoint.

- [ ] **`dev-writer`** — `.gitignore:62-66` — the new `# specflow` block re-admits whole directories with no explanation, and contradicts the comment 25 lines above it
      **Scenario:** the header comment (lines 16-37) says `.claude/*` holds
      "`lgs init`'s output, `openspec init`'s skills and commands" and advises
      "re-admit files by name where you can, and a directory only when
      everything in it is meant to be tracked". Lines 63-66 then re-admit
      `.claude/skills/`, `.claude/commands/` and `.claude/hooks/` as directories,
      under a bare `# specflow` label, after an unrelated `tmp/`. `CLAUDE.md:446`
      says `lgs init` writes into `.claude/skills/`. So the file now tells a
      reader the directory is generated and ignored, and also unignores it.
      Today the only generated skill is kept out by accident:
      `git check-ignore -v --no-index .claude/skills/basecamp/SKILL.md` reports
      `.gitignore:55:basecamp/` (the unrelated build-output rule), not the intent.
      A skill whose directory is named anything else would be tracked silently,
      which is the failure the header comment spends fifteen lines warning about.
      **Severity:** genuine defect in a comment/ignore pair, low to medium. Fix is
      to say why these four directories are re-admitted (they are the plugin's
      and the owner's), or to re-admit by name, and to reconcile the header
      comment. The block's position is forced (negations must follow `.claude/*`)
      but the comment can say so.

- [ ] **`dev-writer`** — `.claude/specflow/PROJECT.md:169-172` (and `CLAUDE.md:547-559`) — the staleness-guard hazard counts four copies and says a fourth is the signal to reshape; #29 says there are nine sites and recommends against one abstraction
      **Scenario:** the overlay is what every dispatched agent reads first. It
      says the guard "is hand-written in `CommitsTab`, `IssuesTab`, `PatchesTab`
      and `ThreadView`" and "A fourth variant is the signal to reshape, not to add
      a test." Issue #29 counts nine sites with seven distinct term sets (it lists
      `CommitView`, `SourceTab` four times and `RepoList` as well), says the terms
      genuinely differ, and says the reviewer's recommendation was "explicitly not
      to build a universal guard". A reviewer following the overlay would report
      every new guard as a missed reshape, against the owner's own issue.
      `CLAUDE.md` carries the same four-name list, which predates this change; the
      overlay's copy is new text and the place to name the count by pointing at
      #29, which is also what the file's own "do not write down what a command can
      answer" rule asks for.
      **Severity:** genuine defect (a stale claim in a new file), low.

- [ ] **`dev-writer`** — `CLAUDE.md:446-448`, `docs/e2e.md:84`, `docs/e2e.md:243` — three paragraphs edited by this change were left badly wrapped
      **Scenario:** each is hard-wrapped prose where the edit changed line length
      without reflowing. `CLAUDE.md:446-448` reads "`lgs init` generates
      `.claude/skills/`, `.cursor/` and `AGENTS.md`, and none of\nit refreshes
      when you upgrade `lgs`. So the\ncopy on disk…", two short lines mid-
      paragraph. `docs/e2e.md:84` is one line of about 110 characters ("…are issue
      #92. The opposite bound is `sync.yaml`'s too: its cancel half") in a file
      wrapped at 80. `docs/e2e.md:243` leaves "The only sign of this" alone before
      a break. A later edit to any of them produces a diff of the whole paragraph.
      **Severity:** stylistic preference, low.

- [ ] **`dev-writer`** — `docs/rust-ffi.md:409-412` — "which is why there is no `target_os` gate" is a non sequitur
      **Scenario:** the sentence reads "A Unix-only assumption in this crate would
      be this repo's own, which is why there is no `target_os` gate or `cfg(unix)`
      branch here … and none should be added without a reason." The absence of a
      gate is a fact the Darwin paragraph at line 477 already records; it is not
      caused by the preceding clause, and "which is why" asserts a motive nobody
      recorded. A reader will take it as the reason the crate has no gates, then
      look for where that was decided. Splitting it ("There is no gate or `cfg`
      branch today (see the Darwin paragraph); adding one is a decision") keeps
      the rule and drops the false causal link.
      **Severity:** stylistic, low.

- [ ] **`dev-writer`** — `CLAUDE.md:145-147` — claims `sh radicle-ui/tests/<script>` is a free, allow-listed call, while the specflow block in the same file says `sh <relative-path>` costs a click
      **Scenario:** the block's table (`CLAUDE.md:875`) lists `sh <relative-path>`
      in the "Costs a click every time" column. The repo bullet and
      `.claude/specflow/PROJECT.md` Test layers both say the relative form is one
      plain allow-listed call, and `.claude/settings.json` does carry
      `Bash(sh radicle-ui/tests/*)`. `origin/main`'s CLAUDE.md said the opposite
      ("the test scripts, by absolute path" free, `sh <relative-path>` a click).
      Nothing in the change records that the relative form was observed not to
      prompt, so one of the two statements in the file is wrong and a reader
      cannot tell which.
      **Severity:** unverified claim, low. Measure it once, then say which form to
      use, or hedge it ("allow-listed as written; not observed").

- [ ] **`dev-writer`** — `CLAUDE.md:663` — the test-layers table gives the core unit-test command as `cd radicle && nix build …` with no note, in a file whose block says never to chain
      **Scenario:** a reader copying the row gets a compound command that prompts,
      and the overlay (`PROJECT.md:30`) gives the cd-free `nix build
      ./radicle#checks…` for the same layer. `design.md` Decision 4 records why the
      row keeps the `cd` form (the cd-free form "has not been run in this repo"),
      but the table row itself carries no such note, so the two files disagree on
      the page and agree only in the archived design.
      **Severity:** stylistic / consistency, low. One clause in the row, or a
      pointer to the overlay's row, closes it.

- [ ] **`dev-writer`** — `docs/OPENSPEC-ARCHIVE.md:141` — cites the allow rule as `Bash(openspec:*)`, which is not in `.claude/settings.json`
      **Scenario:** this change added `.claude/settings.json`'s allow list, which
      spells it `Bash(openspec *)`. The page, rewritten two lines above this one
      in the same commit, still quotes the older colon form, so a reader grepping
      the settings for the rule the page names finds nothing.
      **Severity:** stale reference, low.

- [ ] **`dev-writer`** — `.claude/specflow/PROJECT.md` `## code-reviewer` — a deleted reviewer rule was dropped without being moved or mentioned in `design.md` Decision 2
      **Scenario:** `origin/main:.claude/agents/code-reviewer.md` carried "**An
      operation whose failure is legitimately not an error.** Anything with a
      'best effort' step deserves a look at whether its non-happening is
      observable at all." Decision 2 says each of the nine files "was read for
      content the overlay does not already carry", and its table lists what moved;
      this rule is in neither the table nor the overlay, and `grep -i "best.effort"`
      over the overlay and `CLAUDE.md` returns nothing. It is the repo-specific
      cousin of "Basecamp swallows QML errors" (a failure that produces no
      message), which is the hazard the overlay leads with. The same file's "A
      value that looks obviously invalid and is not. Probe rather than assume."
      (`tester.md`) is also gone and unlisted.
      **Severity:** content loss, low. Either move the rule into
      `## code-reviewer` or add a row to Decision 2 saying it was judged
      the plugin's.
