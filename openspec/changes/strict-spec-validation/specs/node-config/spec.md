## MODIFIED Requirements

### Requirement: The node configuration is a separate surface from the module settings

The module MUST expose node configuration through its own pair of methods,
`getNodeConfig` and `setNodeConfig`, and MUST NOT route it through `getSettings`
or `setSetting`. The node configuration is `<radicle home>/config.json`, a file
the node owns and reads at start.

`getSettings` MUST continue to return exactly the five keys the
`module-settings` capability names, with no node-configuration key, and
`setSetting` MUST continue to refuse any key outside those five.

#### Scenario: The settings surface is unchanged by this capability

- **WHEN** `getSettings` is called
- **THEN** the reply MUST contain exactly the five keys `mode`, `radHome`,
  `radSocket`, `gitPath` and `remoteSeed`
- **AND** it MUST NOT contain `alias`, `listen`, `externalAddresses`, `connect`
  or `peers`

#### Scenario: A node-configuration key is not a module setting

- **WHEN** `setSetting` is called with the key `listen`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST contain the text `listen`

### Requirement: The exposed fields are the ones the crate has

`getNodeConfig` MUST report, and `setNodeConfig` MUST accept, exactly these
fields, each spelled as the node's `config.json` spells it:

- `alias`, a string;
- `listen`, the addresses the node binds, an array of strings;
- `externalAddresses`, its public addresses, an array of strings;
- `connect`, the peers it keeps connected, an array of strings;
- `peers`, `static` or `dynamic`.

A field the module does not expose MUST NOT appear in a reply, and MUST be
refused by name when passed in a write.

#### Scenario: Every exposed field is present in a reply

- **WHEN** `getNodeConfig` is called against a home holding a node
  configuration
- **THEN** the reply MUST contain all five of `alias`, `listen`,
  `externalAddresses`, `connect` and `peers`
- **AND** it MUST contain `inboundReachable` and `restartRequired`
- **AND** it MUST contain no key other than those seven

#### Scenario: A derived member cannot be written

- **WHEN** `setNodeConfig` is called with the field `restartRequired`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST contain the text `restartRequired`

#### Scenario: A successful write returns the whole configuration object

- **WHEN** `setNodeConfig` accepts a call naming only `alias`
- **THEN** the reply MUST be the same shape `getNodeConfig` returns, containing
  all five configuration fields and both derived members
- **AND** the reply MUST carry no `error` member
- **AND** the four fields the call did not name MUST hold the values they held
  before it

#### Scenario: A field the module does not expose is refused by name

- **WHEN** `setNodeConfig` is called with the field `workers`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST contain the text `workers`
- **AND** a subsequent `getNodeConfig` reply MUST contain no key `workers`

#### Scenario: A peer discipline the module does not know is refused

- **WHEN** `setNodeConfig` sets `peers` to `manual`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST contain `static` and `dynamic`

#### Scenario: Static with nobody to connect to is refused

- **WHEN** `setNodeConfig` sets `peers` to `static` against a configuration
  whose `connect` is empty, without also setting `connect`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** a subsequent `getNodeConfig` reply's `peers` MUST be unchanged
- **AND WHEN** `setNodeConfig` sets `peers` to `static` and `connect` to a
  non-empty list of addresses in one call
- **THEN** the reply MUST NOT be an error

#### Scenario: peers carries a discipline rather than addresses

- **WHEN** `setNodeConfig` is called with `peers` set to an array of addresses
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** a subsequent `getNodeConfig` reply's `peers` MUST be one of the
  strings `static` or `dynamic`

### Requirement: Addresses are the two spellings the crate distinguishes

`connect` entries MUST each carry a node id, as `<node id>@<host>:<port>`.
`externalAddresses` and `listen` entries MUST each be a bare `<host>:<port>`
with no node id.

A value in the wrong spelling MUST be refused on write rather than stored, and
the refusal MUST name the value that was rejected and state which spelling was
expected.

