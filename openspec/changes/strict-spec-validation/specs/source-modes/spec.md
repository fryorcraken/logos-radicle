## MODIFIED Requirements

### Requirement: The three modes and what each resolves

The module MUST recognise exactly three modes, persisted as `explore`, `local`
and `embedded`, each resolving its own Radicle home independently. `explore`
MUST resolve no home. `local` MUST resolve the home the environment or the
`radHome` setting names. `embedded` MUST resolve the module's own home under the
Basecamp profile's XDG data directory, never one reachable from `RAD_HOME` or
`$HOME`.

A mode MUST NOT fall through to another's resolution: one that resolves no
home MUST report none.

#### Scenario: Local reports the environment's profile

- **WHEN** a readable Radicle profile exists at the home the environment names
  and the mode is set to `local`
- **THEN** `getCapabilities().radHome` MUST equal that home
- **AND** `getCapabilities().localAvailable` MUST be true

#### Scenario: Explore reports no home even when a profile exists

- **GIVEN** the same environment, in which `local` reports a non-empty
  `radHome` and `localAvailable: true`
- **WHEN** the mode is set to `explore`
- **THEN** `getCapabilities().radHome` MUST be the empty string
- **AND** `getCapabilities().localAvailable` MUST be false

#### Scenario: Embedded reports its own home, not the user's

- **GIVEN** the same environment, in which `local` reports the environment's
  profile as `radHome` with `localAvailable: true`
- **WHEN** the mode is set to `embedded`
- **THEN** `getCapabilities().radHome` MUST be non-empty and MUST be a path
  under the Basecamp profile's XDG data directory
- **AND** it MUST NOT equal the home `local` reported
- **AND** `getCapabilities().localAvailable` MUST be false while no identity
  has been created in that home
- **AND** `getCapabilities().canWriteLocal` MUST be false

#### Scenario: Embedded with no resolvable data directory is inert

- **GIVEN** neither `XDG_DATA_HOME` nor `HOME` is set, while `RAD_HOME` names a
  readable profile
- **WHEN** the mode is `embedded`
- **THEN** `getCapabilities().radHome` MUST be the empty string, and in
  particular MUST NOT be the profile `RAD_HOME` names
- **AND** `getCapabilities().localAvailable` MUST be false
- **AND** `getCapabilities().canWriteLocal` MUST be false
- **AND** `getCapabilities().writeUnavailableReason` MUST be non-empty, MUST
  name the environment variables that would give the module a data directory,
  MUST NOT advise running `rad auth`, and MUST NOT contain a path fragment left
  dangling by an empty home

### Requirement: The reason a profile is absent fits the mode that asked

When the mode in force resolves a home holding no profile, the reason reported
MUST be worded for that mode, and MUST never be empty.

In `local` it MUST advise installing Radicle and running `rad auth`. In
`embedded` it MUST NOT mention `rad auth`, and MUST instead name the embedded
home and state that it is a separate identity from any node the user already
runs.

#### Scenario: Embedded with no identity yet is explained in its own words

- **GIVEN** a readable profile exists at the home the environment names, so
  `local` reports `localAvailable: true`
- **WHEN** the mode is set to `embedded`, whose home holds no identity
- **THEN** `getCapabilities().writeUnavailableReason` MUST be non-empty
- **AND** it MUST NOT contain `rad auth`
- **AND** it MUST contain the value of `getCapabilities().radHome`
- **AND** it MUST state that this is a separate identity

#### Scenario: Local with no profile still gets the rad auth advice

- **GIVEN** the mode is `local` and the home the environment names holds no
  profile
- **THEN** `getCapabilities().localAvailable` MUST be false
- **AND** `getCapabilities().writeUnavailableReason` MUST contain `rad auth`

### Requirement: Startable means a mode resolves a workable home

`startableModes()` MUST report every mode this build can start. A mode is
startable when it resolves a home this module can work against, not when a node
daemon runs in it; on this build all three are. A view MUST NOT hard-code the
set's membership.

`localNodeRunning` MUST be free to change while the module runs, and the
startable set MUST NOT. `modeIsStartable(mode)` MUST agree with membership of
the startable set for every mode.

#### Scenario: Every known mode is startable and says so per mode

