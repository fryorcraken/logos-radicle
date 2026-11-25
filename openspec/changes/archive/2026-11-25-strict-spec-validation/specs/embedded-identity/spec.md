## MODIFIED Requirements

### Requirement: Neither identity method takes a home

`getEmbeddedIdentity()` and `createEmbeddedIdentity(alias, passphrase)` MUST
NOT accept a Radicle home as a parameter. Both MUST resolve the embedded home
themselves, from the module's data directory alone.

The embedded home MUST be derived from `XDG_DATA_HOME`, falling back to
`$HOME/.local/share` when `XDG_DATA_HOME` is unset. The derivation MUST NOT
consult `RAD_HOME`, and MUST NOT consult `$HOME/.radicle`.

#### Scenario: The reported home is the module's, not the environment's node

- **WHEN** `RAD_HOME` names a readable Radicle profile at one path and
  `XDG_DATA_HOME` names a different directory
- **THEN** `getEmbeddedIdentity()` MUST report `home` derived from the
  `XDG_DATA_HOME` directory
- **AND** the reported `home` MUST NOT equal the path `RAD_HOME` names

#### Scenario: Creation writes only into the embedded home

- **WHEN** `RAD_HOME` names a scratch directory holding no profile,
  `XDG_DATA_HOME` names a different directory, and
  `createEmbeddedIdentity("tester", "")` succeeds
- **THEN** `keys/radicle.pub` MUST exist under the embedded home derived from
  `XDG_DATA_HOME`
- **AND** neither `keys` nor `config.json` MUST exist under the path
  `RAD_HOME` names

#### Scenario: A view cannot name the home

- **WHEN** a caller invokes either identity method across the module boundary
- **THEN** the method signature MUST expose no home argument, so no argument
  value can change which directory is read or written

### Requirement: Reporting what the embedded home holds

`getEmbeddedIdentity()` MUST report the embedded home without creating,
modifying or deleting anything in it, as
`{"home":"<path>","exists":bool,"nodeId":"<did>","encrypted":bool,"problem":""}`.

`exists` MUST be true only when `keys/radicle.pub` and `config.json` are both
present. `nodeId` MUST be the DID form, read from the public key, and `""`
when `exists` is false. `encrypted` MUST report whether the signing key on disk
is sealed with a passphrase, and MUST be false when `exists` is false.

#### Scenario: No identity yet is the ordinary state

- **WHEN** the embedded home holds no key material
- **THEN** `getEmbeddedIdentity()` MUST return `exists:false`
- **AND** `nodeId` MUST be `""`
- **AND** `encrypted` MUST be false
- **AND** `problem` MUST be `""`
- **AND** `home` MUST be the non-empty embedded home path

#### Scenario: A complete identity is reported with its DID

- **WHEN** `createEmbeddedIdentity("tester", "")` has succeeded and returned a
  non-empty `nodeId`
- **THEN** a subsequent `getEmbeddedIdentity()` MUST return `exists:true`
- **AND** its `nodeId` MUST equal the `nodeId` the creation reported

#### Scenario: The reported encryption follows the key that was written

- **WHEN** `createEmbeddedIdentity("tester", "")` has succeeded
- **THEN** a subsequent `getEmbeddedIdentity()` MUST return `encrypted:false`
- **AND WHEN** an identity is instead created in a different home with
  `createEmbeddedIdentity("tester", "correct horse battery")`
- **THEN** `getEmbeddedIdentity()` against that home MUST return
  `encrypted:true`

#### Scenario: Reporting encryption needs no passphrase and no agent

- **WHEN** the embedded home holds an identity created with a passphrase, with
  no passphrase supplied through the environment and no ssh-agent reachable
- **THEN** `getEmbeddedIdentity()` MUST return `exists:true` with
  `encrypted:true`
- **AND** `problem` MUST be `""`
- **AND** the reply MUST NOT contain an `error` key

