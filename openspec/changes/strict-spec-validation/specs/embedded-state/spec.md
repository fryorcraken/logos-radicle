## MODIFIED Requirements

### Requirement: Embedded's empty surface reports one of seven states

While the mode in force is `embedded` and no repositories are listed, the
repository list MUST render exactly one state: the first, in the order blocked,
no identity, stopped, starting, start failed, not serving, running and empty,
whose condition holds. It MUST recompute the state from `getCapabilities()`,
`getEmbeddedIdentity()` and `getNodeStatus()`, and MUST NOT hold it as a stored
value. The blocked state's sentence MUST be the `pathsProblem` sentence
verbatim.

#### Scenario: Each state renders its own sentence and its own action

- **GIVEN** a repository list in `embedded` with a non-empty `pathsProblem`
- **THEN** the rendered sentence MUST contain that `pathsProblem` text verbatim
- **AND WHEN** the same list is instead told an empty `pathsProblem`, a
  resolving home and `getEmbeddedIdentity().exists` false
- **THEN** the rendered sentence MUST NOT contain the earlier `pathsProblem`
  text
- **AND** the rendered action MUST differ from the action rendered for the
  blocked state

#### Scenario: A stopped node and a home with no identity are different states

- **GIVEN** a repository list in `embedded` told `exists:false`
- **THEN** the rendered action MUST be the one that opens the guided setup
- **AND WHEN** the same list is told `exists:true` with `encrypted:true` and a
  node reporting `running:false` and `serving:false`
- **THEN** the rendered action MUST be the one that starts the node
- **AND** the rendered sentence MUST differ from the one rendered for
  `exists:false`

#### Scenario: A blocked home offers no action that would write

- **GIVEN** a repository list in `embedded` with a non-empty `pathsProblem`
- **THEN** no action MUST be named
- **AND** no enabled action MUST be rendered
- **AND WHEN** the same list is told an empty `pathsProblem` with a resolving
  home and `exists:false`
- **THEN** an enabled action MUST be rendered

#### Scenario: The most fundamental obstacle is the one rendered

- **GIVEN** a repository list in `embedded` told `encrypted:true`, whose
  `startNode` was answered with a distinctive refusal, and which is then told a
  non-empty `pathsProblem`
- **THEN** the rendered sentence MUST contain the `pathsProblem` text
- **AND** no action MUST be named
- **AND WHEN** the same list is told an empty `pathsProblem`, with the refusal
  unchanged
- **THEN** the rendered sentence MUST contain the refusal's text
- **AND** the named action MUST be the one that starts the node

#### Scenario: The state follows a later reply rather than the first

- **GIVEN** a repository list in `embedded` rendering the stopped state, having
  been told `exists:true` with `running:false` and `serving:false`
- **WHEN** the backend subsequently reports `running:true` with `serving:true`
  and a list reply carrying no repositories
- **THEN** the rendered state MUST be the running-and-empty one
- **AND WHEN** the backend subsequently reports `running:false` and
  `serving:false` again
- **THEN** the rendered state MUST be the stopped one again

### Requirement: A mode with no node to ask issues no list request

The repository list MUST NOT issue a `ListRepos` request while the mode in force
has no node that could answer it — in `embedded`, the blocked, no identity,
stopped and start failed states — and MUST NOT issue it and then discard or
hide the reply. The guard MUST be keyed on whether the mode has a node to ask,
not on whether it is startable. In the starting, not serving and running and
empty states a request MUST be permitted.

#### Scenario: No request is issued while there is no node to ask

- **GIVEN** a repository list in `embedded` told `exists:false`
- **WHEN** the list loads
- **THEN** no `ListRepos` request MUST have been issued
- **AND WHEN** the same list is instead told `exists:true` with a node
  reporting `serving:true`, whose backend answers with a distinctively named
  repository
- **THEN** a `ListRepos` request MUST have been issued
- **AND** that repository MUST be listed

#### Scenario: The guard is not keyed on startability

- **GIVEN** a repository list in `embedded`, told a startable set containing
  `embedded`, and told `exists:false`
- **WHEN** the list loads
- **THEN** no `ListRepos` request MUST have been issued

