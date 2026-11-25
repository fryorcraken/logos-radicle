# Design

## Context

See `proposal.md` for why the gate exists. Four facts about the environment
shape how it is built, none of them visible from the proposal:

- **The length rule is a warning that `--strict` promotes.** On
  `@fission-ai/openspec` 2.1.1, `openspec validate --specs` over the live specs
  as they stand on `main` reports 9 passed with 73 "Requirement text is very
  long (>500 characters)" warnings and exits 0; the same command with
  `--strict` reports 0 passed, 9 failed and exits 1. The rule measures the text
  between a requirement's heading and its first scenario
  (`MAX_REQUIREMENT_TEXT_LENGTH`, used in `dist/core/validation/validator.js`).
  Archived foundations task 4.22 recorded the strict run as clean; that was
  true on the older CLI it was run with, so it is not evidence about this one.
- **The gate reads `openspec/specs/` as the branch holds it.** This change fixes
  the live specs through a delta, and a delta changes nothing there until
  `openspec archive` promotes it. After a trial archive of this change, the
  strict command reports 9 passed, 0 failed (Decision 5).
- **The CLI's exit code does not say it read everything.** A capability
  directory with no `spec.md` is skipped without a word: with
  `node-seeding/spec.md` renamed to `Spec.md`, the strict run on the archived
  tree reports "8 passed, 0 failed (8 items)" and no failure.
