# Design

## Context

`profileinit.rs` draws the new identity's 32-byte key seed by opening
`/dev/urandom` and calling `read_exact` on it, inside one private function
called with no arguments. The spec has required a short read to fail since the
embedded node's foundations landed, and nothing could show that it does:
`/dev/urandom` cannot be made to short-read on demand, portably or otherwise,
so the only source the code could ever draw from was one no test can make
misbehave. See `proposal.md` for why that matters.

The function's two failure messages also disagreed: a read failure said
"entropy", an open failure said only "the system random source". The spec's
"A failed entropy draw creates no key" requires every failure of the draw to
say that the entropy for a new key could not be read.

The C ABI (`radicle_local_init_profile(home, alias, passphrase)`), the C++
`createEmbeddedIdentity(alias, passphrase)` and `radicle_ffi.h` are unchanged.

**Two departures from the issue's "the production path still reads
`/dev/urandom` with no behaviour change", both deliberate.** They are
observable only when the production draw fails, which on a working Linux
system it does not:

- A failed draw now leaves no directory at all, where it used to leave an
  empty four-directory Radicle home (Decision 3).
- An unopenable `/dev/urandom` now reports "the entropy for a new key could
  not be read: …" instead of "could not open the system random source: …"
  (Decision 5). That is the spec's requirement, not a choice.

## Goals / Non-Goals

**Goals:**

- A test can hand identity creation a reader that ends early, fails, chunks or
  reports an interruption, and observe what creation does with it.
- The production path reads `/dev/urandom`, opened after every refusal, and a
  successful draw produces an identity exactly as before.
- Every way the draw can fail produces one message prefix that names entropy,
  followed by the text of whatever error the source reported.

**Non-Goals:**

- Zeroising the seed buffer after use. The bytes sit on the stack exactly as
  they did before; this change does not alter how long they live.
- Replacing `/dev/urandom` with `getrandom`. The vendoring reason recorded in
  `docs/rust-ffi.md` still holds.
- Exposing the source anywhere a caller outside the Rust crate can reach it.

## How the guards below were measured

Each "What breaks without it" names the tests that go red under one mutation,
run against this change's tree with
`cargo test --manifest-path radicle/rust-ffi/Cargo.toml --test profile_init --no-fail-fast`
and reverted with `git checkout` of the one file. A list of tests is the
record, not a count, so a test added later shows up as missing from the list
rather than silently making a number wrong. Re-run the mutation before trusting
a list after the suite has changed.

## Decisions

### 1. Inject the source through a Rust-only function; the C ABI stays closed

`init_profile_with_entropy(home, alias, passphrase, open_entropy)` carries the
whole of identity creation, and `init_profile` — what the `extern "C"` entry
point calls — is that function with the `/dev/urandom` opener fixed. A test in
`tests/profile_init.rs` calls the first; nothing across the FFI boundary can.

The issue's own framing is the reason this exists at all: "`/dev/urandom`
cannot be made to short-read portably, so the test needs an injectable
source." The archived task (`2026-09-12-m3-embedded-node-foundations`, 4.19)
put the evidence in one line: **replacing `read_exact` with a lenient `read`
that discards the count left every test in `tests/profile_init.rs` passing**,
because two half-zeroed seeds still differ in their random prefix, so the
isolation test is satisfied by a weak key.

**Why nothing across the boundary can reach the seam.** `radicle_ffi.h` is
written and checked in by hand; nothing generates it (there is no `build.rs`,
no cbindgen configuration and no build step that emits a header). So a Rust
function reaches C++ only if someone writes an `extern "C"` wrapper for it
*and* a declaration in that header. `init_profile_with_entropy` has neither:
it is a generic Rust function, not even a linkable symbol in the staticlib,
and `radicle_local_init_profile` keeps its three `const char*` parameters.
"No caller can choose the entropy source" is held by those two signatures, and
both are pinned by a test rather than by review alone.

Alternatives considered:

- **Mark the requirement structural-review-only in the spec.** The option the
  archived task offered alongside injection, and called a retreat: it leaves
  the most serious gap that review found exactly as unpinned as it was, with a
  sentence saying so. Rejected for that reason.
- **A `#[cfg(test)]` private seam with in-crate unit tests.** Stronger on
  paper — no Rust caller could reach the seam either — but it moves
  identity-creation tests out of `tests/profile_init.rs`, away from the scratch
  home fixture they share, and the boundary the spec draws is the C ABI and the
  module method, not the crate's Rust surface. Nothing outside this crate links
  its Rust API.
- **A cargo feature gating the seam.** Costs a `Cargo.toml` change and a
  `required-features` dance for the integration tests, for the same protection
  the C ABI already gives.