- **WHEN** the mode is set in turn to `explore`, `local` and `embedded`
- **THEN** `getCapabilities().mode` MUST equal the mode just set, in each case
- **AND** `getCapabilities().modeStartable` MUST be true in each case
- **AND** `getCapabilities().modeUnavailableReason` MUST be the empty string in
  each case

#### Scenario: A mode outside the three is not startable

- **WHEN** `modeIsStartable` is asked about a string that is not one of the
  three modes
- **THEN** it MUST return false
- **AND** that string MUST NOT appear in `getCapabilities().startableModes`

#### Scenario: Startability does not require a running daemon

- **GIVEN** an `embedded` home with no identity created in it and no node
  daemon listening on any control socket
- **THEN** `getCapabilities().modeStartable` MUST be true for `embedded`
- **AND** `getCapabilities().localNodeRunning` MUST be false

#### Scenario: localNodeRunning is false wherever no profile is available

- **GIVEN** the mode in force resolves no readable profile, so
  `getCapabilities().localAvailable` is false
- **THEN** `getCapabilities().localNodeRunning` MUST be false regardless of
  what the resolved socket path is

### Requirement: The startable set is reported, not derived from one mode

`getCapabilities()` MUST report `startableModes` as an array equal, element for
element, to `SettingsStore::startableModes()`, and it MUST NOT vary with the
mode in force.

A view that annotates which modes are available MUST consume `startableModes`,
and MUST NOT derive that set from `modeStartable`, which answers only for the
mode in force.

#### Scenario: The reported array matches the store's own answer exactly

- **WHEN** `getCapabilities()` is called
- **THEN** `startableModes` MUST be an array containing `explore`, `local` and
  `embedded`
- **AND** its length MUST equal the length of
  `SettingsStore::startableModes()`
- **AND** it MUST NOT contain any string that is not a known mode

#### Scenario: The set does not move when the selected mode moves

- **WHEN** `getCapabilities().startableModes` is read with the mode set to
  `local`, and read again with the mode set to `embedded`
- **THEN** both arrays MUST be non-empty
- **AND** the two MUST be equal

#### Scenario: A picker annotates a row from the set, not from the boolean

- **GIVEN** a mode picker rendering one row per mode, told a startable set that
  omits one mode while the mode in force is a different, startable one
- **THEN** the omitted mode's row MUST show its unavailable annotation
- **AND** the rows for the modes in the set MUST NOT show one

#### Scenario: Annotation follows the set when a mode becomes startable

- **GIVEN** a picker told a startable set omitting `embedded`, which therefore
  annotates the `embedded` row as unavailable
- **WHEN** the same picker is told a startable set that includes `embedded`,
  with nothing else changed
- **THEN** the `embedded` row MUST NOT show the unavailable annotation

### Requirement: An unstartable mode is offered and says so before it is chosen

Every mode MUST be selectable, including one the startable set omits, and MUST
NOT be hidden or silently inert when chosen. A control offering the modes MUST
mark each mode the set it was given omits, on that mode's own row or segment,
keyed on membership rather than on selection, for any set. An absent set
(`undefined`) MUST mark nothing; a reported empty array MUST mark every mode; a
populated array MUST mark exactly the modes it omits.

#### Scenario: Only the unstartable mode is marked

- **GIVEN** a segmented control told a startable set containing `explore` and
  `local` but not `embedded`
- **THEN** the `embedded` segment's unavailable marker MUST be visible
- **AND** the `explore` and `local` segments' markers MUST NOT be visible

#### Scenario: An unstartable mode can still be selected

- **GIVEN** a segmented control whose startable set omits `embedded`
- **WHEN** the `embedded` segment is clicked
- **THEN** the control MUST report `embedded` as the mode chosen

#### Scenario: Nothing is marked before the startable set is known

- **GIVEN** a segmented control whose startable set has not been reported —
  the value is `undefined` rather than an array
- **THEN** every segment's unavailable marker MUST NOT be visible, including
  the marker for a mode a reported set would have omitted

#### Scenario: A reported empty set marks every mode

- **GIVEN** a segmented control told a startable set that is an empty array
- **THEN** every segment's unavailable marker MUST be visible

#### Scenario: A deriving component reads an empty set as not yet reported

- **GIVEN** a component that derives only whether the mode in force is
  startable, told an empty startable set and a mode of `local`
- **THEN** its derived startable flag MUST be true
- **AND** when it is told a populated set omitting `local`, that flag MUST
  become false

