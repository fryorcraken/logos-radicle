# spec-test review: inject-entropy-source

Spec: `specs/embedded-identity/spec.md`. Tests: `radicle/rust-ffi/tests/profile_init.rs`.
The implementation was not read, except the two spots mutated below.

**Coverage, scenario by scenario.** Every scenario has a test at a layer that
can see it: the two-homes scenario by `two_homes_get_two_different_identities`;
the short, failing and completable scenarios by `a_source_that_ends_after_16_bytes_creates_no_key`,
`a_source_that_fails_mid_read_creates_no_key`, `a_failed_draw_leaves_the_home_completable`
(plus the 0/1/31-byte table); the pieces and interruption scenarios by
`a_seed_supplied_one_byte_per_read_is_used_whole` and
`an_interrupted_read_is_retried_not_failed` (plus the interruption table). The
last two compare against a node id the `radicle` crate derives itself through
`Profile::init`, which is an independent oracle and not something the code under
test produced. The signature scenario is pinned for the Rust entry point and the
checked-in header, not for the C++ method (finding 4). Reading found none of the
three invariant-breaking shapes in the new tests. The mock source is
input-dependent (scripted per case), so "never drew" and "drew" differ.

**Mutations run, both restored; none left in the tree** (`git status --short`
is empty).

1. `draw_seed` made to read the source to its end after the 32 bytes
   (`read_exact(..).and_then(|_| io::copy(&mut source, &mut io::sink()))`).
   Command: `cargo test --manifest-path radicle/rust-ffi/Cargo.toml --test profile_init`.
   **Survived every scripted-source test**: the output shows
   `a_source_longer_than_a_seed_uses_its_first_32_bytes`,
   `a_seed_supplied_one_byte_per_read_is_used_whole`,
   `an_interrupted_read_is_retried_not_failed` and
   `an_interruption_anywhere_in_the_draw_is_retried` all `ok`. The only signal was
   that the tests going through the real `/dev/urandom` hung ("has been running
   for over 60 seconds"), which I stopped; that is a hang, not a failure.
2. `radicle_ffi.h` given a fourth parameter on `radicle_local_init_profile`.
   Command: `cargo test ... --test profile_init the_declared_init_prototype`.
   **Killed**: "the declared prototype must take exactly home, alias and
   passphrase, got [... 4 entries]".

**Clean:** the refusal-ordering table, the any-length short/fail table, the
interruption table, the compiled-signature pin, and the home-stays-Empty
scenario are each judged by reading to be able to fail for the reason they
name.

- [x] **`spec-writer`** — "A seed supplied in pieces is a full read": "the
      signing key MUST be derived from exactly the 32 bytes the source supplied".
      **Answer to the tester's question: yes, a spec line is needed.** The
      production source, `/dev/urandom`, never ends, so it always supplies more
      than 32 bytes, and the requirement as worded is false or ambiguous for the
      one source that matters: "exactly the 32 bytes it supplied" has no meaning
      for an unbounded stream. Test `a_source_longer_than_a_seed_uses_its_first_32_bytes`
      (marked `NO SPEC`) pins that the first 32 decide the key, but nothing in the
      spec says so. A second property is also unstated: the draw MUST stop after 32
      bytes and MUST NOT need the source to end or stay healthy afterwards, which
      is what keeps a draw from `/dev/urandom` finite. Add to the requirement that
      the seed is the first 32 bytes supplied and that nothing past them is read,
      with a scenario for a source supplying more than 32 bytes.
      **Scenario:** an implementation that reads a source to its end and then takes
      the first 32 satisfies the spec as written and hangs in production.
      **Measured:** mutation 1 above. Severity: medium.
      **Outcome (`spec-writer`): fixed in the spec.** The requirement now says
      the seed is the first 32 bytes the source supplies, in order, and that
      once they are supplied the backend MUST NOT read from the source again,
      whether it would supply more, end or fail. Two scenarios added: "A source
      longer than a seed uses its first 32 bytes" (48 bytes then end of input;
      node id equals that of the first 32 in one read) and "Nothing past the
      32nd byte is read" (32 bytes then a read error on every further read;
      `created:true`, the same node id, and no read received after the 32nd
      byte). The test that fails without the behaviour is the `tester`'s entry
      below; the `NO SPEC` on `a_source_longer_than_a_seed_uses_its_first_32_bytes`
      is now covered by the first scenario.
- [x] **`tester`** — no test pins that the draw stops at 32 bytes. Once the
      spec-writer's line lands, add a scripted source that supplies 32 bytes and
      then reports `Step::Fail` (and one that supplies 32 then yields more than
      32): creation must still report `created:true` with the node id derived from
      those 32 bytes. `a_source_longer_than_a_seed_uses_its_first_32_bytes` cannot
      see an over-read, because the scripted reader has nothing further to fail on
      and ends cleanly.
      **Scenario:** a draw that reads on past byte 32 passes every scripted test
      and is caught only as a hang in the `/dev/urandom` tests.
      **Measured:** mutation 1 above; scripted tests all green. Severity: medium.
      **Outcome (`tester`): fixed** by `nothing_past_the_32nd_byte_is_read`
      (`radicle/rust-ffi/tests/profile_init.rs`), five rows: 32 bytes then a
      read error; 32 bytes then an error of the end-of-file kind; 32 bytes one
      per read then an error; 32 bytes then an interruption then an error; and
      one read of 48 bytes with no failure at all. Each row asserts
      `created:true`, the node id the `radicle` crate itself derives from the
      first 32 bytes, and that the fake saw no read after handing over its 32nd
      byte. The last is a counter in the fake (`Scripted::late_reads`, with the
      32 written out in the test, not read from the code), because an over-read
      from a source that does not fail afterwards produces the same bytes and the
      same node id, and nothing else can see it. `a_source_longer_than_a_seed_uses_its_first_32_bytes`
      now uses the spec's 48 bytes (was 64).
      **Predicted and observed, run filtered to this one test (the real
      `/dev/urandom` tests hang under an over-read):** (a) `draw_seed` reads the
      source to its end after the 32 bytes, errors propagating: predicted red at
      row 1 with the draw failing; observed `one-read-then-an-error: a source
      that failed only after the 32nd byte must not fail the draw`. (b) the same
      with the error swallowed: predicted red on the counter alone; observed
      `one-read-then-an-error: the source was read again ... left: 1 right: 0`,
      so the counter, not the error path, is what sees it. (c) table order
      reversed, same swallowed-error mutation (a temporary edit to the test,
      reverted): the no-failure row reports `left: 2 right: 0`, so that row is
      the only one that would see an over-read from a source with nothing wrong
      with it. Both restored with `git checkout`.
