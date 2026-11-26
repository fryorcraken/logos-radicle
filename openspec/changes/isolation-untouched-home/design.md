# Design

## Context

See `proposal.md` for why. The backend already behaves as the new requirement
says; what changes is the test that pins it,
`two_homes_get_two_different_identities` in
`radicle/rust-ffi/tests/profile_init.rs`. That test used to check the second
home only with `profile_exists` and by making sure the two path strings do not
nest. Neither check can see a creation that writes into, or deletes from, a
sibling home.

The same file already held a byte-for-byte record, `keys_snapshot`, written
for "Reporting encryption leaves the key untouched". It read only the top level
of `<home>/keys`.

`profileinit.rs` is not changed. The strengthened test passes against the
backend as it stands, so the old test was not hiding a defect.

## Goals / Non-Goals

**Goals:**

- Use one recorder of a directory tree in this file for both tests that need
  "nothing under here changed".
- Make a failure name the paths that were added, removed or changed.

**Non-Goals:**

- Comparing file modes, owners or timestamps. The requirement speaks of
  entries and bytes. See Risks.
- Detecting writes to a third location that is neither home. The second
  scenario is about home A's path only.

## Decisions

### 1. Create B first, then A. Do not snapshot a B that does not exist yet

The issue allowed either approach: "snapshots home B before home A is
initialised (or initialises B first, then A)". They are not equivalent. A
snapshot of a home that does not exist yet is empty. An empty record can see
entries appear, but it can never see a rewrite. The failure that matters most
is a creation that overwrites a sibling's key pair, and a rewrite is exactly
what an empty record misses. So B is created and populated first, its record is
taken, and only then is A created.

The other direction comes almost free with this order. A's path does not exist
while B is being created, so the test checks that it still does not exist
afterwards. That is the second scenario, and it catches a creation that brings
a sibling home into being.

The issue's own framing explains why two homes are needed at all: "A fixture
with one home cannot tell isolation from its absence."

### 2. Record the whole home recursively, not just `keys/`, and not just keys plus `config.json`

The issue's minimum was "its key files and config are byte-identical, and no
new entries appeared". The requirement says every entry under the other home,
and those differ in practice. A real profile's home holds 11 entries: 4
directories (`cobs`, `keys`, `node`, `storage`) and 7 files. Three of the files
are `keys/radicle`, `keys/radicle.pub` and `config.json`, and the other four
are SQLite databases (`cobs/cache.db`, `node/node.db`,
`node/notifications.db`, `node/policies.db`). This was measured by printing B's
record once during development. A keys-only record would miss writes to five
of those seven files.

Directories are entries in their own right. Without that, an empty directory
added or removed under B would go unseen, and `storage/` is empty in a fresh
home.

### 3. Generalise `keys_snapshot` instead of adding a second recorder

`keys_snapshot` was a one-level version of the same idea. Keeping it beside a
recursive recorder would mean two definitions of "what a record is" in one
file. They would differ in depth, in key type (`String` against a relative
`PathBuf`) and in how they treat a subdirectory. The old one panicked when it
read a directory as a file. So `tree_record` replaces it.
`reporting_encryption_leaves_the_key_untouched` now runs it on `<home>/keys`.

This landed as its own commit, with no behaviour change and every gate green.
In that commit the keys test still went red when the encryption probe was
mutated to rewrite `radicle.pub` (`changed: radicle.pub`), and again when it
was mutated to add an entry (`added: stray`).

### 4. Report differences as a list of paths, not with `assert_eq!` on the maps

A home's record holds 139,264 bytes of SQLite files (28,672 + 69,632 + 20,480
+ 20,480, from the measurement above). `assert_eq!` would print
all of it and bury the one path that moved. `record_changes` returns one
`added:` / `removed:` / `changed:` line per path, and the assertion message is
that list.

### 5. Do not follow symlinks, and panic on an unreadable entry

`DirEntry::file_type` does not follow symlinks. A symlink is recorded by its
target, so the record never reads outside the root it was asked about. Profile
creation makes no symlinks today, so this case is never exercised; it is there
so that the record has a defined answer for every entry.

An entry that cannot be read panics instead of being skipped. A record with a
gap in it would compare equal across exactly the change it was meant to catch.

### 6. Guards, and what each one turns red

The issue's second condition was: "The assertion is input-dependent. A
mutation that makes init write into a sibling home turns it red." Each guard
below was checked by a temporary edit to `init_profile_inner`, run against
this test alone (`cargo test … --test profile_init two_homes`), with the file
restored afterwards:

| Mutation to `init_profile` | Result |
|---|---|
| copy the new key pair over the previous home's | **red**: `changed: keys/radicle`, `changed: keys/radicle.pub`. The *old* test was **green** under this mutation, which was run before the test changed |
| copy the new `config.json` over the previous home's | **red**: `changed: config.json` |
| add `stray/` to the previous home and remove its `storage/` | **red**: `removed: storage`, `added: stray` |
| create `home` in every sibling scratch dir | **red**: "creating an identity into home B brought home A into being" |
| null implementation: report success with `nodeId = home`, write nothing | **red**: the record cannot list B's home |

The preconditions are the spec's GIVEN clauses:

- The record of B holds `keys/radicle`, `keys/radicle.pub` and `config.json`
  as files. This stops two near-empty records from comparing equal.
- A does not exist before anything is created.
- Neither path is a prefix of the other.

None of them has been shown to be the *only* thing that turns a particular
mutation red. A creation that made only the directories would also fail the
closing `profile_exists` assertion. The preconditions make the test's coverage
explicit. Nothing measured shows they carry it on their own.

### 7. One test, not one per scenario

Both scenarios use the same two homes. The second scenario's check sits
between B's creation and B's record, so splitting it out would mean creating B
twice. The test keeps its name, because `docs/rust-ffi.md` and the header of
`tests/profile_init.rs` refer to it by that name.

### 8. Manual mutation rather than `cargo mutants`

The change is test code. The property is "creation does not write to a
sibling", and `cargo mutants` does not generate the mutants that would break
it: it replaces function bodies and operators, and has no way to add a write
to a different path. The mutations in Decision 6 are the ones that test this
property.

## Risks / Trade-offs

- [The record compares names and bytes, not modes] → A creation that ran
  `chmod` on B's private key, say to 0644, would pass. The requirement as
  written does not cover this. It is reported to the `spec-writer` as a
  finding about the spec, not built here.
- [A future crate version leaves a WAL or SHM file beside a database] → B's
  record would gain entries only if something kept writing to B after its
  creation returned. Nothing does today. If it happens, the failure names the
  file.
- [Tests run in parallel under the same `rust-ffi/tmp/`] → The record covers
  B's home and nothing else. Other tests' scratch directories are siblings of
  `profile-init-iso-b`, not inside it, so they cannot change the record.
