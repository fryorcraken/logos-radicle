# spec-test review: persist-remote-seed

Reviewed: the delta `specs/module-settings/spec.md` against the live
`openspec/specs/module-settings/spec.md`, the issue (`gh issue view 55`), and
`radicle/tests/test_radicle_impl.cpp`. `openspec validate persist-remote-seed
--strict` passes.

**Coverage (part 1): every scenario has a test at a layer that can see it.**
All seven ADDED scenarios map to the `set_remote_seed` / seed-persistence tests
between lines 543 and 751 (restart, stored-before-reply, later-of-two, failed
probe, unstorable seed, no scheme, empty URL). The MODIFIED requirement's
scenario is the pre-existing
`a_seed_that_cannot_be_reached_is_accepted_by_a_settings_write`. The restart
tests model a restart as a second instance over one file, with a default
`SeedClient`, so what it reports can only have come from the file. The seed
URLs are not the default, and the paths are removed and asserted absent first,
so a run that wrote nothing cannot pass. The "refusal emits no event" scenarios
are backed by a control, the accept test that sees exactly one event, so
the no-event assertions are not vacuous. The multi-shape loop asks `setSetting`
first whether it refuses each row, so a row that is really valid cannot hide in
the table.

**Mutations (part 2).** Both ran through
`nix build ./radicle#checks.x86_64-linux.unit-tests --out-link result-test`, with
the log read from `nix log <drv>` in full, because the build's own output
truncates in the middle. The baseline has two failures that belong to the
sandbox, not the change: `starting_a_node_with_no_embedded_identity_says_so_in_the_modes_own_words`
and `embedded_mode_reads_and_writes_its_own_home_and_leaves_the_local_one_alone`
(the 107-byte control-socket cap, per the overlay). Both are restored; nothing
is left in the tree.

1. Deleted the `if (seedUrl.empty()) return error` guard in `setRemoteSeed`.
   **Killed** by `an_empty_url_is_refused_and_leaves_the_stored_seed_alone`
   (`ASSERT failed: out.contains("error")`, line 741). That test is the only one
   that reddened, and it is the spec scenario's own. Its `replies[""]` trick
   works.
2. Deleted the `if (isError(shape)) return` line, the `SettingsStore::validate`
   check ahead of the probe. **Only**
   `a_url_with_no_scheme_is_refused_before_any_request_and_says_why` went red
   (`fake.requested.empty()`, line 726), exactly as `design.md` claims. The two
   spec-required tests, `a_url_with_no_scheme_is_refused_even_when_the_seed_would_answer`
   and the every-shape loop, stayed green because the store's own `validate`
   inside `set` refuses the same URL after the probe. That is not a test defect:
   the observable result the spec requires is identical either way. It does mean
   the shape-first ordering is held only by a `NO SPEC` test (see the last box).

I also probed one thing no test covers: a first-run `setRemoteSeed` over a
settings path whose parent directory does not exist. Every test fixture
pre-creates the directory (`scratchSettingsPath`), and production starts with no
`radicle-module/`. I added a temporary test; it passed (190 passed, the same two
sandbox failures), so the store creates the directory and there is no finding.
The probe test was reverted.

**NO SPEC (part 3).** Four `NO SPEC:` tests, each a spec gap below. I found no
unmarked behaviour pinned beyond them. The uppercase-scheme and leading-space
rows of the loop follow from the live "begin with `http://` or `https://`" rule.

**Spec soundness (part 5).** The delta and the live spec are consistent. The
MODIFIED requirement keeps its header and scenario, and its text is well under
500 characters. The delta no longer states the old "a settings write" wording
that `setRemoteSeed` now falsifies. The issue's "Done when" items (survive a
restart, failed probe persists nothing, distinct-URL tests) are all pinned, and
the proposal's extra refusals (shape, unstorable) are a recorded decision, not
stale scope. No requirement moved between capabilities, so part 4 does not
apply.

- [x] **`spec-writer`** — `specs/module-settings/spec.md`, both new requirements
      use "answers its probe" / "a seed whose probe fails", and nothing defines it
      **Scenario:** no live spec mentions the seed probe (`grep -rni probe
      openspec/specs` hits only the unrelated write-probe in `embedded-identity`
      and `source-modes`). The tests drive a transport error (`failEverything`)
      and a 404. A seed that replies 200 with a body that is not an index, or one
      with no `apiVersion`, is neither required to be accepted nor refused by the
      text, and no test pins it. Whichever way the code goes, the spec cannot
      call it wrong. State what an answer is: the request, and what response
      counts as answering. Then the tester can pin the 200-with-garbage case.
      **Severity:** medium, the gate on what gets persisted.
      **Outcome: fixed (spec).** New requirement "A seed answers its probe only
      with a Radicle seed index": the probe is one GET of
      `<normalised URL>/api/v1`; it answers only with a 2xx status and a body
      that is a JSON object whose `apiVersion` is a string. A transport error,
      any other status, non-JSON, non-object JSON, or an object with no string
      `apiVersion` is a failed probe, refused under "A seed setRemoteSeed
      refuses changes nothing". Scenarios pin the control (`{"apiVersion":"6.2.0"}`
      answers), 2xx bodies `[]`, `null`, `"ok"`, `42`, `{}`,
      `{"apiVersion":6}`, `not json` (each refused, nothing stored or adopted,
      no event), and status 500 with `{"apiVersion":"6.2.0"}` and `{"error":5}`
      (each refused). This is the definition the `dev-writer` entries in
      `correctness.md` (non-object reply) and `security.md` (non-object reply,
      `{"error":5}`, any-JSON-is-a-seed) now have to meet; none of their code
      or tests is changed here. Not measured: that every real seed's `/api/v1`
      carries a string `apiVersion` (the module's own contract and fixtures
      assume it; no seed was reachable from this sandbox).

