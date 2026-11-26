# Design review: settings-write-unknown-keys

Reviewed: `design.md` Decisions 1-8, Non-Goals and Risks against
`radicle/src/settings_store.cpp` and `.h` (`readStored`, `normalised`, `load`,
`set`, `save`), the spec delta, `tasks.md`, and `gh issue view 104` (read
fresh).

The code takes every recorded decision. `set()` reads `readStored()` once,
judges the key against `normalised(stored)`, saves `stored`, and replies
`normalised(stored)` (Decisions 2, 3). Nothing is added (5), the unreadable
case replaces rather than refuses (4), and the type is `nlohmann::json` (6).
The `load()` split is its own commit (`a534222`), as Decision 2 says. The
issue's "Done when" is met: the named file, one unrelated `set`, an on-disk
assertion, a requirement in `module-settings`, and recorded mutation evidence.
No contradiction with the issue. Decision 1 argues the open question, and
Decision 8 keeps the #63 line the issue drew.

What is left is the design falling behind the spec and tests that landed after
it.

- [x] **`dev-writer`** — `design.md:202-211` (Risks) and `:134-143` (Decision 6)
      describe a spec that no longer exists. Risks says the large-number case
      is "reported to the spec-writer rather than guarded here" and that
      duplicate member names have the "Same parser, same silence in the spec".
      Commit `38bcd91` added the requirement "A settings write keeps each
      member's value, not the file's text". It requires a repeated member to be
      kept once with its last value, and it carves out numbers beyond a
      binary64 double. Both Risks bullets are now false: the spec is not silent
      on either, and the duplicate case has a requirement and a test. That
      requirement has no Decision entry. The choice to keep values, not text,
      is mentioned only inside Decision 6, which answers a different question
      (`json` against `ordered_json`). Rewrite the two Risks bullets to point
      at the requirement, and say in Decision 6 or a new entry that this
      requirement is why the text is not preserved.
      **Fixed** in the commit that flips this box: Decision 6 now opens with
      "A settings write keeps each member's value, not the file's text" as
      the reason the file is re-serialised rather than edited in place (a
      text-splicing writer is the one that leaves a duplicate member named
      twice), and derives the duplicate and binary64 specifics from it. Both
      Risks bullets now point at that requirement and the duplicate one at
      its test. A design document, so no test applies.

- [x] **`dev-writer`** — `design.md:145-178` (Decision 7) is the mutation
      evidence for the guard, and it predates three tests. The list of tests
      that went red against the old `set()` omits
      `the_file_layout_does_not_change_what_a_write_keeps`,
      `a_member_the_file_names_twice_is_kept_once_with_its_last_value` and
      `a_write_over_a_file_that_exists_but_cannot_be_opened_replaces_it`. The
      comment block above the tests in `test_settings_store.cpp` says "every
      test below except the refused-write one went red", so it asserts the
      same thing. Decision 2 says removing the fix turns red "exactly the
      tests listed under Decision 7", which is now incomplete. The
      unopenable-file test's own mutation (a `set()` that refuses when the
      path exists but cannot be opened) is recorded only in `tasks.md:65-70`,
      which is archived with the change. Re-run the old shape and the refusal
      shape, and put the full lists in Decision 7. This is a thin entry, not a
      contradiction.
      **Fixed** in the commit that flips this box, from two re-runs on the
      final code. The old shape (`auto stored = load();` in `set()`) turned
      red 16 tests besides the two sandbox socket-path ones: the three named
      here, the four depth tests added since, and the nine already listed;
      `a_refused_write_leaves_the_file_as_it_was` stayed green. The refusal
      shape (an `lstat` that succeeds plus an `ifstream` that fails, returning
      an error) turned red only
      `a_write_over_a_file_that_exists_but_cannot_be_opened_replaces_it`.
      Decision 7 now lists both. The test file's section comment is the
      `tester`'s box in `readability.md`; measured here, its claim ("every
      test below except the refused-write one went red") holds for every
      test in its section.

- [x] **`dev-writer`** — Decision 4 (`design.md:101-120`) does not name the
      "exists but cannot be opened" case, though the spec says "cannot be read"
      and the code handles it through the same `if (!in)` branch as an absent
      file. Nothing in Decision 4 records that `rename(2)` replaces the
      unopenable path itself, which is why replacing works, or why the test
      uses a self-pointing symlink and not a mode-000 file (root can read one).
      Both are choices a reader would plausibly make differently. Today they
      sit only in `tasks.md` and in a test comment. Add one paragraph to
      Decision 4. It costs the entry nothing and keeps the reasoning where
      `settings_store.cpp`'s next toucher reads the design.
      **Fixed** in the commit that flips this box: Decision 4 has a paragraph
      on a file that exists but cannot be opened (same `readStored()` branch
      as an absent one; `save()` renames a temporary over the path, so it
      replaces the entry without opening or following it), and on why the
      test uses a self-pointing symlink rather than a mode-000 file.

- [x] **`dev-writer`** — suggestion, `settings_store.cpp:384-390`. The unknown-key
      refusal is judged against `normalised(stored).contains(key)`, not
      `stored`. Decision 2 states this ("judges the key against
      `normalised(stored)`") but gives no reason and no alternative. The code
      comment gives it: the keys of `normalised` are exactly the five the
      store knows. Judging against `stored` would refuse `radHome` on a fresh
      file, which is the first-write case. One sentence in Decision 2 naming
      that alternative and why it fails would complete the entry.
      **Fixed** in the commit that flips this box, with the code moved on
      since: `6c43102` replaced `normalised(stored).contains(key)` with
      `isKnownKey()` (architecture and readability review), and Decision 2
      now names both `stored.contains(key)` (refuses the first write of a
      key) and `normalised(stored).contains(key)` (right only by the
      defaults literal's coincidence) as the alternatives ruled out.
