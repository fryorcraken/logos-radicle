# Design review: isolation-untouched-home

Reviewed: `design.md` Decisions 1-8 and Risks against
`radicle/rust-ffi/tests/profile_init.rs` (`two_homes_get_two_different_identities`,
`tree_record`, `record_changes`, `holds_file`, the recorder's own tests,
`reporting_encryption_leaves_the_key_untouched`), `docs/rust-ffi.md`'s
isolation paragraph, and issue #59 (read fresh with `gh issue view 59`).

The code takes every recorded decision: B is created first and recorded, A's
absence is checked before and after, the record is recursive with directories as
entries, `keys_snapshot` is gone and the keys test runs on `tree_record`,
failures print a path list, symlinks use `file_type` without following, and
unreadable entries panic. Nothing in the code contradicts the issue; the
issue's two "done when" conditions are met and Decision 6 argues the input
dependence by mutation. The findings below are gaps in the record, not
contradictions.

- [x] **`dev-writer`** — `design.md:96-98` (Decision 5) is now false. It says profile creation makes no symlinks, "so this case is never exercised". The later commit `594dc34` added `the_recorder_does_not_follow_a_symlink_out_of_its_root`, which exercises exactly that branch with a hand-built link. The decision's "defined answer for every entry" is pinned by that test, and nothing in `design.md` says so. Fix the sentence, and name the test as the guard for the symlink rule.
      **Fixed** in the commit that ticks this box: Decision 5 now says neither
      isolation test meets a symlink, and names
      `the_recorder_does_not_follow_a_symlink_out_of_its_root` as the guard for
      the rule. That test is red when the `changed:` arm of `record_changes` is
      dropped ("retargeting a link must show as a change"), measured on this
      tree. Decision 5 also now records the alternative the architecture review
      raised (panic on any symlink) and why it was ruled out.

- [x] **`dev-writer`** — `design.md` Decision 6 and `tasks.md` omit the recorder's own tests (`the_recorder_lists_every_entry_at_every_depth_with_directories`, `the_recorder_names_each_kind_of_change_by_path`, the symlink test). They landed in commit `594dc34`, after the design, and no decision or task records why the instrument is pinned separately or what removing each arm of `record_changes` turns red. The test file's own comments claim "dropping the `changed` arm, the `removed` arm or the `added` loop each turns exactly one row red". That is a mutation claim with no evidence in `design.md`, where Decision 6 requires it for a guard. Either record the measurement or soften the comment. This is a gap, not a contradiction.
      **Fixed** in the commit that ticks this box, by recording the
      measurement: Decision 6 gains "The recorder is pinned by tests of its
      own", which says why the instrument is pinned separately and gives a
      table of each `record_changes` arm dropped in turn against
      `cargo test … --test profile_init the_recorder`. `tasks.md` gains 1.2.
      The measurement does not bear out the comment's "exactly one row":
      dropping `changed:` also turns the symlink test red, and the table test
      stops at its first red row, so "exactly one" is not something the run can
      show. The comment is left as it is here, because it sits on the lines of
      the `tester`'s own open box in `readability.md` ("the doc comment on
      `the_recorder_names_each_kind_of_change_by_path` overclaims"), and the
      `tester` owns that test.

- [x] **`dev-writer`** — `design.md:150` (Risks) says the missing mode check "is reported to the `spec-writer` as a finding about the spec". No such finding exists in the change folder (there is no `findings/` file for it, and `grep` for `chmod`/`mode` finds only this paragraph). The concession names a mechanism that is not there. Either write the finding where the `spec-writer` will read it, or file an issue and cite it by name. A 0644 chmod of the sibling's private key is the one regression of this kind the test cannot see, and it should not be recorded only in an archived design.
      **Fixed**: the finding now exists as the `spec-writer` box in
      `security.md`, and its outcome brought permissions into the requirement.
      The record now carries each entry's permissions (`f3c2bc5`), and the
      Risks entry is gone. In its place, the commit that ticks this box adds
      Decision 9, which records why permissions are compared, what was
      considered, and the security review's 0644 measurement. A Risks entry now
      says that `std::fs::Permissions` holds only the read-only flag off Unix.
      The test that fails without the permissions arm is the `tester`'s, still
      to come: dropping that arm leaves all 22 tests in `profile_init.rs`
      green on this tree, measured. `tasks.md` 1.3 stays unticked for that
      reason.

- [x] **`dev-writer`** — suggestion: Decision 5's "panic on an unreadable entry" has no test behind it. It is justified in prose ("a record with a gap would compare equal"), but no test shows the panic. Say so in the entry as a concession, or add a small test. Decision 5 also lacks the cost of the alternative it rejected (skip and continue).
      **Fixed**, as a concession, in the commit that ticks this box. Decision 5
      now says the panic has no test, and why: a `chmod 000` entry is still
      readable to a process running as root, so the test would pass or fail on
      who runs it. It also gives what skip-and-continue would cost. A
      directory unlistable at both records would hide every write beneath it,
      and a file that became unreadable between them would show as `removed:`
      while still present.

- [x] **`dev-writer`** — suggestion: Decision 2's measured numbers (11 entries, 139,264 bytes of SQLite) are tied to the radicle crate version. They are recorded as fact with no command to reproduce them, and `CLAUDE.md` says to name the command rather than the number. Add the one-line command (print `tree_record` of a fresh home once) and say they are the radicle-0.25.1 shape, so a later reader can tell when they have gone stale. Decision 6's closing paragraph is good: it says the preconditions are not shown to be the sole cause of any red.
      **Fixed** in the commit that ticks this box. Decision 2 now says the
      numbers are the shape of a radicle 0.25.1 home (the version
      `radicle/rust-ffi/Cargo.lock` names), and gives the command and the
      temporary `eprintln!` that re-measure them. They were re-measured on this
      tree that way: 11 entries, the same four database sizes, and the
      permissions now quoted (`keys/radicle` 0600, under umask 022).
