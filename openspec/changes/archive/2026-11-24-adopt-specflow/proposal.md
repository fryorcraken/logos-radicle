# Proposal

## Why

`docs/PLAN.md` is the spec-writer's input, and it has drifted from the code in
ways the flow cannot catch: it calls the `node-config` and `node-seeding`
capabilities present while they sat on an unmerged branch, says the embedded
node has "no running daemon yet" while `startNode`/`stopNode` ship, and stops
mid-sentence in the socket profile-name paragraph. Its forward-looking items
are now GitHub issues in milestones, so the file has no remaining job. The
generic parts of `.claude/agents/*.md` duplicate what the specflow plugin now
provides, and two copies of one rule drift.

## What Changes

Already on the piece branch: the overlay `.claude/specflow/PROJECT.md`, the
specflow block in `CLAUDE.md`, the plugin and permission entries in
`.claude/settings.json`, and the `.gitignore` re-admits. What remains:

- **Delete `.claude/agents/*.md`** — `README.md`, `RUNNER.md` and the seven
  role files. Their generic content is the plugin's; anything repo-specific in
  them that the overlay does not yet carry moves into the overlay first.
- **Slim `CLAUDE.md`** to the specflow block plus the repo-specific sections.
  Rows and prose that point at `.claude/agents/README.md`, `RUNNER.md` or
  `docs/PLAN.md` are removed or repointed.
- **Move the durable rules out of `docs/PLAN.md`, then delete it:**
  - Platform scope → `docs/rust-ffi.md`, with the wording corrected: all three
    of `linux-amd64`, `linux-arm64` and `darwin-arm64` are built and merged
    into releases, tests run on Linux only, and Windows is never built. Keep the
    point that `radicle-node` carries `uds_windows`/`radicle-windows`
    dependencies, so nothing hard-codes against other platforms.
  - "Never point a test at the developer's real Radicle home" (the `probe_*`
    examples do so on purpose and are not tests), and the two-home isolation
    rule → `docs/rust-ffi.md`, test section.
  - The embedded e2e lesson (a spec can create a throwaway embedded home
    rather than depend on a developer's profile) → `docs/e2e.md`, without the
    stale "may finally let local browsing into CI", since `local` is already in
    the `ui-tests.yml` matrix.
  - From "The Logos stack", only the last sentence → `docs/e2e.md`: at the next
    stack bump, re-run a spec on the current sitometres, because 0.1.2 could
    not drive Basecamp 0.3.0. The rest of that section is dropped, being
    already in `CLAUDE.md`'s scaffold section.
  - "Known defects in shipped behaviour" is not moved: it is now an issue.
- **Repoint every file that names `docs/PLAN.md`**: `CLAUDE.md`,
  `docs/M3-embedded-node-plan.md`'s header, `docs/e2e.md`,
  `radicle-ui/tests/ui/sync.yaml`, `radicle/src/radicle_impl.h`, and the
  overlay's Hazards line. Archived changes under `openspec/changes/archive/`
  keep their references: they record what was true when they shipped.
- **Repoint `docs/OPENSPEC-ARCHIVE.md`'s links to `.claude/agents/README.md`**
  at the plugin's skills.

**Not in this change:** the issue also asks for `openspec validate --specs
--strict` in `ci.yml`. The owner moved that step out of this piece into issue
#93 (see the second open question), where PR #95 adds it **without
`--strict`**; `--strict` comes back later as a follow-up. `ci.yml` is not
edited here.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. Nothing here changes what either module does: the change edits prose,
agent instructions, repository configuration and two source comments. The
change declares `skip_specs: true`.

## Open questions for the owner

1. **The two `docs/PLAN.md` sections the issue migration did not cover.**
   Answered by the owner on the issue: "Known defects in shipped behaviour" is
   now its own issue, and "The Logos stack" is dropped except its last
   sentence, which moves to `docs/e2e.md`. Recorded here as the owner's answer;
   What Changes follows it.
2. **`openspec validate --specs --strict` fails on every live spec today.**
   With the CLI as installed in this pod (`openspec --version`), strict mode
   promotes the "Requirement text is very long (>500 characters)" warning to a
   failure, and all nine capabilities under `openspec/specs/` carry such
   requirements; the same command without `--strict` passes all nine. The
   archived foundations task this step comes from recorded the strict run as
   clean, on an older CLI. So the step as the issue words it turns CI red.
   The ways out are to run it without `--strict`, split the long requirements
   (a restructuring of all nine specs, well beyond this piece), or pin a CLI
   version whose strict mode passes.
   Answered by the owner: the step is moved out of this piece into issue #93,
   and runs without `--strict` (PR #95, `piece/ci-validate-specs`), with
   `--strict` left to a later follow-up. This change does not edit `ci.yml`;
   design.md Decision 6 records why.

## Impact

- **Files deleted:** `docs/PLAN.md`, `.claude/agents/*.md`.
- **Files added:** `.claude/specflow/PROJECT.md`.
- **Files edited:** `CLAUDE.md`, `.claude/settings.json`, `.gitignore`,
  `docs/rust-ffi.md`, `docs/e2e.md`, `docs/M3-embedded-node-plan.md`,
  `docs/OPENSPEC-ARCHIVE.md`, `radicle-ui/tests/ui/sync.yaml` (a comment),
  `radicle/src/radicle_impl.h` (a comment).
- **Not edited:** `.github/workflows/ci.yml`; its validation step, without
  `--strict`, is issue #93's, in PR #95.
- **Agent selection:** with the project's role files gone, the plugin's
  `specflow:<role>` agents are the only ones of those names. The deleted
  `spec-writer.md`'s frontmatter is the harness's selection text for the
  unqualified `spec-writer`, so it stops being offered.
- **The spec-writer's input** becomes the change's GitHub issue, not
  `docs/PLAN.md`.
- **No code, API or behaviour change** in either module. The two source-file
  edits are comments.
