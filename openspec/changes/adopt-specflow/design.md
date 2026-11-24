# Design

## Context

See `proposal.md` for why. The piece branch already carries the overlay
(`.claude/specflow/PROJECT.md`), the specflow block at the end of `CLAUDE.md`,
the plugin entries in `.claude/settings.json` and the `.gitignore` re-admits
for `.claude/specflow/`. What it does not yet do is remove the second copy of
everything those now say: the nine files under `.claude/agents/`, the
restatements in `CLAUDE.md` above the specflow block, and `docs/PLAN.md`.

**The settings permissions are recorded here, not decided here.** The allow
list's first twenty entries and the nine `gh api` deny patterns are
`/specflow:init`'s list verbatim (its step 1, which itself notes that
`Bash(git *)` covers `git push` and `git reset` and names branch protection on
the default branch as the backstop); the remaining entries (`lgs`, `nix build`,
the Qt 6 runners, the test scripts, `cargo`) are the stack commands init's
report asks the owner to add. Review raised narrowing `git *`, dropping
`grep *` in favour of the `Grep` tool, naming the test scripts instead of
`sh radicle-ui/tests/*`, and pinning the marketplace source. The file is the
owner's, so each is left with the owner, and the findings record the options.

Two constraints shape every step:

- **A file deleted here takes its links with it.** `git grep` over the tree,
  excluding `openspec/changes/archive/`, is the inventory of what points at
  `docs/PLAN.md` or `.claude/agents/`; every hit outside this change's own
  folder has to be repointed or removed in the same commit as the deletion.
- **A durable rule must land before its source is deleted.** The deleted
  files carry rules nobody else records, mixed in with rules the plugin now
  owns. Each one is either moved, or shown to be held elsewhere already.

## Goals / Non-Goals

**Goals:**

- One copy of each flow rule: the plugin's. The overlay carries only what is
  specific to this repository.
- No live document names `docs/PLAN.md` or a file under `.claude/agents/`.
- Every PLAN.md passage with a future either has an issue or a durable home,
  except the two deferrals Decision 1 marks as awaiting the owner.

**Non-Goals:**

- **Changing anything either module does.** The two source-file edits are
  comments.
- **Rewriting the archived changes.** `openspec/changes/archive/` keeps its
  references to PLAN.md and the agent files: an archived change records what
  was true when it shipped, and rewriting it would make that record false.
- **The CI validation step.** Deferred; see Decision 6.
- **Editing the specflow block in `CLAUDE.md`.** It is written by
  `/specflow:sync`. Its one mention of `.claude/agents/` is a generic statement
  that stays true.

## Decisions

### 1. Where each PLAN.md section went

The issue's migration list, the owner's answer on the issue for the two
sections that list missed, and a survey of the rest. Nothing below is copied
into two places.

| PLAN.md passage | Where it lives now |
|---|---|
| "Windows and macOS are out of scope…" | `docs/rust-ffi.md`, "Platform scope", reworded (below) |
| "Never point a test at the developer's real Radicle home" | `docs/rust-ffi.md`, "Testing this crate" |
| Two-home isolation ("Isolation is the test that matters most") | `docs/rust-ffi.md`, "Testing this crate" |
| "An embedded node is easier to e2e…" | `docs/e2e.md`, beside `local.yaml`'s `RAD_HOME` section, without "may finally let local browsing into CI" |
| "The Logos stack", last sentence | `docs/e2e.md`'s existing sitometres-moves-with-Basecamp paragraph, tied to the stack bump |
| "The Logos stack", the rest | dropped: `CLAUDE.md`'s scaffold section already says the pins move as one set and a bump starts from a scaffold release |
| "Known defects in shipped behaviour" | issue #92; `docs/e2e.md` and `sync.yaml` now point there |
| The allow-is-not-enough gotcha | `radicle_impl.h`'s `seedRepo` comment, which already states it in full, now names it rather than PLAN.md; the `node-seeding` spec |
| "no seeds found" → `--seed <NID>` | issue #75, and `docs/M3-embedded-node-plan.md` |
| Stale gossiped addresses → restart | issue #72, and `docs/M3-embedded-node-plan.md` |
| "Hosting an action is one property plus a branch" | issue #72's acceptance criteria, the archived `embedded-node-wizard` `design.md`, and `test_every_hosted_kind_is_one_the_host_routes` |
| Header caption | issue #76 |
| The configuration panel and node control | issues #68 to #74 |
| Phase 3, writes against the embedded node | issue #82 |
| Whether `git` is in a shipped bundle | issue #80 |
| The unscoped control socket (the paragraph that ends mid-sentence) | issue #79 |
| Embedded e2e as a spec to write | issue #77 |
| "Deferred, deliberately: patch open/update … and delegate/identity management" | **no issue; awaiting the owner**, who decides whether each gets an issue or stays recorded only in `docs/M3-embedded-node-plan.md`'s "Deferred" line under Phase 3, which carries the same deferral and its reason (the `git` push path must be proven first). Issue #86 names delegate management as out of its scope; no issue tracks either item |
| "What exists today", the struck-through shipped items | dropped: the specs and the archived `design.md`s they already pointed at hold them |

