# Architecture review: isolation-untouched-home

Dimension reviewed: **architecture** only. Correctness, security and readability
are other instances'. The diff against `origin/main` is test code
(`radicle/rust-ffi/tests/profile_init.rs`), `docs/rust-ffi.md` and the change
folder; `profileinit.rs` is untouched, as the design says (confirmed by
`git diff origin/main --stat`).

Verified, not taken from the PR text: `cargo clippy --all-targets -- -D warnings`
is clean on this tree, and `cargo test --test profile_init` runs 22 tests, all
passing. No mutation was made; the change is test-only and design Decision 8's
argument against `cargo mutants` holds (it cannot generate "write into a sibling
home"). Nothing was left mutated in the tree.

## Clean areas

- **Make the change easy, then the easy change.** `keys_snapshot` was replaced by
  `tree_record` first, as its own no-behaviour commit, before the isolation test
  used it (`git log`: `55f1123` then `d9e721d`). This is the shape the
  specflow block asks for.
- **One recorder, not two.** No other test file in `radicle/rust-ffi/tests/`
  carries a tree comparison (`node_config.rs` compares single files with
  `fs::read`, which a tree recorder would not simplify), so there is no
  duplicate to fold in.
- **Placement in the file rather than `tests/fixture/mod.rs`** looks like the
  wrong home at first (shared helpers live there) but is right: `fixture` is
  compiled into every integration-test binary, so the recorder's three
  self-tests would run once per binary. Keeping them beside their only consumer
  is the sound reading. The design says "in this file" but does not give this
  reason; it is mentioned here so the closer does not read the placement as an
  oversight.
- **One function, one job.** `tree_record` / `record_under` / `record_changes` /
  `holds_file` each do one thing; the recorder is a plain data structure
  (`BTreeMap<PathBuf, Entry>`) and the comparison is a diff of two maps, with no
  per-call-site guard logic.
- **Fakes input-dependent.** The recorder is tested against a hand-written tree
  whose expected record is spelled out, not derived from the recorder.
- **Interface width.** Nothing crosses the FFI boundary; no new dependency
  (`Cargo.toml` and `Cargo.lock` unchanged).
- **CI gates.** The Rust job's floor is `-ge 28` passed, a floor not an exact
  count; this change adds three tests and removes none, so it cannot lower the
  count. No file moved or renamed, so no gate measures a stale path.
- **`two_homes_get_two_different_identities` carrying two jobs under one name**
  was considered. Decision 7 argues it (the second check sits between B's
  creation and B's record, so a split creates B twice) and the failure messages
  name the property that failed, so I raise no box.
- **`record_changes` returning `Vec<String>`** rather than an enum was
  considered and left alone: the strings exist to be read in a failure, and an
  enum plus `Display` would be more code for the same output.

## Findings

- [x] **`dev-writer`** — `radicle/rust-ffi/tests/profile_init.rs:631` (`Entry::Link`), `:661` (`record_under`'s symlink arm) and `:827` (`the_recorder_does_not_follow_a_symlink_out_of_its_root`) — stylistic, low: a recorder variant, a match arm and a ~35-line test exist for a case no input reaches
      **Scenario:** design Decision 5 states "Profile creation makes no symlinks today, so this case is never exercised; it is there so that the record has a defined answer for every entry." That is the speculative-generality shape the specflow block's "do not refactor speculatively" rules out: the third `Entry` variant, its `read_link` arm and the only test that reaches it serve a future that is not in front of this change. A defined answer for a symlink is available at one line (`panic!("unexpected symlink {path}")`, the same stance the recorder already takes on an unreadable entry), which would also stop it silently comparing equal across a link, and which a later change that does create symlinks would have to replace deliberately. The one real constraint, that the walk must not follow a link out of the root, is kept by using `DirEntry::file_type` instead of `metadata`; a one-line comment at that call already says so.
      **Measured:** nothing; by reading, `Entry::Link` is built only by the one symlink test (`grep -n Link` shows lines 631, 664 and 837 only), so removing both leaves no other test depending on it. Not mutated.
      **Judgement:** a stylistic preference, not a defect. The code is correct as written; rejecting this with Decision 5's argument is a reasonable outcome.
      **Rejected.** The argument is now in design Decision 5, which also
      corrects the sentence quoted above. The isolation test records a home
      the crate lays out, not one this file controls. With a panic on any
      symlink, a radicle version that put a symlink in a fresh home would turn
      the isolation test red at `tree_record(path_b)`, for a reason that has
      nothing to do with isolation. The recorder would then need rewriting
      before the test measured anything again. A `Link` kind keeps the test
      measuring isolation across that change. The saving is also smaller than
      it looks. A panic arm needs its own test to be pinned, which is the gap
      `design-review.md` raises for the unreadable-entry panic, so what it
      removes is the `Link` kind and three lines, not the test. The symlink
      test is also not idle: dropping the `changed:` arm of `record_changes`
      turns it red ("retargeting a link must show as a change"), measured on
      this tree.