- [x] **`spec-writer`** — `NO SPEC` at
      `an_accepted_seed_emits_one_remote_seed_changed_naming_it` (line 618)
      **Scenario:** the delta requires a refusal to emit no `remoteSeedChanged`
      and says nothing about an acceptance. The test pins one event carrying the
      seed now in force. An implementation that stopped emitting on acceptance
      satisfies every scenario, and a view listening for the event would never
      learn of a pick. Add a requirement that an accepted seed emits it once,
      naming the seed in force. **Severity:** low-medium; it is also the control
      the no-event assertions rely on.
      **Outcome: fixed (spec).** New requirement "An accepted seed is reported
      as its normalised URL" requires exactly one `remoteSeedChanged` on
      acceptance, carrying the normalised URL; scenario "An accepted seed emits
      one event naming it". The `NO SPEC:` marker on the test can now go.

- [x] **`spec-writer`** — `NO SPEC` at
      `the_stored_seed_is_the_url_in_force_without_a_trailing_slash` (line 820)
      **Scenario:** the requirement says the store holds "that URL"; the
      implementation stores the URL as the client normalised it. Input
      `https://chosen.example.test/` stores and reports
      `https://chosen.example.test`. A reader of the spec would expect the
      stored value to equal the argument. State the normalisation, or say that
      stored, in-force and replied seed agree. **Severity:** low.
      **Outcome: fixed (spec), both ways.** "An accepted seed is reported as
      its normalised URL" defines the normalised URL (the given URL with every
      trailing `/` removed) and requires the reply's `seed`,
      `getCapabilities().remoteSeed` and the event to be it; "A seed accepted
      by setRemoteSeed survives a restart" now stores the normalised URL rather
      than "that URL". Scenario "A trailing slash is removed wherever the seed
      is reported" checks all three of reply, capabilities and settings. The
      same requirement also pins the reply's `nid` (the probe's when a string,
      else empty), which the probe definition above made reachable with a
      non-string `nid`. The `NO SPEC:` marker can go.

- [x] **`spec-writer`** — `NO SPEC` at
      `a_url_with_no_scheme_is_refused_before_any_request_and_says_why` (line 711)
      **Scenario:** the spec requires only that a malformed URL be refused. The
      shape-first ordering (no request made, and the refusal carries the shape's
      reason) is held by this one test.
      **Measured:** deleting the shape check in `setRemoteSeed` turned only this
      test red; the spec-required `…_even_when_the_seed_would_answer` and the
      every-shape loop stayed green (command above). If the ordering matters,
      the spec should say so. If it does not, the test is pinning an
      implementation choice. **Severity:** low.
      **Outcome: fixed (spec): the ordering matters.** Both halves are
      observable to a caller (a typed URL is or is not sent over the network;
      the message does or does not name the shape), and the live "A settings
      refusal names what was wrong" already holds `setSetting` to that. New
      requirement "A malformed seed URL is refused before any request": a URL
      whose given or normalised form `setSetting` would refuse is refused with
      no request made, carrying `setSetting`'s reason. Covering the normalised
      form takes in `https://` and `http://` (normalised to `https:` /
      `http:`), which is the `dev-writer` entry in `correctness.md` on the
      wasted request and wrong reason; scenario "A scheme with no host is
      refused before any request" pins it. The `NO SPEC:` marker can go.

- [x] **`spec-writer`** — `NO SPEC` at
      `a_failed_probe_leaves_the_seed_in_force_reachable` (line 792)
      **Scenario:** "leaves the seed in force as it was" is observed by the spec
      only through `getCapabilities().remoteSeed`. The test pins that
      `remoteReachable` and `remoteApiVersion` also survive a refused pick. A
      mutate-then-restore implementation, which the old code was, satisfies
      every scenario while making a working seed read as unreachable after a
      mistyped pick. Say whether "the seed in force" includes what was last
      learned about it. **Severity:** low-medium, a user-visible regression the
      scenarios cannot catch.
      **Outcome: fixed (spec): it does.** "A seed setRemoteSeed refuses changes
      nothing" now requires a refusal to leave the seed in force as it was,
      including the `remoteReachable` and `remoteApiVersion` `getCapabilities`
      reports for it. New scenario "A failed probe leaves the seed in force
      reachable". The `NO SPEC:` marker can go.