- **The CLI declares `engines: node >= 22.20.0`**, and resolves its root by
  walking up from the cwd (`docs/OPENSPEC-ARCHIVE.md`, "The root comes from the
  cwd").

## Goals / Non-Goals

**Goals:**

- A step in a required check that fails when any live spec fails
  `validate --strict`, the length rule included, on a CLI version this repo
  chose.
- A home, in this document, for every passage of reasoning the spec-writer cut
  from the live specs to bring them under the limit, so that nothing the specs
  used to explain is lost when they stop explaining it.

**Non-Goals:**

- Validating in-flight change folders in CI. The `closer`'s
  `openspec validate <change> --strict` before archiving stays that check.
- A coverage guard around the CLI (see Decision 3).
- Pinning the 37 new scenarios with tests. That is the `tester`'s stage;
  `tasks.md` group 3 lists them.

## Decisions

### 1. `validate --specs --strict`, as a plain step in `Lint`

**Chosen:** two steps appended to the `lint` job in `.github/workflows/ci.yml`:
`actions/setup-node` at Node 24, then
`npx --yes @fission-ai/openspec@<pin> validate --specs --strict --no-interactive`.

The issue's own reasoning is why it is `--strict` and not the plain command:
without `--strict`, the only rule the live specs break is a warning, so a gate
on the plain command passes a tree that the issue was opened to stop
reaching `main`. PR #95 took the other route, the plain command with the
warning advisory, and was closed unmerged; the issue now asks for `--strict`
with the specs split to meet it.

**Alternatives:**

- **A job of its own.** Rejected: `Lint` is a required check on `main`, and a
  new job name is not, so a new job would go red on a broken spec and still let
  the merge through until branch protection was edited to name it. Nothing in
  the tree can make that edit or check it was made.
- **The `ui-specs` job**, which already sets up Node 24. Rejected: it is named
  "UI spec validation" and validates sitometres specs, so an OpenSpec failure
  there would be reported under the wrong name.
- **`--all`**, which would also validate in-flight change folders. Rejected:
  the proposal names `--specs`, and a change folder on a piece branch is
  mid-edit by design until the `closer` validates it.

**Guard, and what breaks without it:**

- **`--strict`.** Without it the step passes `main` as it stands today, with
  73 warnings: measured as exit 0 against exit 1 with the flag.

**Environment pins, not measured guards.** Nothing below has been shown to turn
the step red when removed; each fixes something the step would otherwise
inherit.

- **`setup-node`.** Without it the step runs on whatever Node the runner image
  ships, which this repo does not choose and the CLI's `engines` may reject.
  Which Node `ubuntu-latest` ships was not measured, so whether the step fails
  without this is unknown.
- **Node 24, not the `engines` floor of 22.20.** The floor would satisfy the
  CLI equally. 24 is the version `ui-specs` already sets up, so the workflow
  holds one Node version to bump rather than two that drift.
- **`--no-interactive`.** Read from the CLI's source, not measured: with
  `--specs` the command takes the bulk path, where interactivity decides one
  thing, whether an `ora` spinner is drawn (`runBulkValidation` in
  `dist/commands/validate.js`). `isInteractive` already answers no when `CI` is
  set (`dist/utils/interactive.js`), so on a runner the flag changes nothing. It
  is there for the reason `OPENSPEC_TELEMETRY` is: the step does not depend on
  the CLI detecting CI, and a reproduction from a terminal prints what CI
  prints.

### 2. The CLI is pinned exactly, once, in `ci.yml`

**Chosen:** `@fission-ai/openspec@2.1.1`, written once, in the step's `run:`.
2.1.1 is the version the issue measured and the version every number in this
document was measured with.

**Alternatives:**

- **Unpinned** (`npx @fission-ai/openspec`). Rejected: an upstream release
  that adds an error-level rule, or tightens the length limit, would turn every
  pull request red with no commit here to point at. This repo refuses that trade
  for its release pipeline too: the catalog is pinned to `@v1.3` rather than the
  moving `@v1` (`CLAUDE.md`).
- **A wrapper script holding the pin** (`.github/scripts/validate-specs.py`, as
  PR #95 had). Rejected with Decision 3: without a guard to carry, a script is
  one more file around one command.
- **Registered in `check-stack-pins.py`.** Rejected: that script asserts that
  several copies of one pin agree, and this pin has one copy. `CLAUDE.md` and
  `docs/OPENSPEC-ARCHIVE.md` say "the version the `Lint` step pins" rather than
  the number, so that stays true. If a second copy is ever written, register it
  then.

**The cost, accepted:** CI validates against rules this repo chose not to
update, and an agent's newer local CLI can reject what CI passes. Bumping is a
one-line commit, which is the point.

**Not pinned: the CLI's own dependencies.** Of the 13 direct dependencies
2.1.1 declares, 11 are caret ranges and two (`skills`, `cross-spawn`) are exact
(`npm view @fission-ai/openspec@2.1.1 dependencies`), and `npx` resolves the
ranges fresh on each run with no lockfile. So the pin fixes the CLI, not
everything that decides a verdict: the length rule is the CLI's own code, but
the spec schemas it validates against are `zod` schemas
(`dist/core/schemas/*.schema.js`), and `yaml` is the parser behind the CLI's
project configuration (`dist/core/project-config.js`), so a minor release of
either can move what passes with no commit here. A
compromised release of any ranged dependency also runs in `Lint`, bounded by
the job's read-only token, no secrets and no install scripts in the resolved
tree (the security review measured all three).

A `package.json` and lockfile for one tool, installed with `npm ci`, was
considered and not taken: it adds two files and a bump path for every
transitive release, for a step whose worst case is a red `Lint` that names
the failing rule in its log. `ui-specs` installs sitometres the same way. The
comment beside the step says the dependencies are unpinned, so it does not
claim the gate is frozen. If a dependency release ever moves a verdict, the
lockfile is the remedy.

**Telemetry:** the step sets `OPENSPEC_TELEMETRY=0`, which the CLI's telemetry
and its version check both honour (`dist/telemetry/opt-out.js`). Both already
turn off when `CI` is set; setting the variable means the step does not depend
on that detection.

### 3. No coverage guard around the CLI

**Chosen:** the CLI's own exit status is the step's result.

PR #95 wrapped the CLI in a script that also failed when a capability
directory went unvalidated or `openspec/specs/` held none, both cases in which
the CLI exits 0 (Context). It is a real blind spot, and the same shape as the
qmllint step's old defect: a gate reporting green while measuring less than it
appears to.

It is not built here because it is behaviour the proposal does not ask for:
the proposal asks that a live spec failing validation fail a required check,
and a directory with no `spec.md` is not a spec the CLI considers. Whether the
gate must also prove it read every directory is a decision about what the gate
promises, for a spec to state before anyone builds a guard, so it is not built
here. The blind spot is written beside the step in `ci.yml`, in
`docs/OPENSPEC-ARCHIVE.md`, and in Risks below; those say the gap exists.
Closing it is tracked by issue #99 (a capability directory with no `spec.md`,
or an empty `openspec/specs/`), and the other gap the gate leaves open, a
declined archive promotion, by issue #100.

### 4. Where the reasoning cut from the specs lives

**Chosen:** in this document, under "Reasoning carried out of the specs" below,
one entry per requirement, named by requirement heading so a reader of the live
spec can find it by grepping the archive. That includes the two passages that
sat after a requirement's scenarios, which the length rule never measured
(`node-paths`' "Embedded mode must never resolve the user's own Radicle home"
and `source-modes`' "A mode that cannot start asks its node for nothing"); the
delta removes both, and their entries below carry them.

**Alternatives:**

- **Leave it in the spec, after the scenarios**, where the length rule does not
  measure it, so the strict run passes over it. Rejected: a spec is the
  behaviour contract, and reasoning in it is reasoning a reader has to separate
  from the contract at every reading. The length rule not reaching a passage is
  no reason for it to stay.
- **The spec's `## Purpose`.** Rejected for the same reason, and because a
  Purpose describes the capability, not why one rule in it exists.
- **Work not done goes to its issue, not here.** Where a cut passage named an
  open gap, the gap already has an issue, and the entry below names it.

### 5. Verifying a change whose code is CI and whose effect is the archive

There is no test layer that can see a workflow step, and the step's effect
depends on a promotion that is the `closer`'s to make. The verification is
therefore a trial: archive this change in this worktree, run the step's exact
command, then restore the tree (`git reset --hard` and `git clean` of the
archived folder), leaving the archive to the `closer`. Measured: 9 passed,
0 failed after the trial archive; 0 passed, 9 failed before it.

### 6. One `MUST` dropped: `local` mode finding the home-derived socket

**Chosen:** `node-paths`' "Socket precedence prefers the runtime directory over
the home" no longer says, as a rule of its own, that `local` mode MUST still
find the home-derived socket when no runtime directory exists. It is the one
live `MUST` this change keeps in no requirement; the spec-test review diffed
every `MODIFIED` requirement against its live text sentence by sentence and
found no other.

The requirement's subject is `resolveSocket`, a function of its inputs, and
everything else in it, scenarios included, is stated in those inputs. The
dropped sentence was the one about a caller. What it asked of `resolveSocket`
the requirement still says: with no runtime directory the result is
`<home>/node/control.sock`, "a fallback that MUST be retained", and the
scenario "Without a runtime directory the socket falls back under the home"
pins that answer.

**Alternatives:**

- **Keep the sentence.** Rejected because it is the only clause in the
  requirement a reader cannot check against `resolveSocket`'s scenarios, so it
  read as covered while nothing covered it (below).
- **Move it into "Without a runtime directory the socket falls back under the
  home"** as a `local`-mode step. Rejected here because that scenario is in
  `resolveSocket`'s terms; a `local`-mode step is a different scenario, and
  adding one is a spec change this change, which promises no new behaviour,
  should not make on its own initiative.

**What it costs, measured:** no scenario, live or delta, pins the caller's half
of the dropped sentence, that `local` mode hands `resolveSocket` an empty
runtime directory when `XDG_RUNTIME_DIR` is unset (`resolvePathsFromEnv` in
`radicle/src/local_store.cpp`, reading it with `envOr`). Under `radicle/tests/`
the only test that sets `XDG_RUNTIME_DIR` is `test_local_writer.cpp`'s, to a
value; none unsets it and asserts the home-derived socket. The live sentence
had no scenario either, so the drop removes a statement and no test. A scenario
for it is reported to the `spec-writer` rather than written here.

### 7. Where the live text was wrong, the delta follows the code

The deltas are not all byte-for-byte copies of the live text they replace, and
the scenarios are not all unchanged. Most `MODIFIED` requirements keep every
scenario byte for byte; three `embedded-state` scenarios each change one step,
and the requirement text listed below is corrected rather than only cut. Each
correction resolves a defect the live spec already carried, toward what the
code and its tests already do, so no implementation changes. The review that
found them is the spec-test review's; the corrections are the `spec-writer`'s.

- **The state conditions disagreed with the order they are read in.** The live
  spec renders the first state, in the order blocked, no identity, stopped,
  starting, start failed, not serving, running and empty, whose condition
  holds, and gave `stopped` as "the node reports neither running nor serving".
  Read literally, that holds in the ordinary case of a refused start, which
  reports `running:false`, so start failed could never be rendered there; yet
  the live spec's own scenario "A refused automatic start is reported as a
  refusal" requires exactly that. The defect is the live spec's; the first
  delta carried it across unchanged. The `ADDED` "Each
  embedded state has one condition" now gives `stopped` as "neither `running`
  nor `serving`, and the last `startNode` not refused", and `starting` as "a
  `startNode` outstanding, with `running` or `serving`". The order is unchanged.
- **Why `starting` needs a running report, rather than `stopped` excluding an
  outstanding start.** The second fix would also make the order consistent,
  and would change what the code must do. `EmbeddedState.qml`'s `current`
  tests `!running && !serving` before `startPending`, so while a start is
  outstanding and the node still reports `running:false` it renders `stopped`.
  Two tests in `tst_embedded_state.qml` pin that, with `startPending` true:
  `test_no_action_is_named_while_a_start_is_outstanding` (`:336`) and
  `test_an_outstanding_start_withholds_a_hosted_start_control` (`:406`; `:372`
  before the tester's last pass inserted a test above it). That is the live
  order read literally, so the spec now says it.
- **Three `MODIFIED` scenarios change one step each to agree.** "The reply
  clears the outstanding start, not the request" and "The start action is
  withheld while a start is outstanding" gain, in their **GIVEN**, the node
  status their requirement already assumed: `running:true`, `serving:false`.
  "A stopped node with an unencrypted key starts by itself" said the rendered
  state MUST be the starting one over a node reporting `running:false`; it now
  says the state stays stopped while that call is outstanding and the node
  still reports `running:false`. The tests for the changed steps are
  `test_the_reply_clears_the_outstanding_start_not_the_request`
  (`tst_embedded_state.qml`) and the outstanding-start leg of
  `test_an_unencrypted_stopped_node_is_started_without_being_asked`
  (`tst_embedded_panel.qml`), both from the tester's last pass.
- **"Each state MUST carry its own action" sat beside states that name none.**
  The `ADDED` "Each state names its own next action" lists the four states that
  name an action and the four that name none, which is `EmbeddedState.qml`'s
  `actionKind`.
- **Optional wording.** `embedded-state`'s "MUST be permitted" for the
  empty-list placeholder obliged nothing and is dropped; `embedded-setup`'s
  lower-case "what it may do, what it must state" becomes the `MUST` it stood
  for.
- **A term the live text left undefined.** `embedded-header`'s rule is keyed on
  whether a mode "has an identity"; the split-out requirement defines it as
  deriving the `local` method prefix, which is how `SourceState.qml` decides
  it (see that capability's entry below).

**Alternative:** carry every scenario and condition verbatim and report the
defects for a later change. Rejected because the split made them worse: the
`ADDED` condition list restated `stopped` without the order's context, so the
contradiction a reader of the live spec had to assemble across two paragraphs
became one list. A delta that promotes a contradiction it has been shown is
not a neutral carry.

## Reasoning carried out of the specs

Each entry is reasoning that sat in the requirement it names before this
change, and that the spec now states only as behaviour. The behaviour itself is
the spec's; nothing here adds to it.

### `embedded-header`

- **Every mode with an identity shows it in the header.** `radicle_impl.h`
  states the reason as a requirement on any view: a user who believes they are
  acting as their existing DID while acting as a different one finds their
  repositories missing, with nothing on screen saying why. Embedded is where
  that is likeliest, because its identity is one the module created. A mode with
  no identity shows nothing rather than a placeholder, because a placeholder
  suggests a value that is loading.
- **One rule decides the header identity for every mode** (split out). The
  header keeps no list of its own: a mode has an identity exactly when it
  derives the `local` method prefix, so the one list that decides routing also
  decides the header. A second list is one someone has to notice and extend,
  which is how Embedded came to show nothing while the contract required
  otherwise. A mode added later therefore gains an identity in the same edit
  that routes it to the `local*` methods, and not before: until then it
  derives `remote` and shows none, the safe direction `source-modes` requires
  of a mode the view does not recognise. The live text called the rule "right
  for a mode added later without anyone editing it"; that holds for the header
  line, not for the routing. The definition is what this change adds to the
  requirement, and the scenario pins the relationship (shown in `local` and
  `embedded`, not in `explore` or an unrecognised string) rather than a mode
  nobody has written, which no input can be. **Mutation evidence: none for the
  new test.** The mutation the `SourceState.qml` comment names (`current ===
  "local"` rewritten as `mode === "local"`) was run against the three older
  tests it lists; neither it nor "every mode but Explore" has been run against
  `test_the_identity_is_shown_in_exactly_the_modes_that_derive_local`, because
  no pod that worked on this change had a Qt 6 `qmltestrunner`. By reading,
  the first reddens its `embedded` case and the second its unrecognised-string
  cases. The test is also pinned on a copy of `Main.qml`'s header binding,
  because `Main.qml` cannot be instantiated in a component test, and
  `local.yaml` never runs Embedded with an identity, so no layer sees a change
  to the binding itself. The owner accepted that gap: the scenario pins the
  rule, not the wiring, and no test, source change or issue follows. The owner
  also accepted CI green as the evidence for this change's tests and deferred
  the named mutations, which need a Qt 6 `qmltestrunner`, a C compiler and
  Nix, none of which the pods that worked on it had.
- **The header identity is copied by taking it.** The one thing anyone does
  with a DID is paste it — into `rad id update --allow`, or a message asking a
  delegate to authorise this node — so copying is what the display serves. A
  clipboard is not available in every environment this module runs in, and a
  confirmation shown regardless tells the user their DID is in an empty buffer.

### `embedded-identity`

- **Neither identity method takes a home.** A safety property: a home
  parameter crosses the QtRO boundary from a sandboxed view, and would let a
  caller point key creation at the user's own `~/.radicle`.
- **Reporting what the embedded home holds.** `encrypted` is the only way a
  later session can learn whether starting the node needs a passphrase. The
  node is handed an already-decrypted key when it is built, so the passphrase
  is supplied at start or not at all; the only other reply with an `encrypted`
  field is `createEmbeddedIdentity`'s, which no later session has and which
  computes the value from its argument. Without it a surface must either always
  prompt, wrong for an unencrypted key, or attempt a start and read the
  failure, a destructive probe.
- **canWriteLocal is not a substitute for the encrypted field** (split out).
  `getCapabilities().canWriteLocal` probes the mode in force's home, not
  necessarily the embedded one, and a `false` from it conflates an encrypted key
  with a missing or unreadable one. A surface reading it as "encrypted" prompts
  for a passphrase an unencrypted key does not have.
- **An unreadable key is not reported as unencrypted** (split out). An
  unreadable key is not an unencrypted one, so the field reports no guess and
  `problem` says why.
- **No data directory is an answer, not an error** (split out). The question was
  answered; a caller renders a blocked state rather than a failed call it might
  retry.
- **Three home states, not two.** Empty is safe to create into; Complete would
  be destroyed by creating; Partial means initialisation did not finish and no
  usable identity exists. `storage/` is not the marker because the directory
  tree, `storage/` included, is created before any key is written. Creating
  into a Partial home is a recovery, not a destructive act, which is why it
  reports `exists:false`.
- **An occupied home is refused and never overwritten.** The signing key is the
  identity: replacing it is unrecoverable, and every repository delegating to
  it becomes unreachable. Refusing before the tree is touched means a refused
  call leaves the filesystem as it found it.
- **A half-created home is reported as recoverable, not occupied.** Claiming a
  signing key is at stake is false and unactionable for a stub left by a failed
  run. The module reports rather than repairs, because a misclassification that
  deleted key material would cost an identity — the one failure here with no
  recovery.
- **The created identity is real and readable.** `nodeId` is spelled as
  `getCapabilities().nodeId` spells it so a caller can compare the two without
  normalising.
- **The passphrase decides whether the profile can sign.** As shipped, the
  reply's `encrypted` is computed from the input (`!passphrase.is_empty()`), and
  the same value selects the passphrase given to `Profile::init`, so the two
  cannot disagree in this code; but the field is not an observation of what was
  written. A future change falling back to an unencrypted key would satisfy the
  field while lying. What closes that is the signability rule, which is checked
  against the key. An earlier draft said the field was "derived from what was
  done", which the code does not do.
- **The embedded identity is a new one and must be stated as such.** It is not,
  and cannot be, the user's existing DID: their repositories are not in this
  storage and their allow-listed DID is not this one. `rad auth` is advice for a
  mode the user did not pick.

### `embedded-setup`

- **Four steps in a fixed order.** The resume after the preflight answers is
  the one other thing that moves the step in force, and it may land on any
  step; the no-skip rule is about the advance and back controls, not about the
  step never changing by more than one. Identity creation is not reversible
  through this flow, which is why a step that performed it reports what it did
  when returned to (now "Going back undoes nothing").
- **The preflight reports four findings before offering a choice.** A
  half-created home reports `exists:false` too, so the identity finding cannot
  claim the home is empty. The setup starts no node, so the socket finding
  blocks nothing and is reported for what it tells the user: something else is
  using the socket this node would want. Backend sentences are shown verbatim
  because they name the path tried and the limit exceeded.
- **A failed preflight blocks the step it gates and says which.** Reporting a
  fact and blocking a step are different acts. Only one finding gates work this
  flow does, a consequence of the setup starting no node. A missing `git` blocks
  nothing because identity creation spawns no `git`, but it is still reported:
  the node will be unable to fetch, and the user is better told now. A step
  blocked for work it does not do is a step a user cannot get past for no
  reason.
- **An unresolvable home blocks the identity step, an occupied one only
  creation** (split out). With no home there is no identity and no way to make
  one, so the forward control is not enabled; with an occupied home the step's
  work is already done, so the control stays and advances.
- **A half-created home does not block the identity step** (split out). The
  preflight cannot tell it from an empty home through `getEmbeddedIdentity()`,
  and the backend's refusal is the only surface naming the `keys` path to
  remove, so submitting creation is how the user learns the recovery.
- **The embedded step confirms Embedded and states the identity consequence.**
  This step is the only place the flow states it, so it is stated in full. It
  was once split between this step and a terminal step that restated it with
  the DID; that step is gone, and a consequence stated in half is one a user
  acts on without. Stating it before creation is right on its own terms: after
  the identity exists, the statement comes after the decision it informs.
- **The embedded step offers no other mode** (split out). The flow sets up one
  mode, so it has no unstartable alternative to caption. A user who does not
  want Embedded leaves through the close control every step offers. This step
  is therefore not a mode picker, so `source-modes`' rule that a view
  annotating availability consume `startableModes` does not reach it; that rule
  binds the header toggle and the settings panel.
- **The embedded step's control puts Embedded in force** (split out). A mode
  written on arrival would put a module into Embedded because the user opened a
  screen, and would leave the stated consequence something shown rather than
  answered.
- **Advancing past the embedded step waits for the backend** (split out). The
  steps after it are about a node no other mode runs, and a backend that has
  not reported `embedded` has not confirmed the write landed.
- **Returning to the embedded step reports Embedded rather than re-offering
  it** (split out). The mode write is idempotent, unlike identity creation, so
  this is about not asking a question the backend has answered, not about
  preventing a second act; that is why the statement stays displayed.
- **The identity step offers one forward control.** Creating the identity is
  how a user leaves the step. Two controls for one act make the user perform a
  second act whose only answer is yes, and make it possible to advance past a
  step whose work was never done. With an identity present, a creation call has
  no outcome but a refusal the user did not ask for.
- **The identity step's forward control says what it will do** (split out). A
  label naming creation on a step that will only advance states an act that
  will not happen, the same defect as a control doing the wrong thing.
- **The identity step distinguishes its three states.** A user who already
  holds an identity has succeeded at this step; telling them creation is
  refused describes a failure that did not happen, since no attempt was made.
- **Created and refused are never shown together** (split out). Both describe
  outcomes of the same act, and both on screen leave the user unable to tell
  which occurred.
- **The identity step names the home it writes to.** A DID says nothing about
  where the identity lives. The embedded home is derived from the Basecamp
  profile's data directory, which the user did not choose and cannot guess, so a
  step reporting only a DID leaves them unable to find, back up or inspect what
  was created, or to tell it from their own Radicle home.
- **The identity step states the passphrase trade where it is chosen.** The node
  is handed a decrypted key when it is built, so a passphrase cannot be supplied
  later; that makes the choice one about every opening of Embedded, chosen here
  and nowhere else. A statement of only the benefit, or only the cost, is a
  recommendation rather than a trade.
- **An existing identity is offered no passphrase trade** (split out). Its key
  was sealed, or not, when it was created, and this flow cannot change that.
- **The setup's passphrase is used for creation alone** (split out). The setup
  starts no node, so it has no second use for it.
- **The alias is the backend's to judge** (split out). `embedded-identity`
  requires the backend to pass through the `radicle` crate's statement of the
  rule; a second rule in the view would drift from it.
- **The network step states outbound-only as the default in force.** A user
  reading "no listen address" cannot derive "another machine cannot clone from
  this one", and the second sentence decides whether the default is acceptable.
  No module method persists a listen address, so an inbound toggle would record
  nothing; a control that silently does nothing is worse than none.
- **The setup ends by finishing, not by being dismissed.** A terminal screen
  whose only act is dismissal asks for an act that changes nothing; its facts
  belong where they are wanted, the DID in the header and the node's state on
  the surface the user returns to, which `embedded-state` owns.
- **A step reports a backend refusal rather than a generic failure.** The
  refusals name the home in the way, the path to remove, the value tried and the
  limit exceeded, which is the reason validation happens on write.
- **The setup is raised over the view, and lowered by its own close.** As a
  navigation destination, back would have to choose between the previous step
  and the previous screen, and the navigation state knows nothing about steps;
  the setup's own back already moves between steps. Raised over the view,
  lowering it restores exactly what was underneath. A surface that closed itself
  would leave whatever raised it believing it is still up.
- **The setup and the settings surface are never raised together.** Both are
  opaque and cover the same screen; two at once leaves one unreachable behind
  the other with no control to lower it — the one-way door this module has
  shipped once, which the settings surface's close control exists to prevent.
- **The setup opens only when a user asks for it.** Choosing a mode and
  configuring a node are different acts; a modal appearing because a segment was
  clicked is the same defect class as writing the mode because a screen was
  opened. A reported state is not a request.
- **A reopened setup lands at the first step with work left.** Every write the
  setup makes is separately durable, so there is no half-committed state to
  resume into, and a remembered step index is a second opinion that is wrong
  whenever anything changed between showings.
- **The node's state does not decide where the setup reopens** (split out). This
  flow neither starts nor stops a node, so a running node says nothing about
  which step has work; landing differently for it would report the node's state
  through a step, which is `embedded-state`'s job.
- **The resume waits for the preflight and moves once** (split out). Advancing
  moves the staleness epoch the preflight replies were issued under, so a
  step-by-step resume would invalidate the replies that chose its destination.
  Every finding has a legitimate falsy value, so choosing from defaults would
  land every reopening on the same early step.
- **A resumed step behaves as one reached by advancing** (split out). The
  identity step's control advancing when an identity exists is about a user who
  reaches it by going back, or with an identity that appeared after the
  preflight; it is not a reason for the resume to land there, which would
  present a step with nothing to do as the first with work left.
- **The setup is offered only for work it can do.** Starting is not a step of
  this flow; a node with an identity has nothing left for any step to do, so
  raising it for a start would present steps already done and none of them the
  act asked for. A restart is two calls whose refusals are different sentences,
  which this capability sequences neither of. Keying on the request rather than
  the state stops a host that never receives a start request today from
  satisfying the rule by accident.

### `embedded-state`

- **Embedded's empty surface reports one of seven states.** The order runs from
  the most fundamental obstacle to the least, so a user is told there is nowhere
  to write before being told a start failed, rather than sent to retry a start
  that cannot succeed. A stored state is a second opinion that can be wrong after
  a node dies, a passphrase is refused, or the mode changes elsewhere. The
  blocked sentence is verbatim because it names the path and the limit, which
  the view cannot reconstruct.
- **Each state names its own next action** (split out). The states differ in
  the action they offer, not only in wording, so one banner cannot stand in for
  them. An unencrypted stopped node is the one state whose next act is the
  module's. Naming an action is distinct from its being available, which is
  stated separately.
- **A mode with no node to ask issues no list request.** A reply already in
  flight still arrives, still passes the staleness guard comparing mode and
  method prefix, and repopulates the model under a panel saying no node exists.
  A guard keyed on startability never fires for `embedded`, which is startable;
  that is how this surface was lost. A node that has stopped serving still
  answers reads, because reads do not touch the daemon.
- **The no-repositories wording is withheld where no node exists.** The
  wording claims a node exists, was asked and answered with nothing. Where no
  node answered it is a different and false claim, and sends a user looking for
  missing repositories rather than for the setup they have not run. Saying the
  node lists what it seeds makes an empty list read as nothing seeded rather
  than something lost.
- **A node that has stopped serving reads as neither running nor stopped.**
  Reported
  running, the user hunts a network fault while fetches never arrive and the
  list looks healthy; reported stopped, they try a start that is refused because
  the runtime still holds the socket. The state is reachable because the FFI
  panic guard does not reach the threads a running node spawns: a node whose
  runtime panics leaves the module's bookkeeping at `running:true`, and
  `serving` is the only field that notices.
- **Starting is distinguished from not serving by the outstanding call.** A
  poll during startup reports the same two fields as a node whose threads died;
  the outstanding call is the one fact only the view holds. Clearing it on issue
  would settle a node that never answers into not serving while its start is in
  flight. Not naming the start action while starting holds whether or not the
  action can be carried out, so it is already true the day it can.
- **A refused start is displayed as the backend worded it.** The refusals name
  the socket in use, the passphrase that did not unlock the key and the home in
  the way. A wrong passphrase is the likeliest refusal, since an encrypted key
  is unlocked at every start, and is only actionable when named.
- **The surface offers setup without performing it.** A state panel that acts
  because it was displayed acts without being asked. How the setup is hosted is
  `embedded-setup`'s.
- **An unencrypted identity starts its node without being asked.** A user who
  set a node up asked for a node; requiring a press the module could make, at
  every opening for the life of the mode, asks for an act with one answer. This
  is the one case with nothing to decline, because no secret is needed.
  Re-issuing on every reading would start a node repeatedly against a slow
  backend. The other states are excluded: nothing to start in blocked and no
  identity; a bare start is refused in not serving; retrying start failed
  unasked loops against a backend that refuses every time.
- **An automatic start is reported as a requested one is** (split out). A node
  that failed to start is the same fact either way, and a user who pressed
  nothing has less context, not more.
- **An encrypted identity is asked for its passphrase, not sent to a flow.** A
  passphrase is one answer to one question at one moment; a multi-step flow would
  make the user walk steps already done to reach the one that is not.
- **Prompting is keyed on the identity's encryption** (split out). Keying it on
  a previous failure would make the prompt a consequence of an error rather than
  of the identity's own nature.
- **A node this module started is not reported as one in the way.** This is the
  defect the setup flow shipped, and the reason starting moved here: a screen
  warned that a node was already answering on the resolved socket directly above
  its own report that the node was running. Both readings were true, and
  together described a conflict that did not exist. A correct fact rendered as a
  hazard when it describes the user's own success sends them looking for a
  second node that is not there.
- **Only an action something can carry out is offered as enabled.** An enabled
  control that does nothing reads as a broken module and leaves nothing else to
  try. Restart has no host: it is a stop then a start whose two refusals a user
  should see separately, and nothing sequences the pair (issue #72 holds that
  work). Keying on whether a request is hosted means hosting one enables it with
  nothing else edited.
- **The blank-pane assertion keeps a live referent.** An empty row count is
  equally true of a legitimately empty node, so only this observable sees a
  blank pane. A copy of the selecting conditions agrees with the items whether or
  not they render. An observable naming an item that no longer exists reduces to
  a constant, and the end-to-end assertion consuming it keeps passing while
  unable to fail.

### `module-settings`

- **The settings store holds exactly five named keys.** The reply-count half
  was unpinned, demonstrated rather than suspected: injecting an extra key into
  the store's reply left the unit suite green, because the covering test makes
  five `contains()` assertions and never counts. That gap is issue #64; the
  split-out "getSettings reports the five keys and no other" now carries a
  scenario for it.
- **The settings file lives under the XDG data home.** Basecamp hands each
  profile its own `XDG_DATA_HOME`, so two profiles hold two independent sets of
  settings.
- **Values are validated on write, not on use.** The `radSocket` check is not
  redundant with the resolution-time one (`node-paths` owns why). `radHome` has
  no value validation, which is a known gap, not the intended contract: an
  unusable home is accepted only to surface later as `localAvailable:false`,
  the deferred failure this capability rejects for `gitPath`. That gap is issue
  #56.
- **A seed URL is checked for its shape only** (split out). Whether the seed
  answers is a network round trip, which does not belong on a settings write.
- **A stored mode this build cannot interpret reads back as explore.**
  `setSetting` never writes an unknown mode, but the file lies under the user's
  data directory, and a hand edit, a torn write or a newer build can leave one.
  `explore` touches no local profile; falling back to `local` would claim a node
  identity on a file the build has just admitted it cannot read. Not rewriting
  the file means a build that understands the value loses nothing.
- **A settings write that repoints the module is applied at once.** `gitPath`
  is applied at the next start because the only channel reaching Radicle's `git`
  spawn sites is the process's own `PATH`, which can only be written safely
  before any thread exists.
- **A configured git path is proved to be git, and never falls back.** A silent
  fallback makes a typo look like a Radicle defect, and makes the reported path
  differ from the binary that runs. The four refusal cases are required together
  because no positive check distinguishes them: a test configuring a working git
  passes equally against a resolver that ignores the setting.
- **The reported git path distinguishes configured from detected.** A refused
  configured path still reports `gitConfigured:true`, because the user did set
  something; "found automatically" would describe a state the module is not in.

### `node-config`

- **The node configuration is a separate surface from the module settings.**
  The two describe different things in different files: the module settings
  describe the module and live in `<XDG data home>/radicle-module/settings.json`;
  the node configuration describes the node.
- **A configuration write returns the whole configuration** (split out). A
  caller re-renders from one reply instead of holding a stale view of the fields
  it did not set.
- **The peer discipline is a discipline, and is checked** (split out). `static`
  with an empty `connect` configures a node to maintain connections to nobody,
  which reaches no peers and reports no error at any later point.
- **Addresses are the two spellings the crate distinguishes.** The crate parses
  these as distinct types, so a mismatched value fails at the node's next start,
  the deferred failure validation on write exists to prevent.
- **Inbound is off by default and its cost is stated.** A node with no listen
  address is outbound-only: it can fetch and announce, but peers cannot fetch
  from it.
- **A write preserves the fields it does not set.** This does not hold by
  default: a key no field of this build's configuration type corresponds to,
  such as one a newer `radicle` crate wrote, has to be kept deliberately, so
  that a change of one field never becomes the deletion of another.
- **The configuration read is of the mode's own home.** A home parameter crossing from
  a sandboxed view would let a caller rewrite the configuration of a node the
  module does not own. In `local` the home is the user's own: writing there
  changes a node this module did not create and does not run, while reading is
  what lets a view show it.
- **A home with no configuration is reported as such.** Defaults
  presented as a configuration are indistinguishable on screen from a real one,
  and a view would offer to edit a node that does not exist. Creating a
  configuration is identity creation's job.

### `node-paths`

- **Socket precedence prefers the runtime directory over the home.** The runtime
  directory is short by construction, scoped to the user's session and cleaned
  up on logout. The home-derived form is kept because it is where a hand-run
  `rad` node places its socket. The live requirement also said, as a rule of its
  own, that `local` mode must still find that socket when no runtime directory
  exists; the delta drops that sentence, and Decision 6 argues it and records
  what no scenario pins as a result.
- **The 108-byte sun_path cap constrains the socket alone.** At resolution rather
  than at connection, so a preflight surface can report it without a live
  profile or a running node.
- **The socket setting is validated against the same cap on write.** Not
  redundant with the resolution-time check: a socket can arrive from
  `RAD_SOCKET`, which the settings store never sees, and a resolution-time
  check alone would let a bad value persist.
- **Embedded mode must never resolve the user's own Radicle home.** Embedded
  promises an identity separate from any node the user runs; a resolution that
  could reach the user's profile would expose their signing key and put two
  nodes on one git storage. The scenario "an unrecognised mode resolves
  nothing" holds by composition, not by anything in `node-paths`: it depends on
  `module-settings` sanitising an unknown stored mode to `explore` on read, and
  `explore` resolving no home. A change to that fallback, outside this
  capability entirely, would break the scenario's `MUST NOT`, so whoever changes
  it must re-check this one. It sits in `node-paths` because the resolved home
  is what a reader of this capability is asking about. The trap is also written
  where the next person to change the fallback will be standing: beside it in
  `SettingsStore::load()` (`radicle/src/settings_store.cpp`), naming both
  scenarios that hold through it and the test that sees the second,
  `a_persisted_mode_this_build_does_not_know_does_not_alias_the_local_profile`.
- **The absent-profile sentence is chosen by the mode that asked.** It
  outranks the empty-home default too, because the case most needing a
  mode-specific sentence, embedded with no data directory, is the case where the
  home is empty. The `rad auth` advice is right for a home the user manages and
  wrong for the embedded one, whose premise is that `rad auth` is never run and
  whose empty home is the state before the wizard has run.
- **The node is running only when its socket accepts a connection.** A socket
  file can outlive the daemon. Two properties here are about a syscall not being
  made — no probe for a store with no profile, none for a path over the cap —
  and neither is observable through this interface, so neither is a
  requirement. They are a matter for review, and for the code comment
  explaining why a truncated path must not reach `connect()`.

### `node-seeding`

- **Seeding is a policy, not a question about stored repositories.**
  `localListRepos("seeded")` reports repositories in storage that this node
  neither delegates nor that are private. The two disagree in both directions,
  and both are ordinary: a repository can be seeded with nothing replicated yet,
  and one can sit in storage with no policy seeding it.
- **Seeding takes a scope and the scope is reported back.** The scope is not
  cosmetic: it decides whether a private repository replicates at all.
- **Seeding acts on the mode's own node and takes no home.** The same reason as
  the node configuration methods: a home from a sandboxed view is a way to
  rewrite the policies of a node the module does not own.
- **The allow-is-not-enough consequence is stated where it applies.**
  Allow-listing a DID on a repository's identity document does not make the
  repository replicate to this node.
- **Every failure crosses the boundary as JSON.** An empty list means "nothing is
  seeded", a different fact from "the policies could not be read"; a view
  acting on the first while the second is true would offer to seed a repository
  that is already seeded.

### `source-modes`

- **The reason a profile is absent fits the mode that asked.** `rad auth` is the
  fix for a home the user manages, and `embedded` is a mode whose premise is
  that the user never runs it. An absence with no explanation is
  indistinguishable on screen from a blank pane.
- **Startable means a mode resolves a workable home.** That all three are
  startable on this build is a fact about what it reports, not a licence for a
  view to assume it: the component tests construct sets omitting `embedded`,
  because a control correct only for the set that ships breaks silently when a
  mode stops resolving. Simplifying a startability check to a constant on the
  strength of that sentence is the error it must not invite. Whether a daemon
  answers is a separate question with a separate lifetime.
- **An unstartable mode is offered and says so before it is chosen.** Marking on
  membership rather than selection makes the caveat legible while the user
  decides.
- **A deriving component reads an empty set as not yet reported** (split out).
  So that no not-implemented state is shown before the first capabilities reply
  arrives.
- **The method prefix is derived from the mode.** A separately stored prefix
  could drift from the mode. Naming every mode that derives `local`, rather
  than every mode that derives `remote`, is what sends an unrecognised mode to
  `remote`, the prefix that touches no local profile.
- **getCapabilities reports the mode and its consequences.**
  `modeUnavailableReason` is empty in every state a test can reach on this
  build, since every known mode is startable and an unknown stored mode
  resolves to `explore`. It is written as a conditional because the view layer
  consumes it for sets the backend does not report, and a fourth mode makes it
  live with no edit. Whether its sentence-building code should stay is issue
  #62.
- **A mode that cannot start asks its node for nothing.** Startability is not
  the test, and treating it as one is how this surface was lost: the guard and
  the explanation it protected were keyed on the same condition, so both went
  away together when `embedded` became startable, leaving a request against a
  home with no identity and a refusal rendered as an error banner. A request
  issued and then hidden still arrives and repopulates the screen. Deriving the
  decision from what the backend reports, not a mode's name, means a mode
  gaining a node changes this with no view edited. The third clause of the
  scenario "An unprovisioned embedded home is not reported as empty", that the
  "no repositories" wording is not shown, was once unpinned: the covering
  test asserted only that the pane was not blank, which an error shown
  alongside that wording also satisfies. `embedded-state`'s "The no-repositories
  wording is withheld where no node exists" now states it as a requirement of
  its own, with scenarios that tell the two apart, so the gap is closed there
  rather than annotated here.

## Risks / Trade-offs

- **[The gate cannot see a capability directory with no `spec.md`, or an empty
  `openspec/specs/`]** → Stated beside the step and in
  `docs/OPENSPEC-ARCHIVE.md`; whether the gate must guard it is Decision 3's
  open question, tracked by issue #99.
- **[A declined archive promotion still passes]** → It leaves every live spec
  valid, so the gate is green exactly when the contract is missing. Unaddressed
  here and tracked by issue #100; catching it needs a check that every
  capability an archived change's `specs/` names carries that change's
  requirements in `openspec/specs/`.
- **[A `MODIFIED` requirement that shrinks but stays over 500 characters passes
  `validate <change> --strict`]** → The CLI flags a `MODIFIED` only when it is
  longer than the text it replaces, so the first signal is this gate going red
  after the archive commit. Recorded in `docs/OPENSPEC-ARCHIVE.md`, and as a
  rule in the overlay's `## spec-writer`, where the writer reads it before the
  requirement is written.
- **[Reasoning under the limit, or after the scenarios, passes]** → The rule
  measures length, not content. Review is the only check.
- **[A behaviour dropped while splitting a requirement passes]** → The
  validator cannot tell a lost `MUST`. That is the spec-test and design
  reviewers' read, against `git diff --no-index` of each pair. The one drop
  this change makes on purpose is Decision 6's.
- **[No new test has an observed failure]** → No pod that worked on this change
  had a Qt 6 `qmltestrunner`, a C compiler or Nix, so every run of the new
  tests is CI's, and a CI pass shows they pass, not that they can fail. The
  owner accepted CI green as the evidence and deferred the mutations. The ones
  worth running where a toolchain exists, each in the single-file form and
  restored after: `SourceState.qml`'s `modeHasIdentity` keyed on
  `mode === "local"` instead of `current === "local"`, against `tst_source.qml`
  (predicted red on `embedded`); `SetupFlow.qml` no longer waiting for the
  capabilities reply before the resume lands, against `tst_setup_wizard.qml`
  (predicted red at the withheld-capabilities test's first
  `compare(flow.step, "preflight")`); and `EmbeddedState.qml` keying the
  prompt on `canWrite` instead of `encrypted`, against
  `tst_embedded_panel.qml` (predicted red at
  `test_a_key_that_cannot_sign_is_not_taken_for_an_encrypted_one`).
- **[The pinned CLI ages]** → A rule a newer CLI enforces passes CI; bump the
  pin deliberately. The `closer`'s local `validate <change> --strict` runs on
  whatever CLI the agent has, so a newer rule tends to surface there first.
- **[An npm registry outage turns `Lint` red]** → npm's error is in the step log.
- **[A release of one of the CLI's ranged dependencies moves a verdict, or is
  compromised and runs in `Lint`]** → Accepted, as for `ui-specs` (Decision 2);
  a lockfile is the remedy if it bites.

## Migration Plan

**This PR's `Lint` is red until the `closer`'s archive commit**, and that is
expected: the step reads `openspec/specs/` as the branch holds it, and the
delta that brings it under the limit is promoted only by `archive`. The first
run shows the gate failing on the length rule alone; the run after the archive
commit shows it passing. Both are the evidence that the step can fail and does
pass.

Rollback is reverting the merge commit, which takes the step and the
promoted specs out together.
