# Proposal

## Why

Nothing in CI reads `openspec/specs/`. A live spec that stops parsing, loses a
scenario or carries a malformed requirement heading merges without any gate
going red. This is the cheap half of archived foundations task 4.22
(`openspec/changes/archive/2026-09-12-m3-embedded-node-foundations/tasks.md`),
which asked for `openspec validate --specs --strict` in `ci.yml`. Issue #51 asked
for the same step. Issue #93 holds it, because the strict form cannot be added
as written.

**Strict mode fails every live spec on the current CLI.** On
`@fission-ai/openspec` 2.1.1, `openspec validate --specs --strict` exits 1 with
every spec failed. Its only findings are the warning `Requirement text is very
long (>500 characters)`, which `--strict` promotes to a failure. Without
`--strict`, the same command passes every spec, prints the same warnings and
exits 0. Task 4.22 recorded the strict run as clean, which was true on the
older CLI it was run with. Reproduce with both commands from the repo root.

## What Changes

The owner chose between issue #93's three options: **add
`openspec validate --specs` to `ci.yml` without `--strict`.**

- **A `ci.yml` step that runs `openspec validate --specs` on every pull
  request**, like the other `ci.yml` gates. It fails the job on a
  structural error in any live spec. The length warning stays advisory: it
  is printed in the log and does not fail the step.
- **Why not strict:** the other two options each cost more than the
  advisory warning does.
  - **Pinning an older CLI** whose strict mode passes ties CI to a version
    nobody has found yet. It would also hold back every later CLI fix for the
    sake of one warning.
  - **Splitting the long requirements** so strict passes on 2.1.1 touches
    every live capability. It is a change of its own: rewriting requirement
    text across every contract cannot ride on a CI change and still be
    reviewable.
  - Either can still be taken later. Adding `--strict` to this step is then a
    one-flag change.
- **The step must be seen failing.** A deliberately broken live spec must turn
  it red before it is trusted. The same goes for the fact that a warning alone
  does not, because that distinction is the whole of the owner's decision.
- **The documents that list CI's gates gain this one**, CLAUDE.md's "The
  gates, all on every PR" table among them. Its entry says what the gate
  catches and what it cannot see (below), so a green run is not read as
  more than it measured.

### What the gate cannot see

- **A declined promotion.** This is the second half of task 4.22, and this
  change leaves it unaddressed. `openspec archive` promotes a delta into
  `openspec/specs/` through a prompt. Declining the prompt archives the change
  without publishing its contract, and leaves every live spec valid. So
  `validate --specs` passes exactly when the contract is missing. Only a
  check that every capability named in an archived `proposal.md` has a live
  spec would catch it, and that is not built here.
- **A requirement too long to read**, by the owner's decision above.
- **An in-flight change's delta.** `--specs` reads `openspec/specs/` only, so
  a change folder on a piece branch is not validated by this step. The
  `closer`'s `openspec validate <change> --strict` before archiving remains
  that check.
- **A spec that contradicts itself.** `validate` checks heading structure, not
  consistency (`docs/OPENSPEC-ARCHIVE.md`).

### Not in this change

- `--strict`, an openspec version chosen for its strict behaviour, and any
  edit to a live spec's text. All three are the options not chosen.
- The declined-promotion check above.
- `--changes` or `--all`. The owner named `--specs`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This is a CI-only change: no module behaviour moves and no requirement
in `openspec/specs/` changes, so the change declares `skip_specs: true` rather
than inventing a requirement to satisfy validation. The live capabilities
are this step's input, not its subject, and their text is untouched.

## Impact

- **Files**: `.github/workflows/ci.yml`; `CLAUDE.md` (the gates table, and the
  CI section's list of what `ci.yml` runs). Also any agent file or
  `docs/OPENSPEC-ARCHIVE.md` passage that says spec validation happens only by
  hand. No module source, test or spec text.
- **Dependencies**: CI gains `@fission-ai/openspec` from npm, under its scoped
  name. The bare `openspec` package is an unrelated placeholder. Whether the
  version is pinned, and if so whether `check-stack-pins.py` should know about
  it, is a design decision with a consequence either way. Unpinned, a new CLI
  release can add an error-level rule and turn every PR red with no commit
  here. Pinned, the step validates against rules this repo chose not to
  update.
- **Gates**: one new failing condition on every PR: a live spec with a
  structural error. A PR that only adds long requirement text is unaffected.
