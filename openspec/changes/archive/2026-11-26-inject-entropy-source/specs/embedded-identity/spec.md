## MODIFIED Requirements

### Requirement: Creation yields a distinct identity per home

Each identity created through `createEmbeddedIdentity` or across the FFI
boundary MUST have a signing key derived from entropy read from the operating
system's random source, never from a fixed or pseudo-random seed. Both MUST
NOT accept an entropy source as an argument.

Two homes MUST therefore hold two different identities. The observable
statement is that the node ids differ: directory-shaped assertions cannot
distinguish this from a shared fixed seed.

#### Scenario: Two homes yield two different node ids

- **WHEN** an identity is created into home A and another into home B, where
  neither path is a prefix of the other
- **THEN** both replies MUST report non-empty `nodeId` values
- **AND** the two `nodeId` values MUST differ from each other
- **AND** each home MUST hold its own complete profile

#### Scenario: No caller can choose the entropy source

- **WHEN** a caller invokes `createEmbeddedIdentity` across the module
  boundary, or identity creation across the FFI boundary
- **THEN** both signatures MUST expose no argument naming or supplying an
  entropy source, so no argument value can change where the seed is read from

## ADDED Requirements

### Requirement: A failed entropy draw creates no key

The identity-creation backend MUST fail with `{"error":"..."}` when its entropy
source cannot be opened, reports an error other than an interruption, or ends
before supplying a seed's 32 bytes. A short seed MUST NOT be padded, truncated
or completed from elsewhere. The error MUST state that the entropy for a new
key could not be read and MUST include the text of any error that opening or
reading the source reported.

A failed draw MUST write no key material and MUST leave the home Empty.

#### Scenario: A source that cannot be opened creates no key

- **WHEN** the identity-creation backend creates an identity into an empty
  home, drawing from a source whose opening reports an error
- **THEN** the reply MUST contain an `error`
- **AND** the error message MUST state that the entropy for a new key could not
  be read
- **AND** the error message MUST include the text of the opening error
- **AND** the reply MUST NOT contain `created` or `nodeId`
- **AND** no file MUST exist under that home's `keys` directory

#### Scenario: A source that ends after 16 bytes creates no key

- **WHEN** the identity-creation backend creates an identity into an empty
  home, drawing from a source that supplies 16 bytes and then reports end of
  input
- **THEN** the reply MUST contain an `error`
- **AND** the error message MUST state that the entropy for a new key could not
  be read
- **AND** the reply MUST NOT contain `created` or `nodeId`
- **AND** no file MUST exist under that home's `keys` directory

#### Scenario: A source that fails mid-read creates no key

- **WHEN** the identity-creation backend creates an identity into an empty
  home, drawing from a source that supplies 16 bytes and then reports a read
  error
- **THEN** the reply MUST contain an `error`
- **AND** the error message MUST state that the entropy for a new key could not
  be read
- **AND** the error message MUST include the text of the read error
- **AND** no file MUST exist under that home's `keys` directory

#### Scenario: A failed draw leaves the home completable

- **WHEN** a creation into an empty home has failed because its source ended
  after 16 bytes
- **THEN** that home MUST classify as Empty
- **AND WHEN** an identity is then created into the same home from a source
  that supplies 32 bytes
- **THEN** the reply MUST report `created:true` with a non-empty `nodeId`

### Requirement: A seed supplied in pieces is a full read

The identity-creation backend MUST accept a seed whose 32 bytes arrive across
several reads, and MUST retry a read the source reports as interrupted rather
than fail it. The seed MUST be the first 32 bytes the source supplies, in the
order supplied, and once the source has supplied them the backend MUST NOT
read from it again, whether or not it would supply more, end or fail.

#### Scenario: A seed supplied in pieces is used whole

- **WHEN** identities are created into two empty homes from the same 32 bytes,
  one source supplying them in a single read and the other one byte per read
- **THEN** both replies MUST report `created:true`
- **AND** the two `nodeId` values MUST be equal

#### Scenario: An interrupted read is retried, not failed

- **WHEN** an identity is created into an empty home from a source that
  supplies 16 bytes, then reports one interrupted read, then supplies the
  remaining 16
- **THEN** the reply MUST report `created:true`
- **AND** its `nodeId` MUST equal the `nodeId` of an identity created into
  another empty home from the same 32 bytes supplied in a single read

#### Scenario: A source longer than a seed uses its first 32 bytes

- **WHEN** an identity is created into an empty home from a source that
  supplies 48 bytes and then reports end of input
- **THEN** the reply MUST report `created:true`
- **AND** its `nodeId` MUST equal the `nodeId` of an identity created into
  another empty home from the first 32 of those bytes supplied in a single read

#### Scenario: Nothing past the 32nd byte is read

- **WHEN** an identity is created into an empty home from a source that
  supplies 32 bytes and then reports a read error on every further read
- **THEN** the reply MUST report `created:true`
- **AND** its `nodeId` MUST equal the `nodeId` of an identity created into
  another empty home from the same 32 bytes supplied in a single read
- **AND** the source MUST have received no read after supplying its 32nd byte
