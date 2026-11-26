# spec-test review: settings-write-unknown-keys

Reviewed: the delta spec (four ADDED requirements, one MODIFIED), the live
`module-settings` spec it modifies, issue #104, `radicle/tests/test_settings_store.cpp`
and the two new tests in `radicle/tests/test_radicle_impl.cpp`. Arrival check
passed (own `worktree-agent-*` tree). `openspec validate settings-write-unknown-keys --strict`
is valid.

**Part 1, coverage.** Every scenario has a test, at a layer that can see it:
the store-level scenarios by `SettingsStore::set` tests that read the
file back with their own parse (`ScratchSettings::onDisk`, `contents`), the
`setRemoteSeed` scenario and the module-boundary reply by tests in
`test_radicle_impl.cpp`. The "cannot be read" clause, which the dev left
untested, now has a test (`a_write_over_a_file_that_exists_but_cannot_be_opened_replaces_it`,
self-symlink, user-independent, with its precondition asserted). The four
scenarios of the MODIFIED requirement are the unchanged existing tests. The
refused-write test compares bytes with a deliberately odd layout, so a rewrite
with the same members is seen. No scenario is untestable as written. The
requirement texts pass the strict length check. The new requirements agree with
the live ones ("five named keys", "stored mode ... MUST NOT rewrite the file",
"A successful write returns the whole settings object"); no staleness against
#104, whose "done when" is met by decision plus test. The spec also adds the
unreadable-file and reply requirements, which the issue does not state but
which follow from its "read the raw object, set the one key" shape.

**Part 2, mutations** (command for both: `nix build ./radicle#checks.x86_64-linux.unit-tests --out-link result-test`,
failures read from `nix log <drv>` saved to a file and grepped for `FAIL`, since the
tool result truncates the suite output). Both mutations are restored; `git status`
is clean.

1. `set()` starts from `load()` instead of `readStored()` (the issue's bug).
   14 failed: the 2 sandbox socket-path tests the overlay names, plus all of
   `a_write_of_one_key_keeps_an_unknown_mode_and_an_unknown_key`,
   `a_write_keeps_a_known_key_whose_value_is_not_a_string`,
   `writing_the_key_that_held_an_uninterpretable_value_replaces_only_that_value`,
   `the_reply_is_the_normalised_view_while_the_file_keeps_its_members`, both
   not-JSON/not-object replacement tests, the unopenable-file test, the layout
   test, the duplicate-member test, `the_first_write_creates_a_file_holding_only_that_key`,
   and at the module layer `a_seed_stored_by_set_remote_seed_keeps_the_files_other_members`
   and `a_settings_write_replies_with_what_get_settings_reports_not_the_file`.
   Killed. `a_refused_write_leaves_the_file_as_it_was` passed, as the test
   file's own comment says it must.
2. A `save(stored)` inserted before the unknown-key refusal, so a refused write
   rewrites the file with the same members. 3 failed: the 2 sandbox tests and
   `a_refused_write_leaves_the_file_as_it_was`. Killed, by the byte comparison.

Read-only judgement on the rest: no test asserts something the implementation
produced; expectations are literals parsed by the test. The duplicate-member
and number-spelling scenarios hold for any implementation that parses and
re-serialises with this JSON library, so they pin the library's behaviour as
much as the code's; they fail only for a text-splicing write, and the spec
asks for exactly that, so this is an observation, not a defect. Not covered, by
design of the scenarios and not worth a box: a `mode` write through the
module's `setSetting` (the repointing path) is not checked for keeping
unknown members, only a `radHome` write is; and a refused write over an absent
file is not stated.

**Part 3, `NO SPEC:`.** One marker, carried from #101 and not introduced here.

- [x] **`spec-writer`** — `test_settings_store.cpp:926`, `a_store_with_no_path_says_why_it_cannot_write`
      The marker says the spec is silent on what the no-path refusal says. It
      is not silent: the MODIFIED requirement "A settings file that cannot be
      read yields defaults" requires a refusal "naming the problem", yet its
      scenario "A store with no path reports defaults and refuses to write"
      only requires `{"error":"..."}`. The test pins that the message names
      `XDG_DATA_HOME` and `HOME` and does not end on a space, which no scenario
      states, so a message that names no cause satisfies every scenario while
      breaking the requirement's own words.
      **Scenario:** `set()` returning `makeError("could not write settings to ")`
      for an empty path passes the spec's scenario and fails only this test.
      **Severity:** low. The change re-states this requirement, so it is the
      natural place to add the clause to the scenario or to say the marker is
      stale. **Measured:** not mutated.
      **Outcome (spec-writer): fixed in the spec.** The MODIFIED requirement
      "A settings file that cannot be read yields defaults" now says the
      refusal names both `XDG_DATA_HOME` and `HOME` (replacing "naming the
      problem"), and its scenario "A store with no path reports defaults and
      refuses to write" checks it: the message MUST contain `XDG_DATA_HOME`,
      and MUST still contain `HOME` once every `XDG_DATA_HOME` is removed. The
      second clause is worded that way because `HOME` is a substring of
      `XDG_DATA_HOME`, so the test's `LOGOS_ASSERT_CONTAINS(message, "HOME")`
      passes for a message naming only `XDG_DATA_HOME`. The trailing-space
      check stays unspecified (it is implied by naming a cause, not a separate
      behaviour). **Left for `tester`:** strengthen the `HOME` assertion to
      the scenario's form, and remove the now-stale `NO SPEC:` comment. The
      implementation's current message already satisfies the amended
      scenario; no code change is needed.
      **Outcome (tester): fixed in the test.** `a_store_with_no_path_says_why_it_cannot_write`
      now removes every `XDG_DATA_HOME` from the message before asserting it
      still contains `HOME`, the scenario's form. Mutated to a message that
      names `XDG_DATA_HOME` and not `HOME` (`"... because XDG_DATA_HOME is not
      set"`): the new test went red with the message printed, and the old
      plain `contains("HOME")` would have passed it, since `HOME` is inside
      `XDG_DATA_HOME`. The stale marker is gone from that test. One
      `NO SPEC:` marker remains, narrowed to what is still unspecified: the
      trailing-space check moved to its own test,
      `the_refusal_of_a_store_with_no_path_does_not_end_on_a_space`, because a
      marker must not be dropped and the spec still says nothing on how the
      sentence ends. It stayed green under the HOME mutation, as it should.

**Parts 4 and 5.** No requirement moved between capabilities. The spec is
self-consistent and testable; the one defect is the box above.
