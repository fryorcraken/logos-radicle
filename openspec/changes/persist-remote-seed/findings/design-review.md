# Design review: persist-remote-seed

Reviewed: every Decisions entry in `design.md` against `radicle_impl.cpp`
(`setRemoteSeed`), `settings_store.cpp` (`validate`), the contract comment in
`radicle_impl.h` and the tests in `test_radicle_impl.cpp`; `design.md` and
`proposal.md` against `gh issue view 55` read fresh.

The code takes every recorded decision. Order is shape, probe, store, adopt;
the candidate client is copied and moved in only after the store write; the
stored URL is `candidate.seedUrl()`; `validate` is the single shape rule, called
from both doors. No decision is contradicted or half-applied. The issue's
"Done when" items are all met, and the open question is decided with the
alternatives and their rejections recorded. Nothing the change acted on is
named in the issue and missing from `design.md`. The findings below are gaps
and thin entries only.

- [ ] **`dev-writer`** — `radicle_impl.cpp:451` — a choice in the code is not in Decisions: an empty URL is refused, where `SettingsStore::validate("remoteSeed", "")` accepts it and `setSetting("remoteSeed", "")` clears the stored seed.
      A reader could have made `setRemoteSeed("")` mean "reset to default" and delegate to `validate`. The refusal predates the change and the spec states it, but `design.md` records the two doors' differences (shape, network) and not this third one, and it is exactly the case where reusing `validate` alone would silently differ. One sentence under "`SettingsStore::validate` is the one shape rule" would do. Gap, not a contradiction.

- [ ] **`dev-writer`** — `design.md` "Risks / Trade-offs" — the two doors now disagree about the stored form and the entry does not say so.
      `setRemoteSeed` stores and adopts `candidate.seedUrl()` (slash stripped, `radicle_impl.cpp:481`); `setSetting("remoteSeed", "https://x/")` stores `value` verbatim and adopts it via `m_seed.setSeedUrl(value)` (`radicle_impl.cpp:442`). "The stored URL is the one in force" is therefore true of one door only. The Non-Goals cite #65 for `setSetting`'s adoption, but not this divergence, and the trailing-slash `NO SPEC:` test at `test_radicle_impl.cpp:820` is written as though the question were closed. Record it as a known consequence of leaving `setSetting` alone, with where it is tracked.

- [ ] **`dev-writer`** — `design.md:95-104` and `design.md:82-90` — "a store write now also able to fail after a successful probe" is argued, but only one way of failing is tested: the no-path store.
      `a_seed_that_answers_but_cannot_be_stored_is_refused` uses `SettingsStore{""}`. A store with a path whose write fails (unwritable directory, failed rename) goes through the same `isError(stored)` branch, so the code is right, but the entry presents the general case as pinned when only the empty-path instance is. Either say that, or add a test with a real path that fails to write. Suggestion; the code is correct.

- [ ] **`dev-writer`** — `design.md:56-74` — the "Probe before store" and "Store before adopt" bullets give no mutation evidence, while "Shape before probe" and the others do.
      Both are guards. Probe before store is pinned by `a_failed_probe_stores_nothing_and_keeps_the_seed_in_force` going red if the store is moved ahead of the probe, and store before adopt by `a_seed_that_answers_but_cannot_be_stored_is_refused`. Name the test each turns red, as the sibling entries do. Suggestion.

- [ ] **`dev-writer`** — `design.md:56-74` — the guard claim for the shape check is partly unverified prose: "only that test" turns red.
      With the shape check removed, `a_url_with_no_scheme_is_refused_before_any_request_and_says_why` fails on `fake.requested.empty()`, but its earlier `LOGOS_ASSERT_CONTAINS(..., "http://")` can pass on the store's own wrapped message, so the failing assertion is the request count, not the reason. State which assertion goes red, so the entry does not imply the reported reason is pinned independently of the no-request check. Suggestion.
