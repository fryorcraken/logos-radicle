## Stages

- [x] spec — `spec-writer`
- [x] design + code — `dev-writer`
- [x] tests — `tester`
- [x] review: correctness — `code-reviewer`
- [x] review: security — `code-reviewer`
- [x] review: readability — `code-reviewer`
- [x] review: architecture — `code-reviewer`
- [x] review: spec-test — `spec-test-reviewer`
- [x] review: design — `design-reviewer`
- [x] findings all ticked, `findings/` deleted — `closer`
- [x] `openspec validate --strict`, then `archive` — `closer`
- [x] CI green, title/body checked, PR merged — `closer`

# Tasks

## 1. One recorder for "nothing under here changed"

- [x] 1.1 Replace `keys_snapshot` in `radicle/rust-ffi/tests/profile_init.rs`
  with `tree_record` (recursive, directories as entries, file bytes, symlinks
  by target), `record_changes` and `holds_file`, in a commit that changes no
  behaviour; verify `reporting_encryption_leaves_the_key_untouched` stays green,
  and goes red when the encryption probe is mutated to rewrite `radicle.pub` or
  to add an entry under `keys/` (design Decision 3).
- [x] 1.2 Pin the recorder itself against a hand-built tree whose expected
  record and changes are written out: every depth with directories as
  entries, one table row per kind of change, and a symlink out of the root
  that is not followed. Verify that dropping the `changed:`, `removed:` or
  `added:` arm of `record_changes` each turns a recorder test red (design
  Decision 6, "The recorder is pinned by tests of its own").
- [x] 1.3 Record each entry's permissions (`Entry` = `Kind` +
  `std::fs::Permissions`, read through `DirEntry::metadata`) and report a
  permissions-only change as `permissions changed:` (design Decision 9).
  The recorder's table has a file row, a directory row and a bytes-and-mode
  row. Verified: dropping the `permissions changed:` arm turns the table test
  red at case 1 (`left: []`, `right: ["permissions changed: top.bin"]`), and
  a temporary chmod of every sibling home's `keys/radicle` to 0644 in
  `init_profile_inner` turns `two_homes_get_two_different_identities` red with
  `["permissions changed: keys/radicle"]`.

## 2. The isolation test checks the other home

- [x] 2.1 Before the test changes, verify that the old
  `two_homes_get_two_different_identities` stays green when `init_profile`
  copies its key pair over the previous home's. This shows what the test
  could not see.
- [x] 2.2 Create home B first. Assert that neither home path is a prefix of
  the other, and that home A does not exist before B is created and still
  does not exist after. Verify the test goes red when `init_profile` creates
  `home` in every sibling scratch directory (scenario "Creating into home B
  does not bring home A into being").
- [x] 2.3 Record B's whole home. Assert the record holds `keys/radicle`,
  `keys/radicle.pub` and `config.json`. Create A, assert its `nodeId` is
  non-empty and differs from B's, and assert that `record_changes` against
  B's home is empty. Verify the test goes red under each of these mutations:
  key pair overwritten, `config.json` overwritten, one entry added and one
  removed, and the null implementation (scenario "Creating into home A leaves
  home B's profile as it was"; design Decision 6).
- [x] 2.4 Run the full Rust layer from this tree. Verify that
  `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and
  `cargo test --manifest-path radicle/rust-ffi/Cargo.toml` are all green.

## 3. Documentation

- [x] 3.1 Update `docs/rust-ffi.md`'s "An isolation test needs two homes"
  paragraph. It no longer defers the second half to the issue, and it says
  why B must exist before it is recorded and why permissions are compared.
  Verify by reading it against the test.

## Workflow follow-up

- Archive after review, per `docs/OPENSPEC-ARCHIVE.md` (the `closer`'s).