- [x] **`spec-writer`** — an unmarked unspecified behaviour. `a_source_that_cannot_be_opened_creates_no_key`
      pins that a source which cannot be opened fails with the entropy message and
      writes no key, and `a_source_that_fails_mid_read_creates_no_key` /
      the unopenable test pin that the source's own error text is carried in the
      message ("failed on purpose", "no such source"). The spec says a failure
      is "an error other than an interruption" from a source that "reports" it,
      which is about reading, not opening, and says nothing about carrying the
      cause. Opening `/dev/urandom` is the realistic production failure, so the
      requirement should name it and say whether the cause is carried. Neither test
      carries a `NO SPEC:` comment; the tester should add one if the spec-writer
      declines. **Scenario:** an implementation that reports a failed open as a
      generic error without the entropy wording, or drops the cause, meets the
      spec and fails these tests. Severity: low.
      **Outcome (`spec-writer`): fixed in the spec; the tested behaviour was the
      right choice.** "A failed entropy draw creates no key" now names a source
      that cannot be opened among the failures, and requires the error to
      include the text of any error that opening or reading the source
      reported. Carrying the cause is safe: `findings/security.md` records that
      the production opener's error names only the `/dev/urandom` constant and
      no secret. Added scenario "A source that cannot be opened creates no key"
      (error, entropy wording, the opening error's text, no `created`/`nodeId`,
      no file under `keys`), and the mid-read scenario gains "MUST include the
      text of the read error". Both tests the entry names now assert spec'd
      behaviour, so no `NO SPEC:` comment is needed. The production opener's
      own path-naming wording is not specified: a test cannot make
      `/dev/urandom` unopenable, so it would be an untestable scenario.
- [x] **`tester`** — "No caller can choose the entropy source" is pinned for the
      Rust entry point (type-checked function pointer) and the checked-in header
      (mutation 2 killed), but not for `createEmbeddedIdentity` in the C++ module,
      which the scenario names. The file says it is held by "every C++ caller
      passing two arguments", which does not hold for a third parameter with a
      default value: all 14 existing call sites in `radicle/tests/test_radicle_impl.cpp`
      would still compile. A test in that file assigning
      `&RadicleImpl::createEmbeddedIdentity` to a pointer of the exact
      two-`std::string` signature would pin it; that layer is the slow Nix core
      build, so it is the tester's call whether it earns the cost, or whether the
      spec-writer should drop the module half of the scenario. Severity: low.
      **Outcome (`tester`): fixed; the test earns its cost.** The layer is slow,
      but this test adds nothing to the run: it is a `static_assert` on the
      member's own type, checked when the file compiles, plus one call through
      that type. The scenario names `createEmbeddedIdentity`, so dropping the
      module half would leave a named behaviour unpinned, and a defaulted third
      parameter is the most plausible way for the seam to be widened. The test
      is `createEmbeddedIdentity_takes_an_alias_and_a_passphrase_and_nothing_else`
      in `radicle/tests/test_radicle_impl.cpp`; it compares
      `decltype(&RadicleImpl::createEmbeddedIdentity)` against
      `std::string (RadicleImpl::*)(const std::string&, const std::string&)`.
      The unmutated tree, `nix build ./radicle#checks.x86_64-linux.unit-tests`:
      the test PASSes; the run ends 180 passed, 2 failed, the two failures being
      `starting_a_node_with_no_embedded_identity_says_so_in_the_modes_own_words`
      (its text: "the node control socket path is too long", the sandbox cap the
      overlay names) and `embedded_mode_reads_and_writes_its_own_home_and_leaves_the_local_one_alone`
      (assertion at `test_radicle_impl.cpp:2079`; I did not read its cause, and
      both lie outside this change). **Predicted and observed:** (a) a third
      parameter with a default value on the header and definition: predicted a
      build failure at the new test and none at any existing call site; observed
      `static assertion failed: createEmbeddedIdentity must take an alias and a
      passphrase and nothing else` and a pointer-conversion error in the same
      test, with no error anywhere else, so the finding's point that every
      existing call site would still compile is confirmed by the compiler.
      (b) a third overload instead: observed `'decltype' cannot resolve address
      of overloaded function`. Both restored with `git checkout`. The Rust-file
      comment that said C++ callers passing two arguments hold the signature is
      corrected. Not edited, because `design.md` is the `dev-writer`'s:
      `design.md:114-116` still says the signature is "held by its callers
      passing two arguments", which is now untrue.