#### Scenario: A half-written home is not reported as occupied

- **WHEN** the embedded home holds `keys/radicle.pub` but no `config.json`
- **THEN** `getEmbeddedIdentity()` MUST return `exists:false`
- **AND** `nodeId` MUST be `""`
- **AND** `encrypted` MUST be false

#### Scenario: No data directory at all

- **WHEN** neither `XDG_DATA_HOME` nor `HOME` is set
- **THEN** `getEmbeddedIdentity()` MUST return `home:""` and `exists:false`
- **AND** `encrypted` MUST be false
- **AND** `problem` MUST be non-empty
- **AND** the reply MUST NOT contain an `error` key

### Requirement: Three home states, not two

The classification of an embedded home MUST distinguish three states:

- **Empty** — no `keys/radicle.pub`, whether or not the directory exists.
- **Complete** — `keys/radicle.pub` and `config.json` both present.
- **Partial** — `keys/radicle.pub` present, `config.json` absent.

`config.json` MUST be the completeness marker, and `storage/` MUST NOT be: it
is created before any key is written. A Partial home MUST report
`exists:false`.

#### Scenario: An existing but empty directory is not a profile

- **WHEN** the embedded home directory exists and is empty
- **THEN** it MUST classify as Empty
- **AND** `createEmbeddedIdentity("tester", "")` into it MUST succeed with
  `created:true`

#### Scenario: `storage/` is present in the Partial state

- **WHEN** a complete profile has been created and its `config.json` is then
  removed
- **THEN** `storage/` MUST still exist under that home
- **AND** `config.json` MUST NOT exist
- **AND** the home MUST classify as Partial rather than Complete

### Requirement: An occupied home is refused and never overwritten

`createEmbeddedIdentity(alias, passphrase)` MUST refuse a Complete home, leave
its key material byte-for-byte unchanged, and offer no `force` parameter or
other way to overwrite an identity.

The refusal MUST be `{"error":"..."}` naming the home that was in the way and
stating that creating another would overwrite a signing key that cannot be
recovered. It MUST happen before the home's directory tree is created or
modified.

#### Scenario: A second creation is refused and the first identity survives

- **WHEN** `createEmbeddedIdentity("tester", "")` has succeeded with node id
  `N`
- **AND** `createEmbeddedIdentity("someone-else", "")` is then called
- **THEN** the second reply MUST contain an `error`
- **AND** the error message MUST name the embedded home
- **AND** the error message MUST state that creating another would overwrite
  its signing key
- **AND** a subsequent `getEmbeddedIdentity()` MUST still report `nodeId` equal
  to `N`

#### Scenario: No force escape hatch exists

- **WHEN** a caller wishes to replace an existing embedded identity
- **THEN** the method signature MUST accept only `alias` and `passphrase`, so
  no argument value can turn the refusal into an overwrite

### Requirement: A half-created home is reported as recoverable, not occupied

`createEmbeddedIdentity` MUST refuse a Partial home with a message distinct from
the occupied-home refusal. It MUST describe the home as half-created, MUST state
that nothing has ever signed with the key material present, MUST name the `keys`
directory as the path to remove, and MUST NOT claim a signing key is at stake.

The module MUST NOT delete key material automatically. Removing the named path
MUST make creation succeed.

#### Scenario: The half-created refusal differs from the occupied refusal

- **WHEN** the embedded home holds key material but no `config.json`
- **AND** `createEmbeddedIdentity("tester", "")` is called
- **THEN** the reply MUST contain an `error`
- **AND** the message MUST describe the home as half-created
- **AND** the message MUST name the `keys` path to remove
- **AND** the message MUST NOT say that a signing key would be overwritten

#### Scenario: The named recovery works

- **WHEN** the embedded home is in the Partial state and the `keys` directory
  it names is removed
- **AND** `createEmbeddedIdentity("tester", "")` is then called
- **THEN** the reply MUST report `created:true` with a non-empty `nodeId`

