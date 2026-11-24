# Design

## Context

See `proposal.md` for why the gate exists and why it is not strict. Three facts
about the environment shape how it is built, and none is visible from the
proposal:

- **The OpenSpec CLI's exit code does not say it looked at anything.** On the
  pinned version, `openspec validate --specs` over a tree with no
  `openspec/specs/` prints "No items found to validate." and exits 0. A
  capability directory whose `spec.md` is missing or misnamed is skipped
  without a word: renaming `node-seeding/spec.md` to `Spec.md` gives
  "8 passed, 0 failed (8 items)" and exit 0.
- **`main`'s branch protection names its required checks by job name**
  (`gh api repos/{owner}/{repo}/branches/main/protection`). `Lint` is one; a job
  that does not exist yet is not.
- **The CLI declares `engines: node >= 22.20.0`**, and resolves its root by
  walking up from the cwd (`docs/OPENSPEC-ARCHIVE.md`, "The root comes from the
  cwd").

## Goals / Non-Goals

**Goals:**

- A step that fails when a live spec has a structural error, or when a live
  capability was not validated at all, and passes when the only findings are
  warnings.
- The same check, by the same CLI version, runnable locally with one command.

**Non-Goals:**

- Validating in-flight change folders. The `closer`'s
  `openspec validate <change> --strict` before archiving stays that check.
- Making the length warning visible anywhere but the step log.

## Decisions

### 1. `validate --specs` without `--strict`

**Chosen:** the plain command. The owner chose it from issue #93's three
options; `proposal.md` carries the argument against the other two.

**The dead end, so nobody walks it again:** on `@fission-ai/openspec` 2.1.1,
`openspec validate --specs --strict` exits 1 with all nine live specs failed,
and every finding is the warning "Requirement text is very long (>500
characters)", which `--strict` promotes to a failure. Without `--strict` the
same run reports 9 passed and exits 0, printing the same warnings. Archived
foundations task 4.22 recorded the strict run as clean, which was true on the
older CLI it was run with, so "strict ran clean" in that task is not evidence
about today's CLI.

**What breaks if `--strict` is added:** the step goes red on every pull request
until every requirement over 500 characters, across all nine capabilities, is
split. Adding it is a one-flag change once that is done.

### 2. The CLI is pinned exactly, in one place, and `check-stack-pins.py` does not know it

**Chosen:** `@fission-ai/openspec@2.1.1`, written once, as `OPENSPEC` in
`.github/scripts/validate-specs.py`.

**Alternatives:**

- **Unpinned** (`npx @fission-ai/openspec`). Rejected: an upstream release that
  adds an error-level rule would turn every pull request red with no commit here
  to point at. This repo already refuses that trade for its release pipeline:
  the catalog is pinned to `@v1.3` rather than the moving `@v1`, because a moving
  tag "would let the release pipeline change under the catalog without a commit
  here saying so" (`CLAUDE.md`).
- **Pinned in `ci.yml`**, as a step `env`. Rejected: the local command would
  then run whatever the developer has installed, and the point of the script is
  that `python3 .github/scripts/validate-specs.py` is CI's check, exactly.
- **Pinned, and registered in `check-stack-pins.py`.** Rejected for now: that
  script asserts that several copies of one pin agree, and this pin has one copy.
  The drift that does matter (the CLI on an agent's machine against CI's) is not
  a file in the tree, so the script cannot see it. If a second copy of the
  version is ever written down, a doc command for instance, register it then.

**The cost, accepted:** CI validates against rules this repo chose not to
update. An agent's newer local CLI can reject what CI passes. Bumping is a
one-line commit that shows in a diff, which is the point.

**Not pinned: the CLI's own dependencies.** `npx` resolves its caret ranges
fresh on each run. A lockfile (a `package.json` and `package-lock.json` for one
tool) was considered and rejected: the rules under test live in the CLI's own
code, which the pin fixes, and the `ui-specs` job already installs sitometres
the same way. See Risks.