### Requirement: The method prefix is derived from the mode

The `remote*` / `local*` method prefix a view calls MUST be derived from the
mode, and MUST NOT be a separately stored or separately chosen value.

`local` and `embedded` MUST both derive `local`, and `explore` MUST derive
`remote`; a mode the derivation does not recognise MUST derive `remote`. The
prefix MUST NOT be treated as a proxy for the mode.

#### Scenario: Each mode derives its prefix

- **WHEN** the mode is `explore`
- **THEN** the derived prefix MUST be `remote`
- **AND** when the mode is `local` the derived prefix MUST be `local`
- **AND** when the mode is `embedded` the derived prefix MUST be `local`

#### Scenario: An unrecognised mode derives the seed prefix

- **WHEN** the mode is a string the view does not recognise
- **THEN** the derived prefix MUST be `remote`

#### Scenario: Before the mode is known the prefix is the seed's

- **GIVEN** a view that has not yet been told the mode in force
- **THEN** the derived prefix MUST be `remote`

#### Scenario: The prefix applies to every method, not only listing

- **WHEN** a method name is built for a suffix other than `ListRepos` while the
  mode is `local`
- **THEN** the resulting method name MUST carry the `local` prefix
- **AND** an explicit source override passed by the caller MUST take precedence
  over the derived prefix

#### Scenario: Switching between two modes sharing a prefix is still a switch

- **GIVEN** the mode in force is `local` and a view has loaded from it
- **WHEN** the mode in force becomes `embedded`, so the derived prefix is
  unchanged
- **THEN** the settled notification MUST have been emitted exactly once
- **AND** when the mode in force returns to `local` it MUST be emitted once
  more, so the user's own node does not render as an empty one

#### Scenario: A switch loads from the mode now in force, not the one clicked

- **GIVEN** a view showing repositories fetched under one mode
- **WHEN** a different mode is picked and the mode in force subsequently
  changes to it
- **THEN** the repositories shown MUST be those the new mode's node returned
- **AND** a reply to a request issued under the previous mode, arriving after
  the switch, MUST be discarded rather than rendered

### Requirement: Picking a mode persists it and the backend is the authority

Picking a mode MUST persist it through `setSetting("mode", …)`, and the control
that offered the choice MUST NOT record its own copy of the mode in force: what
a view displays as the mode in force MUST come from `getCapabilities().mode`. A
pick of the mode already in force MUST persist nothing, and MUST leave the
control able to accept the next pick.

Changing `mode`, `radHome` or `radSocket` MUST repoint the module without a
restart, and MUST announce the new capabilities.

#### Scenario: Clicking a segment persists that mode

- **WHEN** a segment naming a mode other than the one in force is clicked
- **THEN** exactly one `setSetting` write MUST be issued, for key `mode` with
  that mode as its value

#### Scenario: Clicking the mode already in force writes nothing

- **WHEN** the segment for the mode already in force is clicked
- **THEN** a `setSetting` write MUST NOT be issued
- **AND** a subsequent click on a different segment MUST still issue its write

#### Scenario: A click during an in-flight write is not lost

- **GIVEN** a segment has been clicked and its `setSetting` write has not yet
  been answered
- **WHEN** a second, different segment is clicked
- **THEN** two `setSetting` writes MUST have been issued
- **AND** once both are answered, the mode in force MUST be the one clicked
  second

#### Scenario: Selecting an unknown mode is refused before it is persisted

- **WHEN** a mode selection is requested for a string that is not one of the
  three modes
- **THEN** the selection MUST be refused
- **AND** the change notification MUST NOT be emitted
- **AND** the mode the view reports as in force MUST be unchanged

#### Scenario: Changing the mode repoints the module at once

- **GIVEN** the mode is `local` and `getCapabilities().radHome` reports the
  environment's profile
- **WHEN** `setSetting("mode", "embedded")` succeeds
- **THEN** the next `getCapabilities()` MUST report the embedded home, without
  the module being restarted
- **AND** a local-availability change MUST have been announced

### Requirement: getCapabilities reports the mode and its consequences