- **Choosing the source from the environment** (a path in a variable, say).
  Rejected outright: that is a production input selecting where a signing key's
  entropy comes from, which is what "No caller can choose the entropy source"
  forbids.

**What breaks without the closed ABI:** a fourth parameter on the declaration in
`radicle_ffi.h` turns
`the_declared_init_prototype_has_three_const_char_pointer_parameters` red; a
fourth parameter on the `extern "C"` function in `lib.rs` stops the
`profile_init` test target compiling, at
`the_init_entry_point_takes_home_alias_and_passphrase_and_nothing_else` and at
`the_init_entry_point_is_guarded`'s three-argument call. `createEmbeddedIdentity`'s
two-argument signature is on the C++ side, out of the Rust tests' sight, and is
held by its callers passing two arguments.

**What breaks without the seam being live:** making `init_profile_with_entropy`
ignore its opener and draw from `/dev/urandom` regardless turns red every test
that hands it a scripted source except `a_refused_home_never_opens_the_entropy_source`
(which never reaches the draw):
`a_source_that_ends_after_16_bytes_creates_no_key`,
`a_source_that_fails_mid_read_creates_no_key`,
`a_read_error_of_the_end_of_file_kind_keeps_its_own_text`,
`a_source_that_cannot_be_opened_creates_no_key`,
`a_failed_draw_leaves_the_home_completable`,
`a_failed_draw_creates_no_home_directory`,
`a_source_short_of_32_bytes_at_any_length_creates_no_key`,
`a_seed_supplied_one_byte_per_read_is_used_whole`,
`an_interrupted_read_is_retried_not_failed`,
`an_interruption_anywhere_in_the_draw_is_retried` and
`a_source_longer_than_a_seed_uses_its_first_32_bytes`. The tests that compare
node ids compare against an id the `radicle` crate's own `Profile::init`
derives from the same bytes, not against a second call through the seam, which
would agree with itself while ignoring its input.

### 2. Inject an opener, not an open reader

The seam takes `impl FnOnce() -> io::Result<R>` with `R: Read` rather than an
already-open reader, and calls it at the moment of the draw.

Passing an open reader would force `init_profile` to open `/dev/urandom`
before any validation runs. That is a behaviour change on the production path:
a call refused for an empty, relative, occupied or half-created home, or a bad
alias, would open the random source first, and where the open also failed, the
reply would name entropy instead of the refusal the caller earned. An opener
keeps the open where it was relative to every refusal.

It also makes the open failure testable: an opener returning `Err` is the one
way to show that an unopenable source yields the entropy message, which was
the message the old code got wrong.

The opener is generic rather than a `Box<dyn FnOnce() -> io::Result<Box<dyn
Read>>>`. Static dispatch costs nothing here and needs no allocation; the price
is a type parameter on `init_profile_with_entropy` and `draw_seed`, the only
two functions that handle the source. A boxed opener would buy nothing a
caller needs: there are two callers, `init_profile` and the tests, and both
know their reader's type.

**What breaks without it:** opening the source in `init_profile_with_entropy`
before `admit` runs turns `a_refused_home_never_opens_the_entropy_source` red,
and nothing else. The test records whether its opener was called; its message
assertion alone stays green under that mutation, because the eagerly opened
result is not consumed until the draw and the refusal still wins.

### 3. The seed is drawn before `Home::new`, after every refusal, and the shape holds the order

The draw sits after the home-state and alias refusals and before `Home::new`
creates the directory tree. Previously it was evaluated as an argument to
`Profile::init`, after `Home::new`, so a failed draw left an empty Radicle home
— four subdirectories, no key — on disk.

The spec requires only that a failed draw leave the home Empty, which an empty
directory tree satisfies. Drawing first goes further for the same reason every
refusal already fires before `Home::new`: a call that fails should leave the
filesystem as it found it. The cost is nil — the draw needs nothing `Home::new`
produces.

The order is held by the code's shape, not by one line's position in a long
function. `admit` holds every refusal and returns an `Admitted` carrying the
parsed alias and passphrase; `create(Admitted, Seed)` is the only place that
writes, and it takes the seed as an argument, so it cannot run before the
draw. `init_profile_with_entropy` is `admit`, then `draw_seed`, then `create`.
The architecture review proposed this split; it was made as its own commit,
with no behaviour change.

**What breaks without it:** calling `Home::new` before the draw turns
`a_failed_draw_creates_no_home_directory` red, and nothing else. That test is
marked `NO SPEC:` because the spec does not say whether the directory may
exist; this is reported for the spec to decide.

