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

Every task below is verified by the core unit tests,
`nix build ./radicle#checks.x86_64-linux.unit-tests -L --out-link result-test`,
read for the named tests rather than the count. Two embedded-node tests fail
in the specflow pod on the 107-byte socket cap of the Nix build path; that is
the sandbox, not this change.

## 1. Make the change easy

- [x] 1.1 Split `SettingsStore::load()` into `readStored()` (the file's object
      as on disk) and a pure `normalised(stored)`, with `load()` their
      composition, in a commit of its own; verified by the suite passing
      unchanged on that commit.

## 2. A write changes only the key it names

- [x] 2.1 Write the file-on-disk tests for "A settings write changes only the
      key it names" and run them against the old `set()`; verified by
      watching them fail (the list is `design.md`, Decision 7).
- [x] 2.2 Make `set()` read the stored object, assign the one key and save
      that object, refusing before anything is written; verified by
      `a_write_of_one_key_keeps_an_unknown_mode_and_an_unknown_key`,
      `a_write_keeps_a_known_key_whose_value_is_not_a_string`,
      `writing_the_key_that_held_an_uninterpretable_value_replaces_only_that_value`
      and `a_refused_write_leaves_the_file_as_it_was` passing.
- [x] 2.3 Pin the seed picker's path; verified by
      `a_seed_stored_by_set_remote_seed_keeps_the_files_other_members`
      passing.

## 3. The reply is the settings as read

- [x] 3.1 Return `normalised(stored)` from an accepted `set()`; verified by
      `the_reply_is_the_normalised_view_while_the_file_keeps_its_members`
      (store) and
      `a_settings_write_replies_with_what_get_settings_reports_not_the_file`
      (module boundary) passing.

## 4. A write over an unreadable file

- [x] 4.1 Verified by
      `a_write_over_a_file_that_is_not_json_replaces_it_with_only_the_key_written`,
      `a_write_over_json_that_is_not_an_object_replaces_it_with_only_the_key_written`
      and `the_first_write_creates_a_file_holding_only_that_key` passing. The
      "cannot be read" case (a file that exists but cannot be opened) is
      covered by the same branch of `readStored()` as an absent file (the
      `std::ifstream` fails to open), and has no test of its own: a mode-000
      file is still readable by root, so such a test would pass or fail
      depending on who runs the suite. Left to the `tester` to judge.
      Tester: tested, with a file that does not depend on the user: a symlink
      pointing at itself, which `open(2)` refuses (ELOOP) and `rename(2)`
      replaces, in
      `a_write_over_a_file_that_exists_but_cannot_be_opened_replaces_it`. It
      fails against a `set()` that refuses when the path exists but cannot be
      opened, which no other test does.

## 5. The MODIFIED requirement

- [x] 5.1 "A settings file that cannot be read yields defaults" changed in
      wording, and now requires the no-path refusal to name `XDG_DATA_HOME`
      and `HOME`, which the message `set()` already returned does; no code
      changed for it. Its four scenarios are the existing store tests
      (`a_missing_settings_file_yields_defaults_rather_than_an_error`,
      `a_corrupted_settings_file_yields_defaults_rather_than_a_dead_module`,
      `unknown_keys_in_the_file_are_ignored_rather_than_surfaced`,
      `a_store_with_no_path_still_reports_defaults_rather_than_failing`),
      verified passing unchanged.

## 6. A write drops a member nested too deep to keep

- [x] 6.1 Name the five keys once (`kKnownKeys`, `isKnownKey()`), have
      `normalised()` build its defaults from them and `set()` refuse against
      them, and read the file only after every refusal, in a commit that
      changes no behaviour; verified by the suite passing unchanged apart
      from the two sandbox socket-path tests.
- [x] 6.2 Write the three depth scenarios' tests and run them against the code
      without the cap; verified by watching
      `a_member_129_deep_is_dropped_and_the_rest_kept` fail and
      `a_member_nested_far_deeper_still_lets_the_write_complete` end the test
      binary with SIGSEGV. `a_member_exactly_128_deep_is_kept` passed there,
      as it must, and was watched failing against a cap of 127 instead.
- [x] 6.3 Drop every member deeper than 128 in `set()`, with a depth walk that
      keeps its own stack; verified by the three depth tests passing, and by
      a recursive full-depth check, run as a mutation, crashing the binary in
      the 100000-deep test (`design.md`, Decision 9).
- [x] 6.4 Write each member's value compactly rather than with `dump(2)`;
      verified by the `NO SPEC:` test
      `a_kept_nested_member_does_not_grow_the_file_with_its_depth` failing
      against `dump(2)` and passing after, and by every existing file test
      passing unchanged.

## Workflow follow-up

- The closer archives the change once every review row is ticked.
