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

## Round 2

Reviewed: the current delta `specs/module-settings/spec.md` and `proposal.md`
against `radicle/tests/test_radicle_impl.cpp`, `test_seed_client.cpp` and
`test_settings_store.cpp` at HEAD `35f8d35`, and `gh issue view 55`. `openspec
validate persist-remote-seed --strict` passes.

**Coverage (part 1).** Every scenario of the five new requirements and the
MODIFIED one has a test that can fail, at the right layer (the C++ core suite,
which sees the whole `setRemoteSeed` call, the settings file and the event
sink). The restart scenarios use a second instance over one file with a default
`SeedClient`, after asserting the file absent. The "no event" assertions have
their control in `an_accepted_seed_emits_one_remote_seed_changed_naming_it`. The
two-trailing-slash test pins the request URL, the reply, capabilities, settings
and the event against written-out strings. `a_failure_status_is_a_failed_probe_whatever_its_body`
and `a_success_that_is_not_a_seed_index_is_a_failed_probe` check
`requested.back()` names the new host, so the refusal is the probe's. I found
no scenario without a test and none untestable as written.

**The tester's question: `{"apiVersion":"6.2.0","error":"x"}` as a 2xx body.**
The spec's text says the seed answers: the definition is a 2xx status and a body
that is a JSON object whose `apiVersion` is a string, every other outcome is a
failed probe, and nothing mentions an `error` member. No test pins it either
way. **Measured:** I added a temporary test driving exactly that body at status
200 and asserting acceptance; it failed on `!(out.contains("error"))`
(`test_radicle_impl.cpp:1033` in that build). The command was
`nix build ./radicle#checks.x86_64-linux.unit-tests -L --out-link result-test`,
and the result was read in full from `nix log
/nix/store/hxh8pmb6xvlqnaqhccdany5h9a915idx-logos-radicle-tests-0.2.3.drv`:
200 passed, 3 failed, the three being my test and the two sandbox
socket-path-cap failures the overlay names (`starting_a_node_with_no_embedded_identity_says_so_in_the_modes_own_words`,
`embedded_mode_reads_and_writes_its_own_home_and_leaves_the_local_one_alone`).
So the code refuses a seed the spec says answers, and no test notices.

**Mutations (part 2).** One, the temporary test above, a mutation of the suite
rather than of the code. Restored (`git checkout` of the file); nothing is left
in the tree. I judged the rest by reading: a string-`apiVersion` check mutated
to "any object" is killed by the `{}` and `{"apiVersion":6}` rows; stripping
only one trailing slash by the two-slash test; emitting on refusal or with the
typed URL by the event assertions; mutate-then-restore by
`a_failed_probe_leaves_the_seed_in_force_reachable`.

**NO SPEC (part 3).** One marker is left, `a_store_with_no_path_says_why_it_cannot_write`
(`test_settings_store.cpp:669`). It is honest: the live "A store with no path
reports defaults and refuses to write" scenario requires the refusal and is
silent on its wording, so the marker names a real gap, a low one, and I do not
raise it again. The four round 1 markers are gone and each of those behaviours
is now a scenario. The one unmarked behaviour I found is the first box below
the question.

**Spec soundness (part 5).** The delta is self-consistent: "A seed
setRemoteSeed refuses changes nothing" and "A malformed seed URL is refused
before any request" overlap on shape refusals without contradicting, and every
requirement's text is under the strict limit. The issue's three "Done when"
items are pinned and the proposal's extra refusals are recorded decisions, not
stale scope. Part 4 does not apply. `proposal.md` says "No UI change", while
the branch changed `radicle-ui/src/qml/Main.qml` by 6 lines added and 6
removed since round 1; I did not read it and raise no box on it, but whoever
owns that claim should confirm the edit changes no behaviour, since a UI change
would need a UI test.