#### Scenario: A stopped node is asked nothing, a non-serving one is asked

- **GIVEN** a repository list in `embedded` told `exists:true` with a node
  reporting `running:false` and `serving:false`
- **WHEN** the list loads
- **THEN** no `ListRepos` request MUST have been issued
- **AND WHEN** the same list is told `exists:true` with a node reporting
  `running:true` and `serving:false`
- **THEN** a `ListRepos` request MUST have been issued

#### Scenario: A refused start leaves the node unasked

- **GIVEN** a repository list in `embedded` told `exists:true`, whose
  `startNode` was answered with `{"error":"..."}` and whose node still reports
  `running:false` and `serving:false`
- **WHEN** the list loads
- **THEN** no `ListRepos` request MUST have been issued
- **AND** the empty-list placeholder MUST NOT be visible

### Requirement: The no-repositories wording is withheld where no node exists

The empty-list placeholder, which reports a node holding no repositories, MUST
NOT be rendered in the blocked, no identity, stopped, start failed or starting
states.

It MUST be permitted in the running and empty state, and the sentence rendered
there MUST also say that this node lists what it is seeding.

#### Scenario: The empty-list wording is absent where no node was asked

- **GIVEN** a repository list in `embedded` told `exists:false`
- **THEN** the empty-list placeholder MUST NOT be visible
- **AND WHEN** the same list is told `exists:true` with a node reporting
  `running:false` and `serving:false`
- **THEN** the empty-list placeholder MUST NOT be visible

#### Scenario: A serving node that holds nothing may say so

- **GIVEN** a repository list in `embedded` told `exists:true` with a node
  reporting `serving:true`, whose `ListRepos` reply carries no repositories
- **THEN** the rendered sentence MUST state that this node lists what it is
  seeding

### Requirement: A node that has stopped serving reads as neither running nor stopped

When the node reports `running:true` with `serving:false` and no start is
outstanding, the surface MUST report that the node has stopped answering its
control socket while remaining loaded, and MUST NOT report it as running or as
stopped.

The action offered MUST be one that stops and then starts the node, rather than
a bare start.

#### Scenario: A non-serving node is reported as neither running nor stopped

- **GIVEN** a repository list in `embedded` told `exists:true` with a node
  reporting `running:true` and `serving:false`, and no start outstanding
- **THEN** the rendered sentence MUST state that the node has stopped answering
- **AND** the rendered sentence MUST state that it is still loaded
- **AND** the sentence rendered for a node reporting `serving:true` MUST differ
  from it
- **AND** the sentence rendered for a node reporting `running:false` and
  `serving:false` MUST differ from it

#### Scenario: The offered action restarts rather than starts

- **GIVEN** a repository list in `embedded` told `running:true` and
  `serving:false` with no start outstanding
- **THEN** the rendered action MUST differ from the action rendered when the
  node reports `running:false` and `serving:false`

### Requirement: Starting is distinguished from not serving by the outstanding call

A node being started and a node that has stopped serving MUST be rendered as
different states, told apart by whether a `startNode` call the view issued is
outstanding: with the node reporting `running:true` and `serving:false`, the
state MUST be starting while one is and not serving while none is. An
outstanding start MUST be cleared by its reply, success or `{"error":"..."}`,
not by being issued. While starting, the surface MUST NOT name the start action.

#### Scenario: The same reported fields render two different states

- **GIVEN** a repository list in `embedded` that has issued a `startNode` call
  which has not been answered, with the node reporting `running:true` and
  `serving:false`
- **THEN** the rendered sentence MUST state that the node is starting
- **AND WHEN** the same fields are reported with no `startNode` call
  outstanding
- **THEN** the rendered sentence MUST state that the node has stopped answering
- **AND** the two sentences MUST differ

#### Scenario: The reply clears the outstanding start, not the request

- **GIVEN** a repository list in `embedded` that has issued a `startNode` call
  which has not been answered
- **THEN** the state MUST be the starting one
- **AND WHEN** the call is answered with `{"error":"..."}`
- **THEN** the state MUST NOT be the starting one

#### Scenario: The start action is withheld while a start is outstanding

- **GIVEN** a repository list in `embedded` that has issued a `startNode` call
  which has not been answered
