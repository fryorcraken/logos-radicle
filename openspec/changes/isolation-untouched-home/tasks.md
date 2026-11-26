## Stages

- [x] spec — `spec-writer`
- [x] design + code — `dev-writer`
- [x] tests — `tester`
- [ ] review: correctness — `code-reviewer`
- [ ] review: security — `code-reviewer`
- [ ] review: readability — `code-reviewer`
- [ ] review: architecture — `code-reviewer`
- [ ] review: spec-test — `spec-test-reviewer`
- [ ] review: design — `design-reviewer`
- [ ] findings all ticked, `findings/` deleted — `closer`
- [ ] `openspec validate --strict`, then `archive` — `closer`
- [ ] CI green, title/body checked, PR merged — `closer`

# Tasks

## 1. One recorder for "nothing under here changed"

- [x] 1.1 Replace `keys_snapshot` in `radicle/rust-ffi/tests/profile_init.rs`
  with `tree_record` (recursive, directories as entries, file bytes, symlinks
  by target), `record_changes` and `holds_file`, in a commit that changes no
  behaviour; verify `reporting_encryption_leaves_the_key_untouched` stays green,
  and goes red when the encryption probe is mutated to rewrite `radicle.pub` or
  to add an entry under `keys/` (design Decision 3).

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
  why B must exist before it is recorded. Verify by reading it against the
  test.

## Workflow follow-up

- Archive after review, per `docs/OPENSPEC-ARCHIVE.md` (the `closer`'s).