**The platform wording is corrected, not moved verbatim.** PLAN.md said "this
repo has only ever built and tested Linux". That stopped being true when
`ci.yml`'s `build` matrix gained `linux-arm64` and `darwin-arm64` legs, each
built natively, and the `release` job began merging all three into one
`.lgx`. What *is* true, and what the new section says: three platforms are
built, every test job runs on `ubuntu-latest` (the Rust job, the QML job, and
the core unit tests on the `build` matrix's linux-amd64 leg only), and Windows
is never built. The point kept from PLAN.md is the one the issue named:
`radicle-node` carries `uds_windows` and `radicle-windows` (both are in
`radicle/rust-ffi/Cargo.lock`), so a Unix-only assumption here would be this
repo's own, not the crate's.

**The sitometres sentence is folded into the paragraph that already says it**,
rather than added beside it. `docs/e2e.md` already read "When the Basecamp pin
moves, re-run a spec on the current sitometres". The owner's sentence adds when
that is: at the next stack bump, since the Basecamp pin moves only as part of
the set. A second paragraph saying nearly the same thing would be the two
copies this change exists to remove.

### 2. The agent files are deleted, and only their repo-specific rules move

Each of the nine files under `.claude/agents/` was read for content the
overlay does not already carry. Most of it is the plugin's, restated: the
branch table, the stage block, the findings format, the push sequence, the
`EnterWorktree` probes. What is this repository's moved into the overlay as
per-role sections, because the overlay is where the plugin tells each role to
look (`## <role>`), and a rule addressed to every role belongs in `## Hazards`:

| From | To |
|---|---|
| `spec-writer.md`: MUST / MUST NOT only, a definition is not a requirement | `## spec-writer` |
| `spec-test-reviewer.md`: `remote*`/`local*` contract symmetry | `## spec-writer` and `## spec-test-reviewer` |
| `spec-test-reviewer.md`: spend the mutation budget on the Rust and QML layers before the C++ core | `## spec-test-reviewer` |
| `dev-writer.md`: one failure shape, `{"error":"..."}` | `## dev-writer` |
| `design-reviewer.md`: the triggers that make a `design.md` due here (the FFI boundary, the JSON contract, the write path) | `## dev-writer` and `## design-reviewer` |
| `design-reviewer.md`: check against `docs/M3-*.md` where a change touches what they measured; a built subsystem's trap belongs in its trigger doc | `## design-reviewer` |
| `tester.md`: `ETXTBSY` from a test that writes then runs a binary; a resolver test needs a nonexistent path; a value that looks invalid may not be, so probe it | `## tester` |
| `code-reviewer.md`: what input is untrusted, the boundaries | `## code-reviewer` |
| `code-reviewer.md`: the write-gate check (`canWriteLocal`, never `localAvailable`) | `## Hazards`, which already carried it |
| `code-reviewer.md`: the seed API's observed shapes; the lock and the `flake.nix` vendor hash move together; the licence; an operation whose failure is legitimately not an error | `## code-reviewer` |
| `closer.md`: read `docs/OPENSPEC-ARCHIVE.md` before archiving | `## closer` |
| the composer that must reload rather than trust its own copy (in `code-reviewer.md`, `tester.md`, `spec-test-reviewer.md`) | `## Hazards`, once |

Rules already in the overlay were not repeated: the silent-QML-failure
hazards, `guarded()`, the staleness guard, `canWriteLocal`, the binding
inside its own handler, the single-file QML runner, `cargo mutants`' blind
spot, the `ui-tests.yml` matrix.

**Considered: keeping the files, trimmed to their project-specific halves.**
Ruled out because a project agent does not replace the plugin's agent of the
same name; both are offered, the unqualified one wins by being shorter to
address, and it then carries none of the plugin's rules. The overlay's last
hazard line existed only to warn about that collision, and goes with the files.

**The closer's route to `docs/OPENSPEC-ARCHIVE.md` is the overlay.** The
deleted `closer.md` told the closer to read that page before archiving. The
plugin's closer reads the overlay and its own `specflow:openspec-archive`
skill; whether it also reads `CLAUDE.md`'s trigger table is up to the harness.
So the pointer moves into a `## closer` section rather than relying on the
table row.

**`.gitignore`'s re-admits for `.claude/agents/` go too.** Left in, they would
silently make any file dropped there trackable, which the comment above them
argued against at length. The settings re-admit and its reasoning stay. A
project that later adds a role agent (the plugin allows it, through the
overlay's `## Extra stages`) re-admits that one file by name. This departs from
`/specflow:init`'s list, which includes `!.claude/agents/`: a later run of init
appends that line again, and the departure is worth re-making then.

**The specflow block's other directory re-admits stay, and the generated names
are re-ignored after them.** `!.claude/skills/`, `!.claude/commands/` and
`!.claude/hooks/` are init's, for a stated reason: a project skill or hook must
be tracked to reach a worktree cut from `HEAD`. Narrowing them to files that
exist today would undo that for the next project skill, and init would append
them again. What they cost is the hazard the `.gitignore` comment describes:
`lgs init`'s bundled skill under `.claude/skills/basecamp/` was kept out only by
the unrelated `basecamp/` build-output rule, and a `settings.local.json` under
any of them was trackable. Both are now re-ignored by name after the block, the
comment says why the block is the exception to its own advice, and the
overlay's `## Never commit` names what else may appear there. Verified with
`git check-ignore -v --no-index`: `.claude/skills/basecamp/SKILL.md` and
`.claude/{skills,specflow,hooks}/settings.local.json` match the new lines, and
`.claude/specflow/PROJECT.md` and a project skill path match no ignore rule.
`openspec init` 2.1.1 with `--tools claude` writes its skills to the user's
own `~/.claude`, not the project's (measured: it failed creating
`~/.claude/skills/openspec-update-change`), so there is no openspec name to
re-ignore today.

### 3. `CLAUDE.md` keeps this repo's evidence, not the rules the block restates

Where a section above the specflow block restated a rule the block now
carries, the rule text goes and this repo's own evidence for it stays,
because the evidence is what a command cannot tell you and the block does not
hold it.

| Section | Outcome |
|---|---|
| "Where to look for what" | PLAN.md, `RUNNER.md` and `README.md` rows removed; the flow paragraph points at the specflow block and names the issue tracker as where scope lives |
| "Keeping this file true" | the history and the rejected status file stay; the general rules point at the block |
| "How to work in this repo, and what Bash costs" | the `lgs` paragraph, what `.claude/settings.json` allow-lists and why it is tracked, the two repo-specific traps (`/nix/store`, `curl` against crates.io) and the no-argument test scripts stay; the table and the generic bullets point at the block |
| "Scratch files go in `./tmp/`" | removed: the block's section of the same name says the same |
| "Working in a git worktree" | the measurements stay as evidence; the rules point at the block and at `specflow:worktree-discipline` |
| "How to shape a change" | the worked examples stay (`NavState.qml`, the staleness guard, `syncAll()`, `guarded()`, the narrow interface); the rules point at the block |
| "Tests are part of the change" | shortened to this repo's reading of the block's rule, and retitled "What 'tests are part of the change' means here" so it no longer shares a heading with the block's section |
| "Before anything else, make the failure visible" | kept, retitled "Basecamp swallows QML errors: read its log first" for the same reason, and names the block's section it instantiates |
| "The gates, all on every PR" and "The test layers" tables | removed; "The test layers" keeps its title (`docs/e2e.md` cites it) and its reasoning, and points at the overlay's `## Test layers` and `## CI gates` as the one copy of each table. The two copies already disagreed on the core unit-test command and on what each gate cannot see |

**`openspec/config.yaml`'s `context:` is left as it is.** It restates the module
split, the JSON contract and the `canWriteLocal` gate, as the overlay's opening
paragraph and `## Hazards` now also do. It reaches a different reader: it is
injected into every `openspec instructions` artifact prompt, which an agent can
receive without having read the overlay, and its own comment limits it to facts
that change only when the architecture does and requires it to move in the
same commit as `CLAUDE.md`. None of those facts changed here, so it stays.

**Considered: leaving `CLAUDE.md` alone and only repointing links.** Ruled out
by the issue, and by the reason it gives: two copies of one rule drift. The
block is rewritten by `/specflow:sync` whenever the plugin moves; a hand-written
restatement above it would not move with it.

### 4. Stale claims fixed in passing

"Prune as you go" applies to a section being edited, so the false or stale
claims in sections this change touches are fixed rather than carried forward:

- **"The inspector Basecamp used to be the second [raw `nix build`] and no
  longer is: `lgs basecamp setup --inspector` builds it."** False twice over:
  the e2e layer builds the dev `#app` with a raw `nix build` (`docs/e2e.md`
  step 1, the overlay's `## Build`), and `--inspector` was a flag on an
  unmerged scaffold branch this repo no longer uses.
- **The single-file QML command was a bare `qmltestrunner` with no `-import`.**
  The overlay was corrected (commit `b3d8fd0`) because a bare `qmltestrunner`
  can be Qt 5; `CLAUDE.md` now points at the overlay's form.
- **"`IssuesTab.qml` and `PatchesTab.qml` … are still not merged, because
  nothing has yet needed them to diverge"** now has issue #28 saying the
  threshold was met. The example is kept as history, with the issue named.
- **The rows of `CLAUDE.md`'s "The test layers" table read `cd radicle/rust-ffi
  && cargo test` and `cd radicle && nix build …`**, chained commands the Bash
  rules forbid, beside an overlay row giving the cd-free form. The table is
  gone (Decision 3), so the overlay's forms are the only ones.
- **The staleness-guard example counted four hand-written copies** and called
  the fourth the signal to reshape, in `CLAUDE.md` and in the overlay's
  `## Hazards`. Issue #29 counts more sites and records the reviewer's case
  against one universal guard. Both now name #29 for the count and the
  counter-argument instead of carrying a number.
- **`docs/M3-embedded-node-plan.md` still said an embedded node "may finally
  let local browsing into CI, closing the gap CLAUDE.md records"**, the clause
  this change drops from the same lesson's new home. `local` is in
  `ui-tests.yml`'s matrix and `CLAUDE.md` records no such gap, so the clause is
  replaced with that fact and a pointer to `docs/e2e.md`.
- **`docs/e2e.md`'s new embedded-home paragraph assumed a throwaway `$HOME`
  isolates the embedded home.** `embeddedHomeFor()` reads `XDG_DATA_HOME`
  first, so a shell exporting it defeats the isolation; the paragraph now says
  so, and that whether sitometres overrides it is unmeasured.
- **"Every bare `npx …` this repo has run went through unprompted"** no longer
  matches `.claude/settings.json`, which allow-lists no `npx`. Dropped rather
  than reworded: the rule it evidenced (never chain) is the block's now.
- **`docs/rust-ffi.md` said "Two probe examples read this machine's real
  `~/.radicle`"**; there are more `probe_*` examples than two, so the sentence
  names the two that check the read path instead of counting.
- **`CLAUDE.md` said `.gitignore` excludes `.claude/*` with the role agents
  re-admitted.** The agents are gone, and the specflow ignore block re-admits
  `.claude/skills/`, which `lgs init` generates; the parenthesis is removed
  rather than rewritten, and the `.gitignore` comment now carries the reasoning
  (Decision 2).

### 5. `docs/OPENSPEC-ARCHIVE.md` is repointed, not deleted

It names `.claude/agents/README.md` twice, so it has to change. It largely
overlaps the plugin's `specflow:openspec-archive` skill, which would argue for
deleting it, but the proposal does not name it, and it carries this
repository's own measurements (the `node-paths` reference pair, the
`skip_specs` example in the archive). The two links now point at the plugin's
`specflow:flow` and `specflow:worktree-discipline` skills. Whether the page
should shrink to its repo-specific half is reported to the owner, not decided
here.

### 6. `openspec validate --specs` in CI is deferred to issue #93

The issue asked for `openspec validate --specs --strict` in `ci.yml`. Run
against this tree with the pod's CLI (`openspec --version` reports 2.1.1),
strict mode fails all nine live specs: it promotes the "Requirement text is
very long (>500 characters)" warning to a failure, and every capability under
`openspec/specs/` has such a requirement. The same command without `--strict`
passes all nine. So the step as written turns every pull request red, and the
three ways out (drop `--strict`, split the long requirements across all nine
specs, or pin an older CLI) are each a decision of their own. The owner moved
the step out of this piece into issue #93; nothing in `ci.yml` changes here.

**What the deferral costs, while #93 is open.** Nothing in CI validates a live
spec at all, so a structurally broken spec under `openspec/specs/` can merge
with every required check green; the `closer`'s validation before archiving is
a stage an agent runs, not a gate CI enforces. And the step as the
issue worded it would not have closed the gap archived foundations task 4.22
named either: `validate --specs` cannot see a declined promotion, a delta that
never reached `openspec/specs/`. #93 records that only a check that every
capability a proposal names has a live spec would.

## Risks / Trade-offs

- [A durable rule dropped in the slimming] → Decisions 2 and 3 name every
  section removed and where its rule lives, so a reviewer can check each row
  against the deleted text with `git show origin/main:<path>`.
- [An agent still addresses an unqualified `spec-writer`] → with the project
  files gone there is no such agent; the dispatch fails visibly rather than
  reaching a stale copy. The runner skill already addresses `specflow:<role>`.
- [Two PLAN.md deferrals have no issue] → reported to the owner in the
  hand-back; they are scope, so they belong in the tracker or nowhere, and
  creating them is the owner's call.
- [Other worktrees still hold the deleted files] → they are on other branches;
  a reader citing one is the hazard the specflow block's "cite from the tree
  you are working in" already covers.

## Migration Plan

None beyond the commits: no data, no format, no runtime. Rollback is a revert
of the piece's squash commit, which restores every deleted file.
