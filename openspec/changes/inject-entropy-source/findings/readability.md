# Readability findings — inject-entropy-source

Dimension: **readability** only (correctness, security and architecture are other instances).

Reviewed: `radicle/rust-ffi/src/profileinit.rs` (module doc, `open_system_entropy`,
`draw_seed`, `init_profile`, `init_profile_with_entropy`, `init_profile_inner`),
`radicle/rust-ffi/tests/profile_init.rs` (the entropy section and the signature
tests), and the `docs/rust-ffi.md` paragraph. `cargo fmt --check` is clean.

Clean areas: the production code reads well. `draw_seed` and
`open_system_entropy` are each one job; the comment on `draw_seed` says why
`read_exact` and not a hand loop; the one place the code overrides std's text
(`UnexpectedEof`) says why; the opener-not-reader choice and the
draw-before-`Home::new` ordering are both explained at the point where a reader
would ask. Names carry their meaning (`Scripted`, `opening`, `known_seed`,
`node_id_the_crate_derives_from`). `known_seed()` is built so no byte is zero
(37 is odd, `i` runs 1..=32), which is what makes a zero-padded lenient read
visible; that is good and is not commented, but needs no comment. Stale counts
elsewhere in `tests/profile_init.rs:136` ("other eight") and
`profileinit.rs:293` ("all nine tests") predate this change and are not
raised here.

- [ ] **`tester`** — `tests/profile_init.rs:1117-1140` — `a_refused_home_never_opens_the_entropy_source` dispatches on the same strings twice
      **Scenario:** the table at 1109-1115 names five cases, then `match case { "empty-home" => …, "relative-home" => … }` re-spells every name to build its setup, with `let mut alias` reassigned inside one arm and `other => unreachable!` as the fall-through. Renaming a case in the table and not in the match compiles and panics at run time; adding a sixth case needs two edits that nothing ties together. The table holds the cheap half of the data (message words) and the logic holds the rest.
      **Severity:** readability, genuine (this repo's "complexity in the data structure" habit; the file's other tables, `a_source_short_of_32_bytes_at_any_length_creates_no_key` and `an_interruption_anywhere_in_the_draw_is_retried`, keep the script in the table row, so this one is the odd one out). Shape that fixes it: each row carries its own setup (a `fn(&str) -> String` returning the target home, or the alias beside it), and the loop body has no `match`.

- [ ] **`tester`** — `tests/profile_init.rs:1128-1133` — the half-created fixture is written out a second time inside the table
      **Scenario:** `half_created_home` (335-345) already builds exactly this state (create, assert `created`, remove `config.json`) and has a doc comment explaining why it is built from a real init. The `"half-created-home"` arm repeats the three steps by hand, so a change to how that state is made (a second file to remove, say) must be made in two places and the second one has no comment saying why it is built this way.
      **Severity:** readability, genuine; small.

- [ ] **`dev-writer`** — `profileinit.rs:56` and `docs/rust-ffi.md:225` — "Eight lines of `File::read_exact`" is no longer true
      **Scenario:** the module doc argues for `/dev/urandom` over `getrandom` on the grounds that the hand-rolled draw is eight lines. This change replaces the single `os_seed` (10 lines on `main`) with `open_system_entropy` (8 lines, 88-95) plus `draw_seed` (17 lines, 109-125) plus an injection seam, so the justification quotes a size the code no longer has. The argument (no manifest change, no vendor rehash) still holds; only the number is stale, and the project's own rule is to measure a count or not write it. `docs/rust-ffi.md` carries the same phrase and was edited in this change a few lines below it.
      **Severity:** readability, genuine; trivial. Drop the number ("a few lines of").

- [ ] **`dev-writer`** — `profileinit.rs:60-68` against `profileinit.rs:361-374` — the same two claims are written twice in one file
      **Scenario:** the module-level section "Why the source can be swapped, and by whom" and the doc comment on `init_profile_with_entropy` both say that the seam exists because `/dev/urandom` cannot misbehave on demand, that `init_profile` is the seam with `/dev/urandom` fixed, and that the `extern "C"` entry point calls `init_profile` so no caller across the boundary can choose the source. A reader meets the second copy a few hundred lines after the first; the day one is edited the other disagrees. The module doc is the right home for the lenient-`read` history; the function doc is the right home for "tests only, not `extern "C"`".
      **Severity:** stylistic preference; no defect today.

- [ ] **`dev-writer`** — `openspec/changes/inject-entropy-source/design.md:87-91` and `tasks.md:46-51` — claims `cbindgen` emits nothing for the seam, but no `cbindgen` exists in the tree
      **Scenario:** `grep -rl cbindgen radicle` finds no file; `tests/profile_init.rs:1319-1320` says the header "is checked in by hand", and the tester's own note in task 1.8 repeats that. design.md Decision 1 and the first half of task 1.8 still give "cbindgen does not emit it" as the reason the header cannot change. A reader trying to verify the reason finds there is nothing to run. The conclusion (the header is unchanged because nobody edited it, and the function is not `extern "C"`) stands; the stated mechanism does not.
      **Severity:** genuine but a documentation inaccuracy, not code. The design-reviewer holds decisions; raised here because it is a false "why" in a comment-like place.

- [ ] **`tester`** — `tests/profile_init.rs:1179-1186` — the length table repeats each row's numbers in its name and carries a bare `bool`
      **Scenario:** `("ends-after-31", 31, false)`: the name restates the second and third fields, and a `bool` named `fails` says what the row does only at the loop's `if fails`. Changing `31` to `30` and leaving the name gives a failure message ("ends-after-31: …") that names the wrong length. The file already has `Step` for exactly this; a row of `(supplied, Step::Fail or nothing)`, with the case label built by `format!`, cannot drift.
      **Severity:** stylistic preference.

- [ ] **`tester`** — `tests/profile_init.rs:581-591` and `:820-833` — two helpers that list `<home>/keys`, one panicking on a missing directory and one returning empty
      **Scenario:** `keys_snapshot` expects the directory (`expect("the keys directory must exist")`) and `files_under_keys` treats `NotFound` as "none". Both are correct for their callers, but nothing says why the two differ, and a reader adding a third caller has to work out which to copy. A one-line note on `files_under_keys` ("absent is a valid answer here: a failed draw may leave no `keys/` at all") would settle it; `keys_snapshot().keys()` could also replace the second listing loop.
      **Severity:** stylistic preference; no defect.
