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

- Comparing timestamps or ownership. The requirement places both outside it:
  nothing about a home's isolation depends on a timestamp, and an
  unprivileged process cannot change ownership. Permissions are compared; see
  Decision 9.
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
`node/notifications.db`, `node/policies.db`). A keys-only record would miss
writes to five of those seven files.

That is the shape of a home made by radicle 0.25.1 (the version
`radicle/rust-ffi/Cargo.lock` names), so it goes stale when the crate moves. To
re-measure, add a temporary `eprintln!` of each path in `before`, its file
length and its permissions to `two_homes_get_two_different_identities`, run
`cargo test --manifest-path radicle/rust-ffi/Cargo.toml --test profile_init
two_homes -- --nocapture`, and remove the line. On that run the private key
`keys/radicle` was mode 0600, every other file 0644 and every directory 0755;
the last two follow the process's umask, which was 022.

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

Because the keys test uses the same recorder, it now compares permissions under
`keys/` too (Decision 9). That is inside its requirement, which says reporting
"MUST NOT unseal, rewrite or otherwise alter the key", even though the
scenario under it names bytes only.

The recorder stays in `tests/profile_init.rs` rather than moving to
`tests/fixture/mod.rs` with the other shared helpers. `fixture` is compiled
into every integration-test binary, so the recorder's own tests would run once
per binary, and this file is its only consumer.

### 4. Report differences as a list of paths, not with `assert_eq!` on the maps

A home's record holds 139,264 bytes of SQLite files (28,672 + 69,632 + 20,480
+ 20,480, from the measurement above). `assert_eq!` would print
all of it and bury the one path that moved. `record_changes` returns one
`added:` / `removed:` / `changed:` line per path, and the assertion message is
that list.

### 5. Do not follow symlinks, and panic on an unreadable entry

`DirEntry::metadata`, unlike `fs::metadata`, does not follow symlinks, so the
type and permissions recorded for a link are the link's own. A symlink is
recorded by its target, so the record never reads outside the root it was
asked about. Profile creation makes no symlinks today, so neither isolation
test meets one. The rule is exercised by
`the_recorder_does_not_follow_a_symlink_out_of_its_root`, against a
hand-built link out of the root: it is the guard that the record of one root
cannot change when a tree outside it does, and that retargeting a link shows
as `changed:`.

The alternative considered was to panic on any symlink, which would be one
line in place of the `Link` kind and its arm. It was ruled out because it ties
both tests to the crate's home layout: a radicle version that put a symlink in
a fresh home would turn the isolation test red at the record, for a reason
that has nothing to do with isolation, and the recorder would have to be
rewritten before the test measured anything again. A panic arm would also need
a test of its own to be pinned, so the code it saves is the `Link` kind and
three lines, not the test.

An entry that cannot be read panics instead of being skipped. A record with a
gap in it would compare equal across exactly the change it was meant to catch.
Skipping and continuing would cost exactly that: a directory that could not be
listed when both records were taken would leave its contents out of both, and every write
beneath it would go unseen. A file that became unreadable only between the two
records would show as `removed:` while still being there, which is red but
names the wrong change.

That panic has no test of its own, and that is a concession. Making an entry
unreadable needs a `chmod 000` that a process running as root reads through
anyway, so the test would pass or fail on who runs it.

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

**The recorder is pinned by tests of its own**, because both tests that use it
are only as sharp as it is, and neither can tell a blind recorder from a clean
home: a recorder that missed a kind of change reports nothing, which is
exactly what both tests expect. So it is checked against a hand-built tree
whose expected record and expected changes are written out in the tests, not
derived from the recorder:

- `the_recorder_lists_every_entry_at_every_depth_with_directories` pins depth,
  directories as entries, and file bytes.
- `the_recorder_names_each_kind_of_change_by_path` pins each line
  `record_changes` produces, one table row per kind of change.
- `the_recorder_does_not_follow_a_symlink_out_of_its_root` pins Decision 5.

Each arm of `record_changes` was dropped in turn, by a temporary edit, and
`cargo test --manifest-path radicle/rust-ffi/Cargo.toml --test profile_init
the_recorder` run. The table test stops at its first failing row, so the row
named is the first that went red, not the only one that would:

| Arm dropped from `record_changes` | Result |
|---|---|
| `changed:` | **red**: table row 1, "a deep file's bytes change"; and the symlink test, "retargeting a link must show as a change" |
| `removed:` | **red**: table row 2, "a file is removed" |
| `added:` | **red**: table row 4, "an empty directory is added" |
| `permissions changed:` | **green**, all 22 tests in the file, before the recorder's table had a permissions row. That row is the guard for this arm (Decision 9) |

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

### 9. Permissions are part of each entry, as `std::fs::Permissions`

A `chmod` is the one way a creation can harm the other home's secret without
changing a byte of it: making `keys/radicle`, mode 0600 in a fresh home,
world-readable. Before permissions were recorded, the security review made
`init_profile_inner` set the sibling's `keys/radicle` to 0644 after creating a
home, and `two_homes_get_two_different_identities` stayed green under it. The
requirement now forbids a permissions change, so the record carries one.

`Entry` is a struct of the entry's `Kind` (directory, file bytes, or link
target) and its `std::fs::Permissions`, read from the same `DirEntry::metadata`
call that gives the type, so a link's own permissions are recorded and its
target's are not.

Alternatives considered:

- **`PermissionsExt::mode() & 0o7777`.** It gives exact Unix bits, but needs a
  `#[cfg(unix)]` gate on the recorder, and the requirement's check needs
  none. `Permissions` compares on every platform. On Unix its equality is over
  the full `st_mode`, type bits included, which changes nothing here because
  the type is compared through `Kind` first.
- **A permissions field on each `Kind` variant.** Three copies of one field,
  and every match on a kind would have to name it.
- **A second record holding permissions only.** Two maps to keep in step for
  one walk.

`record_changes` keeps one line per path. A path whose kind or bytes changed is
`changed:`; one whose contents are equal but whose permissions moved is
`permissions changed:`. A path where both moved is reported as `changed:`
only. It still turns the test red, and two lines for one path would make the
failure longer without naming anything new.

Without the `permissions changed:` arm the recorder reports a chmod as no
change at all. Its guard is the permissions-only row of
`the_recorder_names_each_kind_of_change_by_path`, together with a chmod
mutation of `init_profile_inner` that turns the isolation test red.

## Risks / Trade-offs

- [`std::fs::Permissions` outside Unix holds only the read-only flag] → On
  Windows a permission change other than read-only would compare equal. No
  test job runs on Windows, and it is not a built platform (see
  `docs/rust-ffi.md`, "Platform scope").
- [A future crate version leaves a WAL or SHM file beside a database] → B's
  record would gain entries only if something kept writing to B after its
  creation returned. Nothing does today. If it happens, the failure names the
  file.
- [Tests run in parallel under the same `rust-ffi/tmp/`] → The record covers
  B's home and nothing else. Other tests' scratch directories are siblings of
  `profile-init-iso-b`, not inside it, so they cannot change the record.
