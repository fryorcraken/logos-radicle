# Findings: readability (code-reviewer)

Reviewed: `radicle/src/settings_store.cpp` and `.h` (the `readStored` /
`normalised` / `load` split, `set()`), the two test files, and the cross-references
into the moved code. Not built or run: the readability dimension needed no
mutation, and the unit-test build is a long Nix build.

Clean: the split itself reads well. `readStored()` / `normalised()` / `load()`
each have one job and a name that says it; `normalised` being `static` and pure
is stated in its header comment and is what lets `set()` derive the reply
without a second read. The `set()` comments say why (the promise an unknown mode
stays as found, why a damaged file is replaced not refused, why no defaults are
written), not what. Test names state the property, each test's starting file is
written by hand and each assertion parses the file in the test, and the reasons
for the odd fixtures (self-referencing symlink, repeated member, multi-line
layout) are written next to them. The `readStored()` / `normalised()` header docs
agree with the bodies.

- [x] **`dev-writer`** — `radicle/src/radicle_impl.h:187`, `radicle/src/radicle_impl.cpp:100`, `radicle/tests/test_radicle_impl.cpp:1292`, `:2147`, `docs/e2e.md:393` — cross-references to `SettingsStore::load()` now land on a two-line function
      **Scenario:** each of these says "See `SettingsStore::load()`" for the argument (why the default is `explore`, why an unknown mode becomes `explore`, "for the whole argument"). This change moved that body, and its long comment, into `normalised()`; `load()` is now `return normalised(readStored());`. A reader following any of the five pointers finds no argument and has to guess where it went. The change fixed the one in-body reference ("the fallback below") but not these.
      **Severity:** minor, a genuine defect of the stale-pointer kind (the rename is this change's, so the update is too). Point them at `SettingsStore::normalised()`. `docs/e2e.md:393` ("`load()`'s default mode") is the one a reader outside the code will follow.
      **Fixed** in `6c43102`: all five now name `SettingsStore::normalised()`.
      Comments only, so no test can see it; the remaining `load()` mentions in
      the tests say what `load()` does (it still normalises), not where the
      argument lives, and were left.

- [x] **`dev-writer`** — `radicle/src/settings_store.cpp:390` — `set()` builds the whole normalised view only to ask "is `key` one of the five"
      **Scenario:** `if (!normalised(stored).contains(key))` runs the defaults construction, the copy loop and the mode fallback, then reads one bit; the comment "Judged against the normalised view, whose keys are exactly the five this store knows" exists to justify that indirection. It also forces `readStored()` to run at the top of `set()`, before the key and value are validated, though nothing before `stored[key] = value` needs the file. A fifth `kKey…` constant in `settings_store.h:79-83` would have to be added to `normalised()`'s `defaults` for `set()` to accept it, which is the invariant carried by a side effect rather than stated. A named known-keys check (or list) shared by `normalised()` and `set()` would delete the comment and let `readStored()` sit next to the assignment it feeds.
      **Severity:** stylistic. Works as written and is correct; this is the "complexity in the data structure, not the logic" preference, and the cost is a hidden coupling, not a bug.
      **Fixed** in `6c43102`, a refactor that changes no behaviour: `kKnownKeys`
      states the five keys once, `normalised()` builds its defaults from it,
      `set()` refuses with `isKnownKey()`, the justifying comment is gone, and
      `readStored()` now sits after the last refusal, beside the assignment
      it feeds. The refusal is still pinned by
      `an_unknown_setting_key_is_refused_by_name` and
      `a_refused_write_leaves_the_file_as_it_was`; the suite passed unchanged
      on that commit (two sandbox socket-path failures only).

- [ ] **`tester`** — `radicle/tests/test_settings_store.cpp:411-427` — the section comment records a mutation result that `design.md` does not back for three of the tests
      **Scenario:** the block says "Every test below except the refused-write one went red" under the old `set()`. `design.md` Decision 7 lists the tests watched failing, and `the_file_layout_does_not_change_what_a_write_keeps`, `a_member_the_file_names_twice_is_kept_once_with_its_last_value` and `a_write_over_a_file_that_exists_but_cannot_be_opened_replaces_it` are in neither that list nor the "stayed green" paragraph. The comment is either an unmeasured claim for those three or `design.md` is out of date; a later reader cannot tell which. A measured result recorded in a code comment is also the self-invalidating-state shape the repo's "Keeping documents true" rule warns against: it is true on the day it is written and silent when a test is added.
      **Severity:** minor. Either measure the three and extend Decision 7, or drop the count from the comment and keep only the reason the tests exist.

- [x] **`dev-writer`** — `radicle/src/settings_store.h:106-114`, `radicle/src/settings_store.cpp:371-383`, `:179-192` — the rule "an absent, unreadable or non-object file is replaced by one holding only the key" is written out three times
      **Scenario:** the `set()` header doc, the `set()` body comment and the `readStored()` header each restate the unreadable-file behaviour and the reason (a write is the way out). A future change to that rule has three places to update, and `design.md` Decision 4 holds a fourth. The `set()` body comment is the longest and mostly repeats the header.
      **Severity:** stylistic preference. Keep the *why* once in the body comment and let the header doc state the contract in a sentence.
      **Fixed** in `6c43102`: the `set()` header doc states the contract in a
      sentence and points at the body for the unreadable-file case and the
      reason; the `readStored()` header now says only what it returns; the
      body comment keeps the why. `design.md` Decision 4 stays, as the
      archived reasoning rather than a code comment. Comments only, so no test
      can see it.