- **THEN** no action MUST be named
- **AND WHEN** the call is answered with `{"error":"..."}`
- **THEN** the named action MUST be the one that starts the node

### Requirement: A refused start is displayed as the backend worded it

When a `startNode` call is answered with `{"error":"..."}`, the surface MUST
display that message as the backend worded it, and MUST go on naming the action
that starts the node.

A refused start MUST NOT be rendered as a running node, nor as the starting
state. A subsequent successful start MUST clear the displayed message.

#### Scenario: Two different refusals display two different messages

- **GIVEN** a repository list in `embedded` whose `startNode` is answered with
  one distinctive message
- **THEN** that message MUST be displayed
- **AND** the named action MUST be the one that starts the node
- **AND WHEN** a further start is answered with a different distinctive message
- **THEN** the second message MUST be displayed and the first MUST NOT

#### Scenario: A success clears a displayed refusal

- **GIVEN** a repository list in `embedded` displaying a start refusal
- **WHEN** a subsequent `startNode` is answered reporting the node started
- **THEN** no refusal message MUST be displayed

### Requirement: The surface offers setup without performing it

In the no identity state the surface MUST offer an action that opens the guided
setup, and MUST state before it is taken that the embedded node runs as a new
identity this module creates, separate from any Radicle node the user already
runs.

Taking the action MUST only request that the setup be opened. Neither taking it
nor rendering the state MUST create an identity, start a node or write the mode.

#### Scenario: Rendering the state writes nothing

- **GIVEN** a repository list in `embedded` told `exists:false`
- **WHEN** the no-identity state is rendered
- **THEN** no `createEmbeddedIdentity` call MUST have been issued
- **AND** no `startNode` call MUST have been issued
- **AND** no `setSetting` call MUST have been issued

#### Scenario: The action requests setup rather than performing it

- **GIVEN** a repository list in `embedded` told `exists:false`
- **WHEN** the action that opens the guided setup is invoked
- **THEN** the request to open the setup MUST have been emitted exactly once
- **AND** no `createEmbeddedIdentity` call MUST have been issued

#### Scenario: The separate-identity consequence is stated before setup is opened

- **GIVEN** a repository list in `embedded` told `exists:false`
- **THEN** the rendered sentence MUST state that the node runs as a new
  identity, separate from any Radicle node the user already runs

### Requirement: An unencrypted identity starts its node without being asked

While the mode in force is `embedded`, the state is stopped and
`getEmbeddedIdentity()` reports `encrypted:false`, the surface MUST issue
`startNode` with an empty passphrase without the user acting. It MUST issue it
once per arrival in the stopped state, not again while it is outstanding or
after it is answered, until the state has left stopped and returned. It MUST NOT
start the node unasked in any other state.

#### Scenario: A stopped node with an unencrypted key starts by itself

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:false`, and a node reporting `running:false` and `serving:false`
- **WHEN** the state is rendered, with no action taken
- **THEN** exactly one `startNode` call MUST have been issued
- **AND** its passphrase argument MUST be the empty string
- **AND** the rendered state MUST be the starting one

#### Scenario: The automatic start is issued once, not on every reading

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:false` and a node reporting `running:false` and `serving:false`,
  whose `startNode` reply is withheld
- **WHEN** the backend reports the same node status a second and a third time
- **THEN** exactly one `startNode` call MUST have been issued

#### Scenario: A refused automatic start is reported as a refusal

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:false` and a stopped node, whose `startNode` is answered with a
  distinctive message
- **THEN** the rendered state MUST be the start-failed one
- **AND** that message MUST be displayed

#### Scenario: No automatic start where there is nothing to start

- **GIVEN** a repository list in `embedded` told `exists:false`
- **THEN** no `startNode` call MUST have been issued
- **AND WHEN** the same list is instead told a non-empty `pathsProblem`
- **THEN** no `startNode` call MUST have been issued
- **AND WHEN** the same list is instead told `exists:true` with
  `encrypted:false` and a node reporting `running:true` with `serving:false`
- **THEN** no `startNode` call MUST have been issued

### Requirement: An encrypted identity is asked for its passphrase, not sent to a flow

While the mode in force is `embedded`, the state is stopped and
`getEmbeddedIdentity()` reports `encrypted:true`, the surface MUST offer a
passphrase field on this surface, not a step of the setup or a flow of its own,
and a control that starts the node with what was typed. It MUST NOT start the
node unasked. A refused start MUST leave the field available, with the refusal
displayed as the backend worded it.

#### Scenario: An encrypted identity prompts rather than starting

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:true`, and a node reporting `running:false` and `serving:false`
- **THEN** no `startNode` call MUST have been issued
- **AND** a field for a passphrase MUST be present
- **AND** the request to open the guided setup MUST NOT have been emitted

