# Security review: settings-write-unknown-keys

Dimension reviewed: **security** only. The file under `~/.local/share/radicle-module/settings.json` (or the profile's `XDG_DATA_HOME`) is untrusted input here: a hand edit, a newer build, or another program can write anything into it, and this change is the first to carry arbitrary members of it from a read back out to a write.

- [x] **`dev-writer`** — `radicle/src/settings_store.cpp:384,405` (`readStored()` then `save(stored)` -> `settings.dump(2)` at `:319`) — a deeply nested unknown member now reaches the recursive serializer, which crashes the module and can write a file hundreds of megabytes long
      **Scenario:** the settings file holds `{"futureKey":[[[[ ... ]]]],"mode":"local"}` with the arrays nested N deep. `readStored()` parses it (nlohmann's parser is iterative and accepts any depth), and `set()` now keeps `futureKey` verbatim, so the next `setSetting`/`setRemoteSeed` calls `stored.dump(2)`. Before this change the member was dropped by `load()`, so a write never serialised it. Reproduced with the same nlohmann 3.11.3 header against a program doing `parse` then `[...] = ...` then `dump(2)`: N=1000 dumps 2 MB; **N=10000 (a 20 KB file) dumps 200 MB** (indent grows with depth, so output is quadratic in N, and that is written to the temp file and renamed over the settings file; every later `getSettings` then slurps 200 MB into a `stringstream` before parsing); **N=20000 and above segfaults** (exit 139) inside `dump`, which recurses once per level. The segfault takes the whole core module process down on a settings click, and it recurs on every restart, since the poisoned file is still there and the write is how a user would try to fix it.
      **Measured:** `tmp/deep 1000` ok, `10000` ok at 200,040,071 bytes, `20000`/`30000`/`100000`/`1000000` segfault; the parse step succeeded in every case, including 1,000,000 levels. Severity: low to medium. It needs a file this build did not write (the module itself never nests), so it is a robustness hole reachable from on-disk state, which the overlay names as untrusted, not a remote one. Genuine defect, not style. Neither the spec nor `design.md`'s Risks section names a depth or size bound; Decision 6 and the Risks entries cover number rounding and duplicate names only.
      **Spec decision (spec-writer), box left for `dev-writer`:** the same
      decision as `correctness.md`'s first entry. The new requirement "A
      settings write drops a member nested too deep to keep" keeps a member
      the write did not name only while its depth is 128 or less, drops a
      deeper one from the file, and requires the write to complete and reply
      as any accepted write however deep the file nests; the 100000-deep
      scenario is the crash case. The output-size half is bounded by the same
      cap and otherwise left to the dev-writer (see `correctness.md`'s second
      entry).
      **Fixed** in `21a6656`, by the same change as `correctness.md`'s two
      entries: members deeper than 128 are dropped by a walk that does not
      recurse, and kept members are written compactly. The crash is
      `a_member_nested_far_deeper_still_lets_the_write_complete` (SIGSEGV
      before the fix, green after); the output size is
      `a_kept_nested_member_does_not_grow_the_file_with_its_depth` (red
      against `dump(2)`). `design.md` Decisions 6 and 9, and a Risks entry
      for the silent drop.

Clean areas, in prose:

- **Key and value reaching the write.** `stored[key] = value` is only reached after `normalised(stored).contains(key)` limits `key` to the five known names and `validate()` has run; an arbitrary key from the view cannot become a new member. The userinfo refusal on `remoteSeed` and the 108-byte `radSocket` cap are unchanged and still run before the write. A refused write returns before `save()`, so no bytes reach disk on refusal.
- **Credential handling.** A `remoteSeed` carrying `user:pw@` that a hand edit placed in the file was already copied through by the old normalising read and written back by the old save; this change neither adds nor removes that. The error strings added or changed name no URL and no file path beyond the existing `could not write settings to <path>`, which is unchanged.
- **Type confusion in `normalised()`.** The `get<std::string>()` on `mode` is guarded by the `value.is_string()` copy loop, and `defaults` always holds strings, so a non-string `mode` in the file cannot throw there. `stored.items()` is only called on an object, which `readStored()` guarantees.
- **Concurrency and file replacement.** The `.tmp`-then-rename path, its fixed temp name and the lost-update window between two instances are unchanged from before (the design records the window honestly). A symlink at the settings path is replaced by the rename rather than followed; the new test builds a self-referential link in its own scratch directory and touches nothing outside it.
- **Dependencies.** None added; `<sstream>` and `<unistd.h>` are standard headers used by the tests. No FFI or Rust files changed, and the interface to the view stays `string` in, JSON out.
- **Tests.** Every new test writes under `ScratchSettings`' own directory and none reads `HOME` or a real Radicle home.
