# Design review: inject-entropy-source

Reviewed: `design.md` Decisions 1-5 against `radicle/rust-ffi/src/profileinit.rs`
(`draw_seed`, `open_system_entropy`, `init_profile`,
`init_profile_with_entropy`, `init_profile_inner`) and `src/lib.rs`'s
`radicle_local_init_profile`; issue #58 read fresh; `docs/rust-ffi.md`;
`docs/M3-embedded-node-plan.md` and `docs/M3-phase0-findings.md` (neither
mentions the entropy draw, so nothing there to contradict).

Decisions 1-5 are each implemented as written: the opener is `impl FnOnce`,
called inside `draw_seed` after every refusal and before `Home::new`;
`read_exact` is kept; every draw failure goes through one `unreadable` prefix;
`init_profile` fixes `open_system_entropy` and is the only caller from
`lib.rs`. The rejected-alternative the issue names (structural-review-only) is
recorded with its reason. `docs/rust-ffi.md` carries the trap a later change
would hit (test the draw through `init_profile_with_entropy`, oracle node id
from the crate). No code contradicts a recorded decision. The findings below
are about what the prose claims and what it leaves out.

- [x] **`dev-writer`** — `design.md:87-91`, `tasks.md` 1.8, `docs/rust-ffi.md:37` claim a mechanism that does not exist
      "`init_profile_with_entropy` is not `extern "C"`, so cbindgen emits nothing
      for it and `radicle_ffi.h` is unchanged." Nothing runs cbindgen here: no
      `build.rs`, no `cbindgen.toml`, no cbindgen in the tree except the sentence
      in `docs/rust-ffi.md:37` ("cbindgen-generated but **checked in**"), and
      `radicle_ffi.h:8` says "Checked in by hand". So "by construction" is
      weaker than stated: the header is exactly what a person types, and a
      hand-edited prototype is the only way the seam could reach C++. What
      actually guards it is the tester's
      `the_declared_init_prototype_has_three_const_char_pointer_parameters` plus
      the compiled-signature test, neither of which `design.md` mentions.
      **Verified:** `git ls-files` lists no `build.rs` or cbindgen config;
      `grep -i cbindgen` over `radicle/` and `docs/` hits only `rust-ffi.md:37`.
      Rewrite the paragraph to name the real mechanism (the header is
      hand-maintained; the two tests are what hold the signatures) and reconcile
      `rust-ffi.md:37` with the header's own comment, since that sentence will
      mislead the next person who adds an entry point.
      **Fixed** in the commit that flips this box. Re-verified first: `git ls-files "*build.rs" "*cbindgen*"` returns nothing, and no file under `radicle/`, `radicle-ui/` or `.github/` mentions cbindgen. Decision 1 now has a paragraph "Why nothing across the boundary can reach the seam" naming the real mechanism (no `extern "C"` wrapper, and no declaration in a header written by hand) and the two tests that hold the signatures; task 1.8 says the same; `docs/rust-ffi.md`'s sentence now says the header is written and checked in by hand, that a Rust function reaches C++ only with both a wrapper and a hand-written declaration, and that `tests/profile_init.rs` pins the init prototype for that reason. The module doc in `profileinit.rs` was given the same mechanism.

- [x] **`dev-writer`** — `design.md:28-29` Goal 2 contradicts Decision 3, and the issue's "no behaviour change" is departed from without saying so
      Goal 2: "The production path reads `/dev/urandom` exactly as before, at the
      same point relative to every refusal." Decision 3 moves the draw from after
      `Home::new` to before it, so a failed production draw no longer leaves an
      empty four-directory home; Decision 5 also changes the open-failure message
      text. Both are defensible and Decision 3 argues its own, but issue #58
      says "The production path still reads `/dev/urandom` with no behaviour
      change", and neither the Goals nor Context says these two observable
      differences are a deliberate departure from it. State them as such
      (observable only on a failed draw, which production essentially never
      sees), and fix Goal 2 to say "after every refusal" rather than "as before".
      **Fixed** in the commit that flips this box. Context now ends with "Two departures from the issue's 'the production path still reads `/dev/urandom` with no behaviour change', both deliberate", naming the no-directory-on-failure change (Decision 3) and the open-failure message (Decision 5, which the spec requires), and saying both are observable only when the production draw fails. Goal 2 now reads "The production path reads `/dev/urandom`, opened after every refusal, and a successful draw produces an identity exactly as before."