#### Scenario: The same node status behaves differently on the encrypted field

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:true` and a node reporting `running:false` and `serving:false`
- **THEN** no `startNode` call MUST have been issued
- **AND WHEN** the same list is told the identical node status with
  `encrypted:false`
- **THEN** exactly one `startNode` call MUST have been issued

#### Scenario: The typed passphrase is the one submitted

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:true` and a stopped node, with `correct horse battery` typed into
  the passphrase field
- **WHEN** the control that starts the node is invoked
- **THEN** exactly one `startNode` call MUST have been issued
- **AND** its passphrase argument MUST be `correct horse battery`

#### Scenario: A refused passphrase can be corrected

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:true` and a stopped node, whose `startNode` is answered with a
  distinctive message
- **WHEN** a passphrase is submitted
- **THEN** that message MUST be displayed
- **AND** the passphrase field MUST still be present
- **AND WHEN** a second passphrase is submitted against a backend that now
  reports the node started
- **THEN** that message MUST NOT be displayed

#### Scenario: The passphrase does not outlive its call

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:true` and a stopped node
- **WHEN** a passphrase is submitted and the reply reporting the node started
  has arrived
- **THEN** the surface MUST NOT retain that passphrase

### Requirement: A node this module started is not reported as one in the way

A node answering the resolved socket MUST be reported as an obstacle only when
this module did not start it. When the module's own start put it there, the
surface MUST report the node as running, and MUST NOT report contention for the
socket. The two MUST be told apart by whether a `startNode` call it issued was
answered with success, not by node status alone. A node found serving when the
module started none MUST still be reported as such.

#### Scenario: A node this surface started reads as running, not as contention

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:false` and a stopped node, whose `startNode` is answered reporting
  the node started, and whose node then reports `serving:true`
- **THEN** the rendered text MUST NOT state that a node is already answering on
  the socket
- **AND** the rendered text MUST NOT state that starting a second node would
  contend for it

#### Scenario: A node the surface did not start is still reported as found

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:true` and a node reporting `serving:true`, with no `startNode`
  call ever issued by this surface
- **THEN** the rendered text MUST state that a node is already answering on the
  socket
- **AND** no `startNode` call MUST have been issued

### Requirement: Only an action something can carry out is offered as enabled

The surface MUST render an action as enabled only when the request it emits
reaches something able to carry it out; otherwise it MUST NOT be enabled, and
the surface MUST say why. The setup and start requests reach a host; a restart
request does not, so the not serving state MUST name its action while leaving it
not enabled, and MUST state that restarting is not yet available from here, not
that it failed or cannot be done. This MUST be keyed on whether the request is
hosted.

#### Scenario: An unhosted action is named but not enabled, and says so

- **GIVEN** a repository list in `embedded` told `exists:true` with a node
  reporting `running:true` and `serving:false` and no start outstanding
- **THEN** the named action MUST be the one that restarts the node
- **AND** it MUST NOT be enabled
- **AND** the rendered text MUST state that restarting the node is not yet
  available from here
- **AND WHEN** the same list is told `exists:false`
- **THEN** the named action MUST be the one that opens the guided setup
- **AND** it MUST be enabled
- **AND** the rendered text MUST NOT state that it is unavailable

#### Scenario: Hosting an action is what enables it

- **GIVEN** a repository list in `embedded` told `exists:true` with a node
  reporting `running:true` and `serving:false`, and told that the request to
  restart a node reaches nobody
- **THEN** the named action MUST NOT be enabled
- **AND WHEN** the same list is told that the request to restart a node reaches
  a host, with nothing else changed