### 4. `read_exact` stays, and the source's own retry semantics are std's

The spec's "A seed supplied in pieces is a full read" — several reads, an
interrupted read retried, the first 32 bytes and no read after them — is
exactly what `std::io::Read::read_exact`'s default implementation does: it
loops until the buffer is full, retries `ErrorKind::Interrupted`, returns
`UnexpectedEof` when a read returns 0 first, and stops calling `read` once the
buffer is full. Hand-writing that loop would be a second copy of std to get
wrong.

The buffer is exactly `SEED_BYTES` long, so a source that would supply more is
never asked for it: the first 32 bytes are the seed, and a longer source is not
drained. That is what the spec's "A source longer than a seed uses its first 32
bytes" and "Nothing past the 32nd byte is read" require.

**What breaks without it:** replacing `read_exact` with a single `read` that
discards the count — the archived task's mutation — turns red every test whose
source does not hand over all 32 bytes in its first read:
`a_source_that_ends_after_16_bytes_creates_no_key`,
`a_source_that_fails_mid_read_creates_no_key`,
`a_read_error_of_the_end_of_file_kind_keeps_its_own_text`,
`a_failed_draw_leaves_the_home_completable`,
`a_failed_draw_creates_no_home_directory` and
`a_source_short_of_32_bytes_at_any_length_creates_no_key` (each gets
`created:true` from a partly zeroed seed), and
`a_seed_supplied_one_byte_per_read_is_used_whole`,
`an_interrupted_read_is_retried_not_failed` and
`an_interruption_anywhere_in_the_draw_is_retried` (each gets the wrong key, or
an interruption reported as a failure).

### 5. One message prefix for every failure of the draw, and the source's own words kept

Every failure reads `the entropy for a new key could not be read: <cause>`,
where the cause is the opener's error for an unopenable source, the reader's
error for a failing one, and "the source ended before supplying 32 bytes" for a
short one. The short-read cause is ours rather than std's
(`read_exact` says "failed to fill whole buffer", which says nothing a user
could act on).

The production opener wraps `File::open`'s error with the path, so an
unopenable `/dev/urandom` still says which source it was.

**Only `read_exact`'s own running-out is reworded.** The spec requires the
error to include the text of any error the source reported, and a source can
itself raise an error of kind `UnexpectedEof` — the same kind `read_exact` uses.
The kind alone cannot tell the two apart, so the draw reads through
`EndObserved`, a pass-through reader that records whether the source returned
end of input (a read of 0 bytes into a non-empty buffer). Only then is the
cause replaced; otherwise the source's error text is carried unchanged.

Alternatives considered:

- **Reword on the kind.** What the first version did. It drops the words of a
  source that raises `UnexpectedEof` itself, which the spec forbids.
- **Keep std's text and append ours** ("…ended before supplying 32 bytes:
  failed to fill whole buffer"). Meets the spec, but puts the text this
  decision exists to hide back into every short-read message.
- **Tell the two apart by the error's payload** (`get_ref()` is `None` for
  std's constant). A source can raise a payload-free `UnexpectedEof` too, so
  this misclassifies exactly the case it is for.
- **Hand-write the read loop** to see the zero-byte read directly. Rejected by
  Decision 4.

**What breaks without it:** restoring the old open-failure message ("could not
open the system random source: …") turns
`a_source_that_cannot_be_opened_creates_no_key` red, and nothing else. Rewording
on the kind instead of on `EndObserved` turns
`a_read_error_of_the_end_of_file_kind_keeps_its_own_text` red (watched failing
against that code before the fix). **Making `EndObserved` never record an end
turns nothing red**: every short-source test asserts the prefix and never the
cause, so the "ended before supplying 32 bytes" text is not pinned. That is
open with the `tester` under the correctness review.

## Risks / Trade-offs

- [A future Rust caller in this crate passes a weak source to
  `init_profile_with_entropy`] → The function's doc comment says it exists for
  tests and that production goes through `init_profile`; the C ABI cannot reach
  it. Review is the guard here, as it is for any other Rust function in the
  crate.
- [A test source that ignores its input still passes an equality assertion] →
  The chunked, interrupted and long-source tests compare against a node id
  derived by the `radicle` crate's own `Profile::init` from the same bytes, not
  by the code under test, so a seam that ignored the source and drew from
  `/dev/urandom` fails them.
- [Someone adds an entry point and assumes a generator will keep the header in
  step] → `radicle_ffi.h` says at its top that it is checked in by hand, and
  `docs/rust-ffi.md` now says the same; the init prototype's shape is pinned by
  a test that reads the header.
