# Correctness review — inject-entropy-source

Reviewed `radicle/rust-ffi/src/profileinit.rs` (`draw_seed`, `open_system_entropy`,
`init_profile`, `init_profile_with_entropy`, `init_profile_inner`) and
`radicle/rust-ffi/tests/profile_init.rs`, against the delta spec and `design.md`.
Dimension: **correctness** only.

Measured from this tree: `cargo test --test profile_init` 32 passed,
`cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean,
`openspec validate inject-entropy-source --strict` valid. `cargo mutants` is not
installed here, so every mutation below was made by hand, run, and reverted with
`git checkout` of the one source file.

Mutations that **were caught** (the guards the design names do their jobs):

- `read_exact` replaced by a single `read` discarding the count: 8 tests red.
- `read_exact(&mut bytes[..31])` (a 31-byte seed, last byte left zero): 5 red,
  including the `ends-after-31` row and the crate-derived-id oracle tests.
- The seam ignoring its opener and opening `/dev/urandom`: 10 red.
- The draw moved after `Home::new`: exactly `a_failed_draw_creates_no_home_directory`
  red, as Decision 3 says; the home still classifies Empty, so the spec's
  completable scenario holds either way.

Clean areas: the draw's ordering against every refusal (the opener is never
called for an empty, relative, occupied, half-created or bad-alias call, and a
refusal is never reported as an entropy failure); no reachable panic on the
production path (the buffer is a fixed 32-byte array, `read_exact` owns all
indexing, no `unwrap`/`expect` outside tests); the C ABI is unchanged
(`radicle_local_init_profile` still takes three arguments, `radicle_ffi.h` is
untouched, and the extern entry point still goes through `guarded`);
`Interrupted` retry, chunked reads and over-long sources behave as the spec
requires; the oracle node ids come from the crate's own `Profile::init`, not
from the code under test.

Noted, not boxed (nobody needs to act): a source that itself returns an error of
kind `UnexpectedEof` is reworded to "the source ended before supplying 32 bytes"
and loses its own text, because `read_exact`'s short-read error and a reader's own
`UnexpectedEof` are indistinguishable by kind. `/dev/urandom` never does this, and
the spec only requires that the reply say the entropy could not be read, which it
does. The production opener's path-naming error (`open_system_entropy`) cannot be
exercised by any test, since `/dev/urandom` cannot be made unopenable; it is
three lines of `map_err`.

- [x] **`tester`** — `profileinit.rs:116-122` — the "source ended before supplying 32 bytes" cause text is pinned by no test
      **Scenario:** replacing the `UnexpectedEof` branch with the plain `e.to_string()` path (mutation `if false && e.kind() == ...`) returns std's "failed to fill whole buffer" for every short source, which Decision 5 says is the reason the branch exists. All 32 tests in `profile_init.rs` still pass: every short-source test asserts only the `ENTROPY_UNREADABLE` prefix, never the cause.
      **Measured:** 32 of 32 pass under this mutation. Severity low; the prefix the spec requires is pinned, only the user-facing cause is not.
      **Outcome (`tester`): fixed**, by `a_source_that_ends_early_says_so_instead_of_quoting_std`, which creates from a source ending after 0, 16 and 31 bytes and asserts the error contains "the source ended before supplying 32 bytes" and does not contain std's "failed to fill whole buffer". The code the finding names has since been replaced by `EndObserved`, so the mutation is restated against it: `EndObserved::read` never records an end (`if false && n == 0 && ...`). **Predicted** exactly one test red and every other test green; **observed** the same, with the suite run in full: only the new test failed (`0 bytes then end of input: the error must say the source ended early, got: {"error":"the entropy for a new key could not be read: failed to fill whole buffer"}`). Restored with `git checkout`.
      **Judgement call, recorded: the test carries `NO SPEC:`.** The spec requires a short source's error to state that the entropy could not be read, and says nothing of the cause's wording, which is `design.md` Decision 5's choice. The assertion is therefore on unspecified behaviour, and a reader should be able to see that; it is also a separate test, so a later spec change to the wording lands on this test and not on the scenario tests that pin the prefix.
- [x] **`dev-writer`** — `design.md:77-85,142-150` and `tasks.md:29-31` — the red-test counts quoted for two mutations are stale
      **Scenario:** Decision 1 says the seam ignoring its argument "turns eight tests red", Decision 4 says the lenient `read` "turns six tests red", and task 1.3 repeats "eight". Run against this tree, the ignore-the-argument mutation turns **10** red (adding `a_source_short_of_32_bytes_at_any_length_creates_no_key` and `an_interruption_anywhere_in_the_draw_is_retried`, which the tester added after the counts were written) and the lenient `read` turns **8** red (the same two again). The named tests listed in each Decision are a subset of the real ones.
      **Measured:** 10 and 8, from `cargo test --test profile_init` under each mutation. Severity low, documentation accuracy only; fix by re-measuring, not by applying a delta.
      **Fixed** in the commit that flips this box, by re-measuring and then replacing the counts with named lists. I measured only after this pass, which added `a_read_error_of_the_end_of_file_kind_keeps_its_own_text`: it goes red under both mutations, and the rest of each red list is the set you named, so a count would already have been stale again. Decision 1, Decision 4 and task 1.3 now state the property and name the tests, with no number, and `design.md` gives the command each list came from (`cargo test --manifest-path radicle/rust-ffi/Cargo.toml --test profile_init --no-fail-fast`, mutation applied, file restored with `git checkout`). Your noted-not-boxed point about a source's own `UnexpectedEof` became a spec violation under the new spec text and is fixed in `7c0162d`; see `design.md` Decision 5.
