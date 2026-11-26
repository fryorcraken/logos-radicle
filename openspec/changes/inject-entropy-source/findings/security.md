# Security review: inject-entropy-source

Dimension: **security** only. Scope: `radicle/rust-ffi/src/profileinit.rs`,
`radicle/rust-ffi/tests/profile_init.rs`, read against the delta spec and
`design.md`.

## What was checked, and found clean

- **Can a caller choose the seed's source?** No. `init_profile_with_entropy` is
  a generic Rust `pub fn`, not `extern "C"` or `#[no_mangle]`, so the staticlib
  exports no symbol for it. `radicle_local_init_profile` still takes exactly
  `home, alias, passphrase` and calls `init_profile`, which fixes
  `open_system_entropy`. `radicle_ffi.h` is hand-written and unchanged. Nothing
  in `radicle/src/` (C++) references the seam. No environment variable, path
  argument or file descriptor reaches the opener; the only path is the constant
  `SYSTEM_ENTROPY`. The crate is `["staticlib", "rlib"]`, so a Rust consumer
  linking the rlib could call the seam, but nothing else in the repo links it.
  The design records that as a review-held risk, which is accurate.
- **Error paths cannot weaken the seed.** `draw_seed` returns `Err` for an
  open failure, a read error and a short read, and `?` returns before
  `Home::new` and `Profile::init`. The seed is never padded or partly used. A
  short read, a mid-read failure, an unopenable source and an empty source were
  all checked as tests; no key file is written.
- **Short-read handling.** `read_exact` retries `Interrupted` and maps a zero
  read to `UnexpectedEof`, which is turned into the entropy message. I mutated
  `read_exact(&mut bytes)` to `read_exact(&mut bytes[..31])`: five tests went
  red (`a_source_short_of_32_bytes_at_any_length_creates_no_key` at the
  `ends-after-31` row, `an_interruption_anywhere_in_the_draw_is_retried`,
  `an_interrupted_read_is_retried_not_failed`,
  `a_source_longer_than_a_seed_uses_its_first_32_bytes`,
  `a_seed_supplied_one_byte_per_read_is_used_whole`). The node-id oracle (the
  `radicle` crate's own `Profile::init`) is what catches it.
- **Source swap on the production path.** I changed `SYSTEM_ENTROPY` to
  `/dev/zero`: exactly one test went red,
  `two_homes_get_two_different_identities`, with two identical node ids.
- **Ordering.** The draw sits after every refusal and before `Home::new`, so a
  refused call never opens the source and a failed draw leaves no directory
  (`a_failed_draw_creates_no_home_directory`, `a_refused_home_never_opens_the_entropy_source`).
  A failed draw leaves the home `Empty`, so it is retryable with no manual step.
- **Error messages.** The opener error names only the constant
  `/dev/urandom` and the OS error text. No key material, seed byte or caller
  secret enters any message. The other messages that carry the home path were
  there before and the caller supplied that path.
- **Panic exposure.** The entry point still goes through `guarded()`. A source
  returning a count larger than the buffer would make std's `read_exact` panic,
  but only a test-supplied `Read` can do that, and `/dev/urandom` cannot.
- **Seed lifetime in memory.** `ec25519::Seed` is `Copy` and has no zeroize on
  drop, so the 32-byte `bytes` array and the `Seed` stay on the stack unwiped,
  and now survive across `Home::new` rather than being built as an argument to
  `Profile::init`. This is no worse than before in kind, and `design.md` lists
  zeroising as a non-goal. Not a finding.
- **`/dev/urandom` read directly rather than `getrandom(2)`.** Unchanged by this
  piece and recorded in the module docs. It does not check that the opened file
  is a character device, so a regular file at that path would be accepted. That
  needs control of `/dev`, which is root; not a finding. Not tested.
- **Dependencies.** None added; `Cargo.toml` and `Cargo.lock` are untouched.

## Findings

- [ ] **`tester`** — `radicle/rust-ffi/tests/profile_init.rs` (production wiring: `profileinit.rs:358`, `open_system_entropy`) — nothing pins that the production source supplies all 32 bytes as entropy; a seed with 64 bits of entropy passes every test
      **Scenario:** `init_profile` is changed to draw through
      `open_system_entropy().map(|f| Read::take(f, 8).chain(std::io::repeat(0)))`,
      so every production seed is 8 random bytes and 24 zero bytes. A creation
      still reports `created:true`, and two homes still get two different node
      ids, because the random 8 bytes differ. This is the same
      "mostly random is accepted" failure the change was written to catch, on
      the one path the scripted-source tests do not reach: they go through
      `init_profile_with_entropy` with a scripted source, so the wiring from
      `init_profile` to `/dev/urandom` is covered only by
      `two_homes_get_two_different_identities`, which asserts nothing about
      the seed's length. The design (Decision 1) says "production still reads
      `/dev/urandom` exactly as before" and task 1.7 cites that one test as the
      evidence.
      **Measured:** with that mutation applied, `cargo test --test profile_init`
      passed 32 of 32. For contrast, `/dev/zero` in place of `/dev/urandom`
      fails exactly one test, and `read_exact(&mut bytes[..31])` fails five.
      **Severity:** medium. The key is a permanent identity and the gap is on
      the production path itself. A pin is cheap: a unit test in
      `profileinit.rs` calling `draw_seed(open_system_entropy)` twice and
      asserting that the first 16 and the last 16 bytes of the two seeds each
      differ (false-failure probability 2^-128). `Seed` derefs to `[u8; 32]`.
      That test is red under the `take(8)` mutation, and under `/dev/zero`, a
      fixed seed and any half-zeroed seed.
