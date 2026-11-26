# Security review — isolation-untouched-home

Dimension: **security** only. The diff is test code, docs and the change folder;
`radicle/rust-ffi/src/` and both `Cargo.toml`/`Cargo.lock` are unchanged
(`git diff origin/main --name-only` over them is empty), so there is no new
untrusted-input path, no new dependency and no change to `init_profile_inner`.

Clean areas, checked:

- **Never the real profile.** Every home comes from `fixture::scratch_dir`
  (`CARGO_MANIFEST_DIR/tmp/<name>`, gitignored by `radicle/rust-ffi/.gitignore`).
  `grep` finds no `RAD_HOME`/`HOME`/`set_var`/`env::var` in the test file apart
  from the header comment, and `profileinit.rs` reads no environment variable
  either, so a developer's exported `RAD_HOME` or `RAD_PASSPHRASE` cannot
  redirect or alter the new test. Every scratch name in the file is unique
  (the new `tree-record-*` and `profile-init-iso-*` included), so parallel tests
  cannot collide on a home.
- **The recorder cannot read outside its root.** `DirEntry::file_type` does not
  follow symlinks; `the_recorder_does_not_follow_a_symlink_out_of_its_root`
  pins that. `cleanup` and `scratch_dir` use `remove_dir_all`, which unlinks a
  symlink rather than following it, so the symlink fixture cannot delete
  `outside/`.
- **No key material in failure output.** Every assertion that prints a record
  prints paths only (`before.keys()`, `record_changes` lines). No `Entry` that
  came from a real home is `Debug`-printed or passed to `assert_eq!`; the
  `assert_eq!` on `Entry` values runs only against the hand-built fixture.
- **Unreadable entry panics rather than being skipped**, so the record cannot
  silently compare equal across a hole.
- **Unix-only code is gated** (`#[cfg(unix)]` on the symlink test).
- **Mutations I ran** (in my own tree, in `radicle/rust-ffi/src/profileinit.rs`):
  after a successful `init_profile`, scan sibling scratch homes and (1) `chmod`
  `keys/radicle` to 0644, which `two_homes_get_two_different_identities` passed
  under; then (2) as a control that the mutation was reachable, remove
  `keys/radicle`, which turned the test red with
  `["removed: keys/radicle"]`. Mutation (2) is the control for (1).

- [x] **`spec-writer`** — `radicle/rust-ffi/tests/profile_init.rs:60-100` (`Entry`), `design.md` Risks — "untouched" does not cover the mode of B's private key
      **Scenario:** `init_profile` for home A sets `~B/keys/radicle` to 0644
      (world-readable signing key) after B was created. `tree_record` stores
      names and bytes only, so `record_changes` is empty and
      `two_homes_get_two_different_identities` stays green.
      **Measured:** with that edit applied to `init_profile_inner`, the test
      passes (1 passed); the same edit with `remove_file` instead of `chmod`
      fails it, so the edit does run. This is the one way a creation can harm
      the other home's secret without changing a byte, and the requirement as
      written (entries and bytes) neither forbids nor tests it.
      `design.md` says the gap "is reported to the `spec-writer` as a finding
      about the spec", but no document in the change records that report, so
      this box is the record. Severity: low to medium (a permission regression
      on the user's real signing key; not reachable by the current backend).
      Needs a spec decision first: either add "and the mode of every entry" to
      the requirement (then `Entry` gains a `mode` on unix, and the recorder
      test table gains a chmod row), or state that modes are out of scope in the
      spec, not only in `design.md` Risks and `docs/rust-ffi.md`.
      **Outcome (`spec-writer`): accepted, and permissions are now in scope.**
      The requirement "Creating an identity leaves every other home untouched"
      now says the creation "MUST NOT change the permissions of any entry under
      it", and it states that timestamps and ownership are outside it. The first
      scenario now records each entry's permissions and asserts that every entry
      under home B still carries them. Why this way: "untouched" is what the
      requirement's title promises. A chmod is the one way to harm the other
      home's secret without changing a byte. And `std::fs::Permissions` can be
      compared on every platform, so the check needs no Unix-only gate.
      Timestamps stay out because nothing about a home's isolation depends on
      them. Ownership stays out because an unprivileged process cannot change
      it. The proposal's What Changes says the same.
      `openspec validate isolation-untouched-home --strict` passes. This now
      needs work from **`dev-writer`**: `Entry` records permissions, the
      recorder test table gains a permissions-only row, a chmod mutation is
      watched turning `two_homes_get_two_different_identities` red,
      `record_changes` names a permission change, and the text that rules modes
      out is reworded (`design.md` Non-Goals and Risks, `docs/rust-ffi.md`
      around "compares bytes and names, not file modes or timestamps").

- [ ] **`dev-writer`** — `radicle/rust-ffi/tests/profile_init.rs:25` — header says homes are "removed on drop"; nothing is dropped
      **Scenario:** `cleanup` is an explicit call at the end of each test, not a
      `Drop` guard. The new isolation test has several early `assert!` exits
      between creating B (an unencrypted key pair, passphrase `""`) and
      `cleanup`, and failing is what it is for, so on a red run both homes and
      their private keys stay in `radicle/rust-ffi/tmp/` until the next run
      clears them in `scratch_dir`. The keys are throwaway and the directory is
      gitignored, so the harm is a stray secret on a developer's disk, not a
      leak; the defect is that the comment claims a guarantee the code does not
      give. Style/accuracy, severity low: reword to "removed at the end of a
      passing test", or give `cleanup` a guard type. Pre-existing text, but this
      change adds the panics that make the claim false more often.
