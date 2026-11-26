# Correctness review: isolation-untouched-home

Dimension: **correctness** only. Security, readability and architecture are other instances.

Arrival check passed (worktree under `.claude/worktrees/`, branch `worktree-agent-ab8e487bb3518545a`, no upstream). The diff is test code, docs and change documents; `radicle/rust-ffi/src/` is unchanged, so the review is of the test instrument, run by mutating the backend and the recorder.

## What was measured (all in this tree, every mutation reverted with `git checkout`)

`cargo test --test profile_init`: 22 passed. `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: clean.

Mutations to `init_profile_inner`, each run against `two_homes_get_two_different_identities` alone:

| Mutation | Result |
|---|---|
| append a byte to every file under every sibling home | red, 7 paths named: `cobs/cache.db`, `config.json`, `keys/radicle`, `keys/radicle.pub`, `node/node.db`, `node/notifications.db`, `node/policies.db` |
| create `home` in every sibling scratch dir | red at "creating an identity into home B brought home A into being" |
| copy the new key pair over every sibling's key pair | red: `changed: keys/radicle`, `changed: keys/radicle.pub` |
| the same key-pair overwrite, against `origin/main`'s version of the test | **green**, so the design's claim that the old test could not see this is true |
| null implementation (return `nodeId = home`, write nothing) | red, but by a panic inside the recorder (see finding 1) |

Mutations to the recorder in the test file: not descending into subdirectories turns three tests red (`the_recorder_lists_every_entry…`, `the_recorder_names_each_kind…`, and the isolation test's precondition), as intended. `reporting_encryption_leaves_the_key_untouched` stays green under it, correctly, since `keys/` is flat.

Design claims re-measured: B's record holds 7 files and 4 directories; the four SQLite files total 28672 + 69632 + 20480 + 20480 = 139264 bytes. Both match Decision 2 and Decision 4.

Clean areas: the recorder's three arms (directory, symlink, file) and `record_changes`'s three outcomes; the symlink test (never read through a link, retarget shows as `changed`); the preconditions (non-nesting, A absent, record holds the three named files); the ordering (B created and recorded before A); the docs paragraph in `docs/rust-ffi.md` against the test; no reachable panic on the production path (nothing outside `tests/` changed). The mutation tool (`cargo mutants`) was not run: it mutates `src/`, which this change does not touch, and design Decision 8 gives the same reason. It cannot see an added write to a path, which is the property here.

## Findings

- [x] **`tester`** — `radicle/rust-ffi/tests/profile_init.rs:85-93` — a creation that reports success and writes nothing is caught by a recorder panic, not by an assertion naming the cause
      **Scenario:** `init_profile` returns `{"created":true,"nodeId":"<home>"}` and creates no directory. `nid_b` is non-empty, so the `!nid_b.is_empty()` assertion passes; `path_a.exists()` is false; `tree_record(path_b)` then panics with `could not list …/profile-init-iso-b/home/ for the record: No such file or directory`. The test is red, which is right, but the message reads as a broken instrument rather than "home B was never created", and the failure sits on the helper's line, not the test's.
      **Measured:** the null implementation turns the test red at `profile_init.rs:652`, inside `record_under`. An `assert!(profile_exists(&home_b), …)` straight after B's creation would name it. Severity low; a diagnostics improvement, not a missed defect.

      **Outcome (`tester`): fixed.** `two_homes_get_two_different_identities` now asserts `profile_exists(&home_b)` straight after B's creation, with a message saying the identity was reported but left no profile. Mutation: `init_profile_inner` returning `{"created":true,"nodeId":<home>}` before `Home::new` turned the test red at that assertion ("creating an identity into home B reported … but left no profile there"), where before it panicked inside `record_under`. Restored.

- [x] **`tester`** — `radicle/rust-ffi/tests/profile_init.rs:665-670` — the recorder's fallback arm treats every non-directory, non-symlink entry as a regular file
      **Scenario:** `tree_record` is run over a home that holds a Unix socket (the embedded node's control socket) or a FIFO. `std::fs::read` on a socket fails with an error that the recorder turns into a panic, and `read` on a FIFO with no writer blocks forever, so a test hangs instead of failing. Nothing in this change creates either, since B's home holds only regular files and directories today. The risk is that a later test reuses this generic recorder (the file header presents it as the one recorder) against a home with a running node.
      **Measured:** not exercised, since no test here creates one. Severity low and latent; a stylistic hardening, not a defect today. Recording such an entry by kind alone (a fourth `Entry` variant) would remove it.

      **Outcome (`tester`): fixed.** `Kind` gains a `Special` variant, and `record_under` reads bytes only for `file_type.is_file()`; a socket, FIFO or device is recorded by existence and permissions. The new test `the_recorder_records_a_socket_by_its_kind_without_reading_it` (Linux only: it binds the socket through `/proc/self/fd/<n>`, because the scratch path can exceed the 107-byte socket address limit) was watched failing before the fix with `unreadable file sock: No such device or address (os error 6)`, and passes after. A FIFO was not tested directly: it hangs where a socket fails, and the same `is_file()` branch covers both.