- [x] **`spec-writer`** — `specs/module-settings/spec.md`, "A seed answers its
      probe only with a Radicle seed index"
      **Scenario:** a 2xx reply `{"apiVersion":"6.2.0","error":"x"}` is, by the
      text, an answer: an object, a string `apiVersion`, nothing excluding
      `error`. The code refuses it. The module's one failure shape is
      `{"error":"..."}`, so a body carrying an `error` member reads as a failure
      to every caller that checks for one, and the code's refusal may be the
      intended behaviour. Decide it and say it: either add "and no `error`
      member" to what answers (and to the list of failed probes), or state that
      such a body answers and have the code follow.
      **Measured:** temporary test at status 200 with that body failed on
      `!(out.contains("error"))`; 200 passed, 3 failed, two being the sandbox
      failures (command and log above). **Severity:** medium, because the spec
      is the gate on what gets persisted and today the text and the code
      disagree.
      **Outcome: fixed (spec): such a body is a failed probe.** Decided under
      the owner's delegation. The requirement now takes a seed to answer only
      on a 2xx JSON object holding a string `apiVersion` **and no `error`
      member**, and requires every failed probe to reply with only an `error`
      member, a non-empty string, never the seed's body relayed. New scenario
      "A success carrying an error member is a failed probe" drives 2xx bodies
      `{"apiVersion":"6.2.0","error":"x"}`, `…"error":null`, `…"error":""`,
      `{"error":false}` and `{"error":5}`, asserting a non-empty string
      `error`, no other member, both seeds unchanged and no event. "A failure
      status is a failed probe whatever its body" gains `{"error":""}` and the
      same two reply assertions, since an empty `error` reads as success in the
      view's parser too. The code's refusal now meets the text on acceptance;
      its relaying of the body does not, which is the `dev-writer` entries in
      `correctness.md` and `security.md`. Reasons are in `proposal.md`.

- [ ] **`tester`** — `test_radicle_impl.cpp`, after the spec-writer entry above
      lands
      **Scenario:** no test pins a 2xx body that is an object with a string
      `apiVersion` and an `error` member, so an implementation can answer or
      refuse it and stay green. Add a scenario-named test for whichever way the
      spec goes, with the body above, asserting the reply, `getSettings()`,
      `getCapabilities()` and the event. **Severity:** medium, paired with the
      entry above.

- [ ] **`tester`** — `every_url_set_setting_refuses_is_refused_by_set_remote_seed_though_it_would_answer`
      (`test_radicle_impl.cpp`)
      **Scenario:** "A malformed seed URL is refused before any request" says
      the refusal is without a request and carries `setSetting`'s reason, for
      every URL `setSetting` refuses. The scenarios name only a missing scheme
      and `https://`/`http://`, and those two tests check `fake.requested` and the
      message. The six rows of this loop (`ftp://`, `//host`, leading space,
      `https:/host`, `https:host`, `HTTPS://`) check neither. An implementation
      that screens only for a missing `://` sends the request for `ftp://seed...`,
      sees it answer, and is refused later by the store, so this loop stays
      green while the requirement is broken for those rows. Assert
      `fake.requested.empty()` and the control's own message in the loop. **Not
      measured:** reasoned from the test body; I did not mutate the code, which I
      may not read. **Severity:** low-medium.

- [ ] **`tester`** — `a_failure_status_is_a_failed_probe_whatever_its_body` and
      the success rows (`test_radicle_impl.cpp`)
      **Scenario:** the definition is "a 2xx status" and "any other status" is
      a failed probe, but the only successful status ever driven is 200 and the
      only failures are 404, 500 and a transport error. An implementation that
      accepts only `status == 200` refuses a seed answering 201, which the spec
      says answers, and one that accepts `status < 400` accepts a 3xx the spec
      says fails. `FakeSeed::replyStatus` already takes a value: add a 201 with
      a seed index (accepted) and a 301 with a seed index (refused, nothing
      stored). **Severity:** low.

- [ ] **`tester`** — `an_error_status_surfaces_the_seeds_own_message`
      (`test_seed_client.cpp`)
      **Scenario:** it pins that a seed's own error text reaches the caller, and
      no scenario of any spec describes that. It is written as the control for
      the next test, which the delta's "A failure status is a failed probe
      whatever its body" does cover. Either mark it `NO SPEC:` with what was
      chosen, as the project's rule requires, or fold it into the control role
      its comment gives it. **Severity:** low.