**Telemetry:** the script sets `OPENSPEC_TELEMETRY=0` for the CLI. Upstream
disables telemetry in CI by detection; setting it means a local run of the gate
reports nothing either, and CI does not depend on the detection.

### 3. A script around the CLI, not a bare `run: openspec validate`

**Chosen:** `.github/scripts/validate-specs.py`, in the shape of
`check-stack-pins.py` beside it. It runs the CLI with `--json` from the
repository root, prints each spec's verdict and issues (errors also as
`::error file=…` annotations on the PR), and fails on any of:

- an item the CLI reports invalid;
- **a capability directory under `openspec/specs/` the CLI did not report on**;
- no capability directories at all;
- no parseable JSON report, or a non-zero exit with every item valid.

**Why the coverage checks are guards, and what breaks without them:** the
CLI's exit code is 0 for a run that validated less than the tree holds
(Context). Each check in `problems()` is the only thing that fails one case:

- **the `expected - seen` loop** fails
  `git mv openspec/specs/node-seeding/spec.md openspec/specs/node-seeding/Spec.md`,
  naming `node-seeding/`. Without the loop the run prints "all 8 live specs
  valid" and exits 0; that mutation was run.
- **the `if not expected` check** fails a tree with `openspec/specs/` moved
  away. Without it the CLI's empty report has nothing for the loop to compare,
  and the run exits 0.

This is the qmllint step's old defect in a new place: a gate reporting green
while measuring nothing. Both failures are behaviour the proposal did not ask
for; each carries a `NO SPEC:` marker in the script.

**Alternatives:** parsing the human output's "Totals: N passed" line. Rejected:
`--json` is the CLI's machine interface, and the human format is free to
change. Comparing against `git ls-files` rather than the filesystem. Rejected:
the CLI reads the filesystem, so the comparison has to read the same thing, and
a stray local directory failing a local run is correct, not noise.

**Why `cwd=ROOT`:** the CLI takes its root from the cwd, so a local run from
`radicle/` would otherwise validate nothing or the wrong tree. `ROOT` is
derived from the script's own path, which is also where `SPECS` comes from, so
the two sides of the coverage comparison cannot disagree about the tree.

### 4. A step in the `Lint` job, not a new job

**Chosen:** two steps appended to `lint`, `setup-node` then the script.

**Alternatives:**

- **A job of its own.** Rejected: `Lint` is a required check on `main` and a new
  job name is not, so a new job would report red on a broken spec and still let
  the merge through until someone edited branch protection. Nothing in the tree
  can make that edit or check it was made.
- **The `ui-specs` job**, which already sets up Node 24. Rejected: it is named
  "UI spec validation" and validates sitometres specs, so an OpenSpec failure
  there would be reported under the wrong name.

**`setup-node` is a guard:** without it the step runs on whatever Node the
runner image ships, which this repo does not choose and the CLI's `engines`
may reject. Node 24 matches the `ui-specs` job.

## Risks / Trade-offs

- **A compromised or broken release of one of the CLI's dependencies** reaches
  the `Lint` job unannounced, because only the CLI itself is pinned. →
  Accepted, as it already is for the `ui-specs` job. A lockfile is the remedy if
  it ever bites.
- **The pinned CLI ages.** A rule a newer CLI enforces passes CI. → Bump the
  pin deliberately; the closer's local `validate <change> --strict` runs on
  whatever CLI the agent has, so a newer rule tends to surface there first.
- **An npm registry outage turns `Lint` red.** → The script names it ("printed
  no JSON report"), and npm's own error is forwarded above it.
- **The step log carries one line per long requirement**, around eighty today.
  → Accepted: the owner chose to keep the warning advisory and visible. Errors
  also appear as annotations, so they are not lost in the list.
- **A declined archive promotion still passes.** → Unaddressed, by the
  proposal; it is the second half of task 4.22 and issue #93's "What the gate
  cannot see".

## Migration Plan

None. The step reads files already on `main`; revert the commit to remove it.