`getCapabilities()` MUST report, among its fields, `mode` (one of the three,
whatever is on disk), `modeStartable`, `startableModes`,
`modeUnavailableReason`, `radHome`, `radSocket`, `pathsProblem`,
`localAvailable`, `localNodeRunning`, `canWriteLocal` and
`writeUnavailableReason`. `modeUnavailableReason` MUST be present, and empty
whenever `modeStartable` is true.

`getCapabilities()` MUST NOT fail: a probe that cannot answer MUST be reported
as a reason a capability is absent.

#### Scenario: The mode fields are all present and consistent

- **WHEN** `getCapabilities()` is called in any mode
- **THEN** the reply MUST contain `mode`, `modeStartable`, `startableModes`,
  `modeUnavailableReason`, `radHome`, `radSocket` and `pathsProblem`
- **AND** `mode` MUST be one of `explore`, `local`, `embedded`
- **AND** `modeUnavailableReason` MUST be the empty string whenever
  `modeStartable` is true

#### Scenario: Switching modes changes the reported home

- **GIVEN** `getCapabilities().radHome` read with the mode set to `local`,
  pointed at a readable profile
- **WHEN** the mode is set to `embedded` and `getCapabilities()` is called
  a second time
- **THEN** the two reported `radHome` values MUST differ
- **AND** both `radSocket` and `pathsProblem` MUST be present in each reply

### Requirement: A mode that cannot start asks its node for nothing

A view MUST NOT issue a request against a mode with no node able to answer it:
one the startable set omits, or a startable one whose node does not exist or is
not loaded. It MUST NOT issue the request and then hide the reply.

While declining, it MUST render why, and MUST NOT render wording claiming the
mode's node exists and holds nothing. The decision MUST be derived from what the
backend reports, not from a particular mode's name.

#### Scenario: An unstartable mode issues no list request

- **GIVEN** a repository list in a mode the reported startable set omits
- **WHEN** the list loads
- **THEN** a list request MUST NOT be issued to the backend
- **AND** an explanation MUST be visible
- **AND** the "no repositories" wording MUST NOT be visible

#### Scenario: A startable mode fetches and shows what it fetched

- **GIVEN** the same repository list, in a mode the reported startable set
  contains, whose node exists and is serving, and whose backend answers with
  repositories named after that mode
- **WHEN** the list loads
- **THEN** a list request MUST be issued
- **AND** the rows shown MUST be the ones that mode's node returned
- **AND** the explanation MUST NOT be visible

#### Scenario: The declining state follows the set, not a mode name

- **GIVEN** a repository list showing the explanation because the reported
  startable set omits the mode in force
- **WHEN** the reported startable set is changed to include that mode, and that
  mode's node is reported as existing and serving
- **THEN** the explanation MUST NOT be visible

#### Scenario: A startable mode whose node does not exist is asked nothing

- **GIVEN** a repository list in `embedded`, which the reported startable set
  contains, where `getEmbeddedIdentity().exists` is false
- **WHEN** the list loads
- **THEN** a list request MUST NOT be issued to the backend
- **AND** an explanation MUST be visible
- **AND** the "no repositories" wording MUST NOT be visible

#### Scenario: An unprovisioned embedded home is not reported as empty

- **GIVEN** the mode is `embedded`, the mode is startable, and no identity
  exists in the embedded home yet
- **THEN** no repositories MUST be listed
- **AND** the pane MUST NOT be blank — it MUST carry rows, a message or an
  error, so there is something for the user to act on
- **AND** the "no repositories" wording MUST NOT be shown in its place, since
  a home with no identity is not a node that exists and holds nothing
- **AND** no error banner MUST be raised, since nothing was asked

The third clause of that scenario was recorded here as unpinned by any test: the
covering test asserted only that the pane was not blank, which an error
*alongside* the "no repositories" wording would satisfy. `embedded-state` now
states it as a requirement in its own right, with scenarios that distinguish the
two, so the gap closes there rather than remaining an annotation here.

## ADDED Requirements

### Requirement: A deriving component reads an empty set as not yet reported

A component that only derives whether the mode in force is startable, and
annotates nothing itself, MUST treat an empty startable set as not yet reported
and reduce to startable.

#### Scenario: An empty set derives startable for Embedded too

- **GIVEN** a component that derives only whether the mode in force is
  startable, told an empty startable set and a mode of `embedded`
- **THEN** its derived startable flag MUST be true
- **AND WHEN** it is told a populated set omitting `embedded`
- **THEN** that flag MUST be false