- **THEN** the named action MUST be enabled
- **AND** the rendered text MUST NOT state that it is unavailable

#### Scenario: An action that is not enabled emits no request

- **GIVEN** a repository list in `embedded` told `exists:true` with a node
  reporting `running:true` and `serving:false` and no start outstanding
- **WHEN** the rendered action is taken
- **THEN** no request MUST have been emitted
- **AND** no `stopNode` call MUST have been issued

### Requirement: The blank-pane assertion keeps a live referent

The module MUST expose an observable that is true exactly when the repository
list is showing no rows, no placeholder and no state panel.

It MUST be computed from the rendered items' own visibility rather than from a
copy of the conditions those items are keyed on, and replacing an item it reads
MUST carry the observable to the replacement.

#### Scenario: The blank-pane observable is false whenever a state is rendered

- **GIVEN** a repository list in `embedded` with no rows, in each of the
  blocked, no-identity, stopped, starting, start-failed and not-serving states
  in turn
- **THEN** the blank-pane observable MUST be false in each case

#### Scenario: The observable follows the rendered item, not a recomputed copy

- **GIVEN** a repository list in `embedded` with no rows, whose state panel is
  prevented from rendering while the conditions selecting that state are
  unchanged
- **THEN** the blank-pane observable MUST be true

## ADDED Requirements

### Requirement: Each embedded state has one condition

Each embedded state MUST be selected by its condition:

- **blocked** — a non-empty `pathsProblem`, or an empty
  `getEmbeddedIdentity().home`;
- **no identity** — `exists` is false;
- **stopped** — the node reports neither `running` nor `serving`;
- **starting** — a `startNode` the view issued is outstanding;
- **start failed** — the last `startNode` was refused;
- **not serving** — `running:true`, `serving:false`, no start outstanding;
- **running and empty** — `serving:true` and an empty list.

#### Scenario: A home that resolves to nothing is blocked

- **GIVEN** a repository list in `embedded` told an empty `pathsProblem` and a
  `getEmbeddedIdentity()` reply whose `home` is empty
- **THEN** no action MUST be named
- **AND** no `createEmbeddedIdentity` or `startNode` call MUST have been issued

### Requirement: Each state names its own next action

Each state MUST carry its own sentence and its own named action, and one banner
MUST NOT stand in for them. No identity names opening the setup; stopped with an
encrypted key, a passphrase and a start; not serving, a restart; blocked, no
action that writes. Stopped with an unencrypted key names none, and MUST NOT
render a control duplicating the start already issued. A state whose action
nothing can carry out MUST still name it.

#### Scenario: A stopped node with an unencrypted key offers no start control

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:false` and a node reporting `running:false` and `serving:false`,
  whose `startNode` reply is withheld
- **THEN** no control that starts the node MUST be rendered
- **AND** exactly one `startNode` call MUST have been issued

### Requirement: An automatic start is reported as a requested one is

A start the surface issued unasked MUST be reported through the same states as
one a user asked for — starting while it is outstanding, and start failed with
the backend's message displayed when it is refused — and MUST NOT be reported
more quietly for having been automatic.

#### Scenario: A refused automatic start offers a start as a refused one does

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:false` and a stopped node, whose automatic `startNode` is answered
  with a distinctive message
- **THEN** that message MUST be displayed
- **AND** the named action MUST be the one that starts the node

### Requirement: A typed passphrase is neither shown nor kept

The surface MUST NOT display the passphrase as it is typed, and MUST NOT retain
it after the reply to the `startNode` call it was submitted with.

#### Scenario: The passphrase field does not show what is typed

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:true` and a stopped node, with `correct horse battery` typed into
  the passphrase field
- **THEN** the text `correct horse battery` MUST NOT be displayed

### Requirement: Prompting is keyed on the identity's encryption

Whether the surface prompts for a passphrase or starts the node unasked MUST be
derived from the reported `encrypted` field, and MUST NOT be derived from
whether a start has previously failed.

#### Scenario: A refused automatic start does not become a prompt

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:false` and a stopped node, whose automatic `startNode` is answered
  with `{"error":"..."}`
- **THEN** no field for a passphrase MUST be present