#### Scenario: A connect address without a node id is refused

- **WHEN** `setNodeConfig` sets `connect` to `["seed.example.test:8776"]`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST contain `seed.example.test:8776`
- **AND** a subsequent `getNodeConfig` reply's `connect` MUST be unchanged

#### Scenario: An external address carrying a node id is refused

- **WHEN** `setNodeConfig` sets `externalAddresses` to
  `["z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7@seed.example.test:8776"]`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** a subsequent `getNodeConfig` reply's `externalAddresses` MUST be
  unchanged

#### Scenario: Each spelling is accepted in its own field

- **WHEN** `setNodeConfig` sets `connect` to
  `["z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7@seed.example.test:8776"]` and `externalAddresses` to
  `["node.example.test:8776"]`
- **THEN** the reply MUST NOT be an error
- **AND** a subsequent `getNodeConfig` reply MUST report both values as
  submitted

### Requirement: Inbound is off by default and its cost is stated

A configuration that has never set `listen` MUST report it as an empty array,
and the node MUST bind no port. `getNodeConfig` MUST report `inboundReachable`,
false exactly when `listen` is empty.

`setNodeConfig` MUST NOT add a listen address on its own — not when
`externalAddresses` or `connect` is set, and not when the node is started. A
view offering the choice MUST state that with inbound off, peers cannot fetch
from this node.

#### Scenario: A configuration that never set listen is outbound-only

- **WHEN** `getNodeConfig` is called against a home whose configuration has
  never set `listen`
- **THEN** `listen` MUST be an empty array
- **AND** `inboundReachable` MUST be false

#### Scenario: Setting an external address does not turn inbound on

- **WHEN** `setNodeConfig` sets `externalAddresses` to
  `["node.example.test:8776"]` against a configuration whose `listen` is empty
- **THEN** the reply MUST NOT be an error
- **AND** a subsequent `getNodeConfig` reply's `listen` MUST still be an empty
  array
- **AND** `inboundReachable` MUST still be false

#### Scenario: Setting a listen address turns inbound on

- **WHEN** `setNodeConfig` sets `listen` to `["0.0.0.0:8776"]`
- **THEN** a subsequent `getNodeConfig` reply's `listen` MUST be
  `["0.0.0.0:8776"]`
- **AND** `inboundReachable` MUST be true

#### Scenario: The panel states what inbound off means

- **WHEN** a view offers the inbound choice
- **THEN** it MUST display text saying that with inbound off, peers cannot
  fetch from this node

### Requirement: A write preserves the fields it does not set

`setNodeConfig` MUST change only the fields the call names, and MUST leave every
other field of `config.json` as it found it — including fields this module does
not expose, and keys no field of this build's configuration type corresponds to.

A write MUST leave the file parseable as a node configuration, and a write that
cannot be completed MUST leave the previous file intact rather than truncated.

#### Scenario: An unrelated known field survives a write

- **WHEN** a configuration holds a non-default value for a field this module
  does not expose
- **AND** `setNodeConfig` changes `alias`
- **THEN** that field MUST still hold its previous value afterwards

#### Scenario: A key this build does not know survives a write

- **WHEN** `config.json` holds a key no field of this build's configuration type
  corresponds to
- **AND** `setNodeConfig` changes `alias`
- **THEN** that key MUST still be present afterwards with its previous value
- **AND** the changed `alias` MUST be the value written

#### Scenario: A failed write leaves the previous configuration

- **WHEN** `setNodeConfig` is refused because a value did not validate
- **THEN** `config.json` MUST be byte-for-byte what it was before the call

### Requirement: The configuration read is of the mode's own home

`getNodeConfig` and `setNodeConfig` MUST act on the home the mode in force
resolved, and MUST NOT take a home as a parameter.