- [x] **`dev-writer`** — `design.md:77-85` count is stale and not self-invalidating
      "turns eight tests red" lists the eight scripted-source tests present when
      the dev-writer finished. The tester then added
      `a_source_short_of_32_bytes_at_any_length_creates_no_key` and
      `an_interruption_anywhere_in_the_draw_is_retried`, which also hand a
      scripted source and so also go red when the seam ignores it. Name the
      property ("every test that scripts a source except the refused-home one")
      rather than the number, or re-measure it. I read this from the test list
      and did not run the mutation.
      **Fixed** in the commit that flips this box, both ways. Every "What breaks without it" now states the property and lists the tests by name, with no count; a new section "How the guards below were measured" gives the command (`cargo test --manifest-path radicle/rust-ffi/Cargo.toml --test profile_init --no-fail-fast`) and says to re-run a mutation before trusting its list. Every list was re-measured by running its mutation against this tree, which now also holds `a_read_error_of_the_end_of_file_kind_keeps_its_own_text` from this pass: the ignore-the-opener mutation's list is every scripted-source test except `a_refused_home_never_opens_the_entropy_source`, which is the property you named.

- [x] **`dev-writer`** — a choice with a real alternative is decided in code and not recorded (gap)
      `draw_seed` reads exactly 32 bytes and stops, so a longer source is not
      drained and its first 32 bytes are the seed (pinned by
      `a_source_longer_than_a_seed_uses_its_first_32_bytes`, marked `NO SPEC`).
      Decision 4 covers short and chunked reads but not this edge, and the test
      says design.md owns it. Also unrecorded: an `UnexpectedEof` raised by the
      source itself (not by `read_exact` running out) is rewritten to "the source
      ended before supplying 32 bytes", losing the source's own text (Decision 5
      mentions the rewrite only for std's own message); and the generic
      `impl FnOnce() -> io::Result<R: Read>` over a boxed `dyn Read` opener
      (static dispatch, no allocation, but the function is generic over the
      reader). One or two sentences each in Decisions 4 and 5.
      **Fixed**, the three parts separately. (1) First 32 bytes: the spec has since taken it ("A source longer than a seed uses its first 32 bytes", "Nothing past the 32nd byte is read"), so it is behaviour, not a design choice; Decision 4 now says the 32-byte buffer is why a longer source is not drained and why no read follows the 32nd byte, and the test's `NO SPEC:` marker was removed in `7c0162d`. (2) The source's own `UnexpectedEof`: the new spec text ("MUST include the text of any error that opening or reading the source reported") made the rewrite a spec violation, not a gap, so the code changed in `7c0162d`: `draw_seed` reads through `EndObserved`, which records whether the source returned end of input, and only then rewords; `a_read_error_of_the_end_of_file_kind_keeps_its_own_text` was watched failing before it. Decision 5 records the choice and the three alternatives rejected (reword on the kind, append std's text, tell by `get_ref()`). (3) Generic over boxed opener: one paragraph in Decision 2. Recorded in the commit that flips this box.

- [x] **`dev-writer`** — suggestion: Decision 1's "what breaks without it" evidences a different guard than the one the decision is
      The decision is "the C ABI stays closed", but the mutation evidence given
      is for the seam being live (ignores its argument), which is Decision 2/4
      territory. For the closed-ABI guard, the mutation is "add a fourth
      parameter to the declaration in `radicle_ffi.h` / to the `extern "C"`
      function", which turns the two tester tests red. Once the first finding is
      fixed, add that line so the decision's own guard has its evidence.
      **Fixed** in the commit that flips this box. Decision 1 now has two "What breaks" paragraphs: one for the closed ABI and one for the seam being live. Both closed-ABI mutations were run: a fourth `const char* entropy` on the declaration in `radicle_ffi.h` turns `the_declared_init_prototype_has_three_const_char_pointer_parameters` red (4 parameters against 3); a fourth parameter on the `extern "C"` function in `lib.rs` stops the `profile_init` target compiling, at `the_init_entry_point_takes_home_alias_and_passphrase_and_nothing_else` (E0308) and at `the_init_entry_point_is_guarded`'s three-argument call (E0061).
