# Readability review: isolation-untouched-home

Dimension: **readability only** (the prompt named it; correctness, security and
architecture are other instances). Reviewed the diff against `origin/main`:
`radicle/rust-ffi/tests/profile_init.rs`, `docs/rust-ffi.md` and the change
folder.

Clean areas: the names are good (`tree_record`, `record_changes`, `holds_file`,
`Entry`, `recorder_tree`) and each function does one job. The comments say why
rather than what: why B is created first, why a failure prints paths and not the
maps, why symlinks are not followed, why an unreadable entry panics. The old
`keys_snapshot` is gone from the tree and nothing still refers to it. The
numbers in `design.md` Decision 2 and 4 add up (28,672 + 69,632 + 20,480 +
20,480 = 139,264). `docs/rust-ffi.md` reads true against the test, and the
`design.md` Risks are honest about the mode and timestamp gap. The findings
below are small.

- [ ] **`tester`** — `radicle/rust-ffi/tests/profile_init.rs:768-772` — the doc comment on `the_recorder_names_each_kind_of_change_by_path` overclaims
      **Scenario:** it says dropping the `changed` arm, the `removed` arm or the `added` loop "each turns exactly one row red". By the table, `changed` appears in two rows (`a deep file's bytes change` and `one of each at once`), `removed` in three and `added` in three. A reader who trusts the comment will believe a failing case is the only affected one. The loop aborts at the first failing case, so the output hides the others and makes the comment look true. **Measured:** with the `changed` arm deleted from `record_changes`, the test stops at `case 1: a deep file's bytes change`; case 6 would also fail but is never reached. Defect in a comment, not in the code. Say "at least one row" or name which rows.

- [ ] **`tester`** — `radicle/rust-ffi/tests/profile_init.rs:613-619`, `:875` — the new section header for the recorder swallows the tests that follow it
      **Scenario:** the file is organised under banner comments (`// ---` blocks at 353 and 521). The new "Recording a directory tree" banner opens at 613 and no banner closes it, so `reporting_encryption_leaves_the_key_untouched`, `an_unreadable_key_reports_a_problem_rather_than_unencrypted` and `a_home_with_no_key_reports_a_problem_rather_than_unencrypted` now read as members of the recorder section. They belong to the "Reporting whether an EXISTING key is encrypted" section opened at 521. Before the change the helper sat inside that section without a banner of its own. Move the recorder block so the encryption section stays contiguous, or add a banner before 875. Preference about structure, not a defect.

- [ ] **`tester`** — `radicle/rust-ffi/tests/profile_init.rs:76-79` — the "homes must not nest" precondition cannot fail
      **Scenario:** `fresh_home` puts each home at `<scratch dir>/home` under two different scratch dirs (`profile-init-iso-a` and `profile-init-iso-b`). One string cannot be a prefix of the other, so the assert holds for any implementation, and it compares strings rather than paths. It was inherited from the old test, but the change moved it to the front and labelled it a precondition, which presents it as a guard. Either drop it, or replace it with a check that can fail (the two scratch dirs differ). Preference, since it does no harm.

- [ ] **`tester`** — `radicle/rust-ffi/tests/profile_init.rs:826-833` — a dangling pronoun in the symlink test's comment
      **Scenario:** the doc comment's first line says "A symlink is recorded by its target and never followed." The next paragraph begins "What it must not do is read through the link". "It" has no referent: the first line's subject is the symlink, and the second means the recorder. Replace "it" with "the recorder". Preference.

- [ ] **`tester`** — `radicle/rust-ffi/tests/profile_init.rs:769-771` — the case table hardcodes its own length
      **Scenario:** `let cases: [(&str, Edit, &[&str]); 7]` makes whoever adds an eighth case edit a count that exists only to satisfy the compiler. A mismatch is a compile error rather than a silent skip, so nothing breaks. Binding to `&[...]` or a `Vec` removes the number. Preference, low.

- [ ] **`tester`** — `radicle/rust-ffi/tests/profile_init.rs:67-68` — "The reverse direction" does not say which direction
      **Scenario:** the doc comment calls the first half "node ids differ" and the second half "untouched". It then adds "The reverse direction is the absence of A", which is the B-creation-leaves-A-absent check, a third thing. A reader has to work out what it is the reverse of. Say "Creating B must also not bring A into being" and drop the direction language. Preference.
