# Proposal

## Why

The `embedded-identity` spec already requires a short read of the new key's
entropy to fail rather than be padded, because a partly random seed becomes a
permanent signing key. Nothing can show that it does: the operating system's
random source cannot be made to short-read portably, and a mutation that
accepts a short read leaves every identity-creation test passing while the
module reports success with a well-formed DID.

## What Changes

- The rule that a short read of entropy fails moves out of "Creation yields a
  distinct identity per home" into a requirement of its own, which also covers
  a source that reports an error, and states what a failed draw leaves behind:
  an error reply, no key material, and a home a later creation can complete.
- A second new requirement says what "short" does not mean: a seed whose 32
  bytes arrive across several reads, or after an interrupted read, is a full
  read, and the key is derived from exactly those bytes.
- "Creation yields a distinct identity per home" is scoped to creation through
  `createEmbeddedIdentity` and the FFI boundary, and gains the statement that
  neither accepts an entropy source as an argument, so the operating system's
  random source stays the only one a caller can reach.
- The identity-creation backend gains a seam through which a test supplies its
  own entropy source. No observable behaviour of the production path changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `embedded-identity`: "Creation yields a distinct identity per home" is
  modified (short-read paragraph out, boundary statement and scenario in); "A
  failed entropy draw creates no key" and "A seed supplied in pieces is a full
  read" are added.

## Impact

- `radicle/rust-ffi/src/profileinit.rs`: how the seed is drawn, so a test can
  substitute the source; the production caller keeps reading the operating
  system's random source.
- `radicle/rust-ffi/tests/profile_init.rs`: tests for the short, failing and
  chunked sources.
- No change to `radicle_local_init_profile`'s signature, `radicle_ffi.h`,
  `radicle_impl.h`, the C++ core, the QML, or any dependency.
