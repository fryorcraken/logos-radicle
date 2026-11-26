# Findings: correctness

Reviewed `radicle/src/settings_store.cpp` and `.h` (the `readStored` /
`normalised` / `load` split and the new `set()`), the two callers in
`radicle_impl.cpp` (`setSetting`, `setRemoteSeed`), and the spec delta and
`design.md` against them. I built `settings_store.cpp` with its real
sources and nlohmann 3.11.3 (`-O0` and `-O2`) into a small harness with stubs
for `gitProbe`, `makeError` and `isError`, and fed `set()` hostile files. I did
not run the Nix unit-test build or any mutation of the C++ tests (the tests'
`logos_test.h` is not available outside it), so what the tests can and cannot
see is left to the spec-test reviewer.

Clean, by reading (only the nesting and read-only cases below were probed):

- `set()` ordering: unknown-key refusal, then `validate`, then the empty-path
  refusal, all before `save`, so a refused write cannot touch the file. The only
  side effect of a failed save is `makeParentDirs`, which `save` runs after
  every refusal.
- A key whose stored value is non-string (`radSocket: 5`, `gitPath: null`) is
  kept verbatim by a write of another key, reported as `""` in the reply, and
  replaced by a write of its own key.
- A stored uninterpretable `mode` survives a write of another key, is reported
  as `explore` in the reply and in the next `getSettings`, and is replaced by a
  write of `mode`. The reply equals `getSettings` (both are `normalised` of the
  same object).
- Absent file, empty path, a non-object, truncated or non-JSON text, trailing
  garbage and a directory at the path all read as an empty object, so the write
  replaces them with only the key written; a directory at the path fails at
  `rename` and is reported as `could not write settings`.
- Both writers (`setSetting`, `setRemoteSeed`) reach the same `set()`, and
  neither caller depends on the old reply shape (the reply is still exactly the
  five keys).
- Reads are unchanged: `normalised` is the old body of `load()` verbatim apart
  from the `const auto&` loop, and `readStored` is the old file handling.
  A 100000-deep nested value in the file reads without a crash.

- [x] **`dev-writer`** — `radicle/src/settings_store.cpp:404-405` — a write over a file holding a deeply nested unknown member crashes the host process
      **Scenario:** the settings file is `{"x":[[[[ ... ]]]]}` with the array
      nested 100000 deep (a 200 KB file, an unknown key `x`). `setSetting("radHome", "/h")`
      reads it (`parse` is iterative and copes), assigns, and `save` calls
      `settings.dump(2)`, which nlohmann implements recursively: the process
      dies with SIGSEGV (stack overflow), reproduced at `-O0` and `-O2`, and at
      50000 deep as well. The file is unchanged, so every later write dies the
      same way and the user can no longer change any setting from the UI; the
      crash takes the Basecamp host with it. **Before this change it was not
      reachable**: `set()` saved the normalised view, which holds only strings,
      so an unknown member was never dumped. It is new because preserving
      arbitrary members preserves arbitrary depth. The file is the user's own
      (a hand edit, or another program writing the same data dir), so this is
      low likelihood, but the project's overlay names on-disk state as
      untrusted input and asks for reachable crashes to be hunted.
      **Severity:** medium (crash, narrow trigger). **Fix is the dev-writer's
      to shape and needs a spec decision first:** the delta says every other
      member MUST keep its value and is silent on depth, so a depth cap on write
      (refuse, naming the file, or parse with a depth-limiting callback and treat
      the file as unreadable) contradicts "MUST NOT by itself cause the write to
      be refused" or "keeps every member" until the spec-writer says which gives.
      **Measured:** harness, `deep` input of 100000 and 50000: exit 139; 10000:
      completes (see the next entry); read-only (`all()`) at 100000: fine.
      **Spec decision (spec-writer), box left for `dev-writer`:** the new
      requirement "A settings write drops a member nested too deep to keep"
      gives way on "keeps every member", not on "not refused": of the members
      other than the one written, a write keeps every one of depth 128 or less
      and drops from the file every one deeper, and completes as any accepted
      write however deep the file nests. Depth counts arrays and objects on the
      deepest path, the value itself included (`[]` is 1). "A settings write
      changes only the key it names" now names that exception. Scenarios pin
      128 kept, 129 dropped, and 100000 completing with the rest kept. Reads
      are unchanged, since a member that deep holds no string. Note the depth
      check itself must not recurse per level, or it crashes where `dump` did.
      **Fixed** in `21a6656`: `set()` drops every member deeper than 128
      (`dropMembersNestedDeeperThan`, `kMaxKeptDepth`) before assigning, and
      `nestsDeeperThan()` walks with its own stack, stopping one level past
      the cap. `a_member_nested_far_deeper_still_lets_the_write_complete`
      ended the test binary with SIGSEGV (exit 139) before the fix and passes
      after; `a_member_129_deep_is_dropped_and_the_rest_kept` failed before;
      `a_member_exactly_128_deep_is_kept` failed against a cap of 127. A
      recursive full-depth check, run as a mutation, crashed the binary in the
      100000-deep test. Reasoning in `design.md`, Decision 9.

- [x] **`dev-writer`** — `radicle/src/settings_store.cpp:319` — `dump(2)` of a preserved nested member writes output quadratic in its depth
      **Scenario:** `{"x":[[[ ... ]]]}` nested 10000 deep is a 20 KB file;
      `setSetting("radHome", "/h")` replaces it with a 200,040,027-byte file
      (indentation grows with depth), and at 1000 deep a 2 KB file becomes a
      2,004,027-byte one. Every later read then parses 200 MB, and the write
      holds it in memory three times over (`stored`, the dump string, the
      stream buffer). Nothing is lost or wrong in value, but a write blows a
      small file up by a factor of 10000 and makes startup read it. Not
      reachable before this change for the same reason as the entry above.
      **Severity:** low (needs a deliberately nested file; resolved by the same
      depth decision as the entry above, or by `dump()` without indent for the
      kept members, which the spec's "whitespace is outside the requirement"
      permits). **Measured:** harness, depth 1000 → 2,004,027 bytes written,
      depth 10000 → 200,040,027 bytes written.
      **Spec decision (spec-writer), box left for `dev-writer`:** the depth cap
      in "A settings write drops a member nested too deep to keep" (128) bounds
      this but does not remove it: a kept member 128 deep still indents to
      roughly 128 times its compact size, and a file can hold many. The file's
      whitespace stays outside the contract, so how the kept members are
      serialised is the dev-writer's choice; no requirement is added on size.
      **Fixed** in `21a6656`: `save()` writes through `serialise()`, one
      member per line with each value compact, which is the same text as
      `dump(2)` for the flat all-string files this module writes and linear
      in a nested member's size. The `NO SPEC:` test
      `a_kept_nested_member_does_not_grow_the_file_with_its_depth` (a
      128-deep member; the rewritten file must stay under twice the
      original's size) failed against `dump(2)` and passes after; every
      existing file test passed unchanged. `design.md`, Decision 6.
