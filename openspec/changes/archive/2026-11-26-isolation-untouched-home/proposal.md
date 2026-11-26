# Proposal

## Why

Embedded identities are isolated in two senses: two homes hold two different
identities, and creating one leaves the other alone. The `embedded-identity`
contract states only the first, and the isolation test checks the second only
by existence and non-nesting, which a creation that wrote into a sibling home
would pass. Both the contract and its test have to state the second half before
a regression there can turn anything red.

## What Changes

- Add a requirement to `embedded-identity` that creating an identity into one
  home adds, removes and rewrites nothing under any other home, changes the
  permissions of nothing there, and does not bring a home path that did not
  exist into being. A permission change matters on its own: making another
  home's private key world-readable changes no byte of it.
- Strengthen the isolation test (`two_homes_get_two_different_identities`) to
  record one home's entries, their permissions and the files' bytes before the
  other is created and compare them afterwards, and to check that the second
  home's path is absent until its own creation.
- No product behaviour changes: the requirement states what the backend is
  expected to do already. If the strengthened test goes red, that is a defect
  the old test hid, and it is fixed in this change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `embedded-identity`: adds the requirement "Creating an identity leaves every
  other home untouched", alongside the existing "Creation yields a distinct
  identity per home".

## Impact

- `radicle/rust-ffi/tests/profile_init.rs`, the Rust FFI test layer. No API,
  JSON shape, dependency or QML change.