In `explore`, both MUST return `{"error":"..."}` naming the mode, and
`setNodeConfig` MUST write nothing anywhere. In `local`, `setNodeConfig` MUST
refuse, naming the mode, and `getNodeConfig` MUST succeed.

#### Scenario: Neither method takes a home

- **WHEN** a caller invokes either method across the module boundary
- **THEN** the method signature MUST expose no home argument, so no argument
  value can change which file is read or written

#### Scenario: Explore has no configuration to read

- **WHEN** the mode is `explore` and `getNodeConfig` is called
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST name the mode

#### Scenario: Local is readable but not writable

- **WHEN** the mode is `local` against a home whose `config.json` sets `alias`
  to `local-node-alias` and `connect` to
  `["z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7@seed.example.test:8776"]`
- **AND** the home `embedded` would resolve holds no `config.json`
- **THEN** `getNodeConfig` MUST NOT return an error
- **AND** the reply's `alias` MUST be `local-node-alias`
- **AND** the reply's `connect` MUST be
  `["z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7@seed.example.test:8776"]`
- **AND WHEN** `setNodeConfig` is called with `alias` set to `renamed`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST contain `local`
- **AND** that home's `config.json` MUST be byte-for-byte what it was before the
  call
- **AND** a subsequent `getNodeConfig` reply's `alias` MUST still be
  `local-node-alias`

#### Scenario: Embedded reads and writes its own home

- **WHEN** the mode is `embedded` with an identity created in its home, and
  `setNodeConfig` changes `alias`
- **THEN** the reply MUST NOT be an error
- **AND** the `config.json` under the embedded home MUST carry the new alias
- **AND** no file under the home `local` would have resolved MUST have changed

### Requirement: A home with no configuration is reported as such

`getNodeConfig` MUST return `{"error":"..."}` when the resolved home holds no
`config.json`, naming the home and stating that no identity has been created
there, and MUST NOT return a set of defaults for it.

`setNodeConfig` MUST refuse likewise, and MUST NOT bring a home or a
configuration into existence.

#### Scenario: An empty embedded home has no configuration

- **WHEN** the mode is `embedded`, no identity has been created, and
  `getNodeConfig` is called
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST name the resolved home

#### Scenario: A write does not create a configuration

- **WHEN** the mode is `embedded`, no identity has been created, and
  `setNodeConfig` is called
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** no `config.json` MUST exist under that home afterwards

## ADDED Requirements

### Requirement: A configuration write returns the whole configuration

A successful `setNodeConfig` MUST return the whole configuration object, in the
shape `getNodeConfig` returns, rather than only the fields that changed. It MUST
accept a subset of the fields, and MUST leave the fields it was not given
unchanged.

#### Scenario: Two fields set at once are both reported back

- **WHEN** `setNodeConfig` accepts a call naming `alias` and
  `externalAddresses`
- **THEN** the reply's `alias` and `externalAddresses` MUST be the values
  submitted
- **AND** its `listen`, `connect` and `peers` MUST hold the values they held
  before the call

### Requirement: The derived members are reported and never written

`getNodeConfig` MUST additionally report two derived members,
`inboundReachable` and `restartRequired`. They are not configuration fields,
and `setNodeConfig` MUST refuse either by name.

#### Scenario: inboundReachable cannot be written either

- **WHEN** `setNodeConfig` is called with the field `inboundReachable`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST contain the text `inboundReachable`

### Requirement: The peer discipline is a discipline, and is checked

`peers` MUST be `static` or `dynamic`, and a caller MUST NOT be able to submit
addresses through it: any other value MUST be refused with a message naming
both. `peers` set to `static` with an empty `connect` MUST be refused rather
than stored.

#### Scenario: A dynamic node with nobody to connect to is accepted

- **WHEN** `setNodeConfig` sets `peers` to `dynamic` against a configuration
  whose `connect` is empty
- **THEN** the reply MUST NOT be an error
- **AND** a subsequent `getNodeConfig` reply's `peers` MUST be `dynamic`