#### Scenario: Key material is never deleted by the module

- **WHEN** `createEmbeddedIdentity` refuses a home in the Partial state
- **THEN** `keys/radicle.pub` MUST still exist under that home afterwards

### Requirement: The created identity is real and readable

A successful `createEmbeddedIdentity` MUST return
`{"created":true,"nodeId":"did:key:z6Mk…","home":"<path>","alias":"<alias>","encrypted":bool}`.

`home` MUST be the embedded home the module derived, and `nodeId` MUST be
spelled as `getCapabilities().nodeId` spells it. An independent read of
`keys/radicle.pub` MUST yield the reported `nodeId`. The created profile MUST be
openable by the module's existing local read path.

#### Scenario: The reported identity is the one on disk

- **WHEN** `createEmbeddedIdentity("tester", "")` reports `nodeId` `N`
- **THEN** an independent read of that home's `keys/radicle.pub` MUST report
  `nodeId` `N`

#### Scenario: A fresh profile reads as an empty one, not as a failure

- **WHEN** an identity has just been created into an empty embedded home
- **AND** the local read path lists that home's repositories
- **THEN** the reply MUST NOT contain an `error`
- **AND** it MUST list zero repositories

### Requirement: The passphrase decides whether the profile can sign

`createEmbeddedIdentity` MUST treat an empty `passphrase` as no passphrase and
write the signing key unencrypted; a non-empty one MUST encrypt it. The reply's
`encrypted` MUST agree with the key on disk.

An unencrypted key MUST be signable at once, with no prompt. An encrypted key
MUST NOT be: with no passphrase in the environment and no agent holding it, a
write probe MUST report that it cannot write. Reads MUST NOT require the
passphrase: the identity MUST stay visible while the key is locked.

#### Scenario: An empty passphrase yields an immediately signable profile

- **WHEN** `createEmbeddedIdentity("tester", "")` is called with an empty
  passphrase
- **THEN** the reply MUST report `encrypted:false`
- **AND** a write probe against that home MUST report that a write could
  succeed

#### Scenario: A passphrase yields a profile that cannot write until unlocked

- **WHEN** `createEmbeddedIdentity("tester", "correct horse battery")` is
  called
- **AND** no passphrase is available through the environment or an agent
- **THEN** the reply MUST report `encrypted:true`
- **AND** a write probe against that home MUST report that a write could not
  succeed

#### Scenario: `canWriteLocal` follows the passphrase in Embedded mode

- **WHEN** the module is in `embedded` mode and the embedded identity was
  created with an empty passphrase
- **THEN** `getCapabilities().canWriteLocal` MUST be true
- **AND WHEN** the embedded identity was instead created with a non-empty
  passphrase and nothing can supply it
- **THEN** `getCapabilities().canWriteLocal` MUST be false with a non-empty
  `writeUnavailableReason`

#### Scenario: The identity stays visible while the key is locked

- **WHEN** the embedded identity was created with a non-empty passphrase and
  nothing can supply it
- **THEN** `getEmbeddedIdentity()` MUST still report `exists:true` with the
  created `nodeId`

### Requirement: Creation does not change the mode, and repoints only in Embedded

`createEmbeddedIdentity` MUST succeed whichever mode is in force, and MUST NOT
change the persisted mode.

When `embedded` is the mode in force, a successful creation MUST repoint this
instance's local store at the new identity without a restart, and MUST announce
the changed capabilities. When any other mode is in force it MUST NOT repoint
the local store: the module MUST go on reading the home that mode resolved.

#### Scenario: In Embedded, the new identity is readable at once

- **WHEN** the mode is `embedded`, `getCapabilities().localAvailable` is false,
  and `createEmbeddedIdentity("tester", "")` succeeds
- **THEN** `getCapabilities().localAvailable` MUST become true without a
  restart
