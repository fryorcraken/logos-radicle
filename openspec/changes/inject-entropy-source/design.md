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

## Goals / Non-Goals

**Goals:**

- A test can hand identity creation a reader that ends early, fails, chunks or
  reports an interruption, and observe what creation does with it.
- The production path reads `/dev/urandom` exactly as before, at the same point
  relative to every refusal.
- Every way the draw can fail produces one message prefix that names entropy.

**Non-Goals:**

- Zeroising the seed buffer after use. The bytes sit on the stack exactly as
  they did before; this change does not alter how long they live.
- Replacing `/dev/urandom` with `getrandom`. The vendoring reason recorded in
  `docs/rust-ffi.md` still holds.
- Exposing the source anywhere a caller outside the Rust crate can reach it.

## Decisions

### 1. Inject the source through a Rust-only function; the C ABI stays closed

`init_profile_with_entropy(home, alias, passphrase, open_entropy)` carries the
whole of identity creation, and `init_profile` — what the `extern "C"` entry
point calls — is that function with the `/dev/urandom` opener fixed. A test in
`tests/profile_init.rs` calls the first; nothing across the FFI boundary can,
because the function is not `extern "C"` and cbindgen does not emit it.

The issue's own framing is the reason this exists at all: "`/dev/urandom`
cannot be made to short-read portably, so the test needs an injectable
source." The archived task (`2026-09-12-m3-embedded-node-foundations`, 4.19)
put the evidence in one line: **replacing `read_exact` with a lenient `read`
that discards the count left every test in `tests/profile_init.rs` passing**,
because two half-zeroed seeds still differ in their random prefix, so the
isolation test is satisfied by a weak key.

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

### 2. Inject an opener, not an open reader

The seam takes `impl FnOnce() -> io::Result<impl Read>` rather than an
already-open reader, and calls it at the moment of the draw.

Passing an open reader would force `init_profile` to open `/dev/urandom`
before any validation runs. That is a behaviour change on the production path:
a call refused for an empty, relative, occupied or half-created home, or a bad
alias, would open the random source first, and where the open also failed, the
reply would name entropy instead of the refusal the caller earned. An opener
keeps the open exactly where it was relative to every refusal.

It also makes the open failure testable: an opener returning `Err` is the one
way to show that an unopenable source yields the entropy message, which was
the message the old code got wrong.

### 3. The seed is drawn before `Home::new`, after every refusal

The draw sits after the home-state and alias refusals and before `Home::new`
creates the directory tree. Previously it was evaluated as an argument to
`Profile::init`, after `Home::new`, so a failed draw left an empty Radicle home
— four subdirectories, no key — on disk.

The spec requires only that a failed draw leave the home Empty, which an empty
directory tree satisfies. Drawing first goes further for the same reason every
other refusal in `init_profile_inner` already fires before `Home::new`: a call
that fails should leave the filesystem as it found it. The cost is nil — the
draw needs nothing `Home::new` produces.

**What breaks without it:** moving the draw back after `Home::new` turns
`a_failed_draw_creates_no_home_directory` red, and nothing else. That test is
marked `NO SPEC:` because the spec does not say whether the directory may
exist; this is reported for the spec to decide.

### 4. `read_exact` stays, and the source's own retry semantics are std's

The spec's "A seed supplied in pieces is a full read" — several reads, an
interrupted read retried — is exactly what `std::io::Read::read_exact`'s
default implementation does: it loops until the buffer is full, retries
`ErrorKind::Interrupted`, and returns `UnexpectedEof` when a read returns 0
first. Hand-writing that loop would be a second copy of std to get wrong.

**What breaks without it:** replacing `read_exact` with a single `read` that
discards the count — the archived task's mutation — turns red the short-source
and failing-source tests (no error comes back) and the one-byte-per-read test
(the key is derived from 1 real byte and 31 zeros).

### 5. One message prefix for every failure of the draw

Every failure reads `the entropy for a new key could not be read: <cause>`,
where the cause is the opener's error for an unopenable source, the reader's
error for a failing one, and "the source ended before supplying 32 bytes" for a
short one. The short-read cause is ours rather than std's `UnexpectedEof` text
("failed to fill whole buffer"), which says nothing a user could act on.

The production opener wraps `File::open`'s error with the path, so an
unopenable `/dev/urandom` still says which source it was.

## Risks / Trade-offs

- [A future Rust caller in this crate passes a weak source to
  `init_profile_with_entropy`] → The function's doc comment says it exists for
  tests and that production goes through `init_profile`; the C ABI cannot reach
  it. Review is the guard here, as it is for any other Rust function in the
  crate.
- [A test source that ignores its input still passes an equality assertion] →
  The chunked and interrupted tests compare against a node id derived by the
  `radicle` crate's own `Profile::init` from the same bytes, not by the code
  under test, so a seam that ignored the source and drew from `/dev/urandom`
  fails them.