- **AND** `getCapabilities().nodeId` MUST equal the created `nodeId`

#### Scenario: In Local, creating an embedded identity changes nothing on screen

- **WHEN** the mode is `local` with `radHome` resolved to the user's own
  profile, and `createEmbeddedIdentity("tester", "")` succeeds
- **THEN** `getCapabilities().mode` MUST still be `local`
- **AND** `getCapabilities().radHome` MUST still be the user's own profile
  path
- **AND** `getEmbeddedIdentity().exists` MUST be true, so the identity really
  was created elsewhere

### Requirement: The embedded identity is a new one and must be stated as such

The identity `createEmbeddedIdentity` creates MUST be a new DID, distinct from
any the user already holds, and the reply MUST carry its `nodeId`. A view MUST
state that it is a separate identity.

Where the module explains why writes are unavailable in Embedded before an
identity exists, the explanation MUST name the embedded home, MUST say it is a
separate identity from any Radicle node already running, and MUST NOT advise
running `rad auth`.

#### Scenario: The absent-identity explanation fits Embedded, not Local

- **WHEN** the mode is `embedded` and no embedded identity has been created
- **THEN** `getCapabilities().writeUnavailableReason` MUST be non-empty
- **AND** it MUST contain the resolved `radHome`
- **AND** it MUST state that this is a separate identity
- **AND** it MUST NOT mention `rad auth`

## ADDED Requirements

### Requirement: Reporting encryption leaves the key untouched

Determining `encrypted` MUST NOT require a passphrase, an ssh-agent, or any
attempt to load or use the private key. Reporting it MUST NOT unseal, rewrite
or otherwise alter the key.

#### Scenario: A report leaves the sealed key as it was

- **GIVEN** an embedded home holding an identity created with the passphrase
  `correct horse battery`
- **WHEN** `getEmbeddedIdentity()` is called
- **THEN** every file under that home's `keys` directory MUST be byte-for-byte
  what it was before the call
- **AND** the reply MUST report `encrypted:true`

### Requirement: An unreadable key is not reported as unencrypted

When the signing key cannot be read, `getEmbeddedIdentity()` MUST report
`encrypted:false` with a non-empty `problem`, so that an unreadable key and an
unencrypted one are never reported alike.

#### Scenario: An unreadable key carries a problem

- **GIVEN** an embedded home holding a complete identity whose private key file
  exists but cannot be read by the module
- **WHEN** `getEmbeddedIdentity()` is called
- **THEN** `encrypted` MUST be false
- **AND** `problem` MUST be non-empty
- **AND WHEN** the same home's key is readable and unencrypted
- **THEN** `encrypted` MUST be false
- **AND** `problem` MUST be `""`

### Requirement: No data directory is an answer, not an error

When neither `XDG_DATA_HOME` nor `HOME` is set, `getEmbeddedIdentity()` MUST
return `home:""`, `exists:false`, `nodeId:""`, `encrypted:false` and a non-empty
`problem`. It MUST NOT return an `{"error":"..."}` object.

#### Scenario: No data directory still reports every field

- **WHEN** neither `XDG_DATA_HOME` nor `HOME` is set
- **THEN** `getEmbeddedIdentity()` MUST return `nodeId` as `""`
- **AND** the reply MUST contain each of `home`, `exists`, `nodeId`,
  `encrypted` and `problem`

### Requirement: canWriteLocal is not a substitute for the encrypted field

`getCapabilities().canWriteLocal` MUST NOT be treated as a substitute for
`getEmbeddedIdentity().encrypted`.

#### Scenario: A key that cannot sign is not taken for an encrypted one

- **GIVEN** a repository list in `embedded` told `exists:true` with
  `encrypted:false`, a node reporting `running:false` and `serving:false`, and
  capabilities reporting `canWriteLocal:false`
- **THEN** a `startNode` call with an empty passphrase MUST have been issued
- **AND** no field for a passphrase MUST be present
