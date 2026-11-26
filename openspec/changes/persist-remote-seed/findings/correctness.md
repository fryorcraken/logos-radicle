# Correctness findings: persist-remote-seed

Dimension reviewed: **correctness** only. Method: read the diff against
`origin/main`, then ran throwaway probe tests and a mutation against the core
unit tests (`nix build ./radicle#checks.x86_64-linux.unit-tests`). Baseline
there is two failures, both the 107-byte socket cap in the Nix build dir
(`starting_a_node_with_no_embedded_identity_says_so_in_the_modes_own_words`,
`embedded_mode_reads_and_writes_its_own_home_and_leaves_the_local_one_alone`);
neither touches this change.

Clean areas: the `SettingsStore::validate` extraction is behaviour-preserving
(unknown-key refusal still precedes it, every per-key branch moved whole, the
no-rule keys fall through to an empty object, and `isError` reads that as
accept). The candidate-client copy shares the transport and discards cleanly on
every refusal, so a failed probe leaves the in-force seed's URL, `reachable` and
`apiVersion` untouched (measured: `a_failed_probe_leaves_the_seed_in_force_reachable`).
Mutation: probing `m_seed` instead of `candidate` is caught, by two tests
(baseline 2 failures, mutated 4). The empty-URL, no-scheme, failed-probe,
unstorable and trailing-slash paths behave as the spec says. The probes below
are the only defects found; the mutation is left in the tree (see hand-off).

- [x] **`dev-writer`** — `radicle/src/radicle_impl.cpp:494` (reached after the
      store and adopt at `:480-489`) — a seed whose `/api/v1` answers valid JSON
      that is not an object makes `setRemoteSeed` throw after it has stored and
      adopted the seed
      **Scenario:** settings file carries `remoteSeed` `https://kept.example.test`;
      the transport answers every request with `[]` (also `null`, `"ok"`, `42`: a
      typo'd URL that lands on some other JSON endpoint). `getJson` returns the
      parsed value, `probe()` copes (`contains` on a non-object is false) and
      `isError` is false, so the call proceeds: `m_settings.set` stores
      `https://array.example.test`, `m_seed` is replaced, `remoteSeedChanged` is
      emitted, and then `index.value("nid", "")` throws nlohmann `type_error` 306
      ("cannot use value() with array"). There is no `try`/`catch` in
      `radicle_impl.cpp`, so the exception leaves the method instead of a
      `{"error":"..."}`. This breaks "A seed setRemoteSeed refuses changes
      nothing" for a seed that does not answer as a Radicle seed, and it is the
      one durable form of the pre-existing throw: the seed is now persisted, so
      it is also what the next start adopts.
      **Measured:** probe test, 190 passed 7 failed against 2 baseline failures:
      the call threw; afterwards `getSettings().remoteSeed` and
      `getCapabilities().remoteSeed` were both `https://array.example.test`
      (expected `https://kept.example.test`) and one `remoteSeedChanged` had been
      emitted. Severity: medium (a refusal that is neither a refusal nor a
      reply, and it is persisted). The `value()` call is pre-existing
      (`origin/main` has it at the same expression); what is new is that it now
      runs after a durable write. Requiring `index.is_object()` before the store
      is the natural check; no test covers a non-object probe reply.
      **Fixed** in "Refuse a seed that is not a Radicle index, and shape-check
      the stored URL". `SeedClient::probe` refuses a reply that is not a JSON
      object with a string `apiVersion` (the spec's new "A seed answers its
      probe only with a Radicle seed index"), so the check sits before the
      store, and the reply's `nid` comes from `SeedClient::nid()` rather than
      `index.value(...)`. Regression test
      `a_success_that_is_not_a_seed_index_is_a_failed_probe` covers `[]`,
      `null`, `"ok"`, `42`, `{}`, `{"apiVersion":6}` and `not json`, asserting
      refusal, the kept seed in force and stored, and no event; watched failing
      first with "type_error.306 cannot use value() with array". Under the fix
      the suite is 198 passed, 2 failed, the two being the socket-cap sandbox
      failures named above.

- [x] **`dev-writer`** — `radicle/src/radicle_impl.cpp:458-460` and
      `radicle/src/seed_client.cpp:25-29` — the shape rule is applied to the URL
      as typed, but the store is asked about the URL after the client strips
      trailing slashes, so `https://` passes the shape check, costs a request,
      and is then refused with the wrong reason
      **Scenario:** `setRemoteSeed("https://")` (also `http://`, `https:///`).
      `validate` accepts it (it starts with `https://`), `candidate` normalises
      it to `https:`, the probe requests `https:/api/v1`, and when that answers
      (the fake in the tests; a real transport would normally fail, giving a
      transport error that names neither the shape nor the seed) `m_settings.set("https:")`
      fails `validate` and the caller is told *"the seed answered, but it could
      not be saved, so it was not adopted: a seed URL must start with http://
      or https://"*, which blames the store for a malformed URL and claims a
      seed answered that was never one. State is intact in every case.
      This contradicts the design's "Shape before probe" decision, whose point is
      that a URL the store would refuse spends no request.
      **Measured:** with a fake answering every request: `fake.requested.size()`
      was 1 (expected 0), the error was the wrapped text above, and
      `getCapabilities().remoteSeed` and `getSettings().remoteSeed` stayed
      `https://kept.example.test`. Severity: low (wrong message and one wasted
      request on a degenerate input; no state damage). A genuine defect against
      the recorded decision, not a style point. Checking the shape of what will
      be stored (`candidate.seedUrl()`) as well as what was typed closes it.
      **Fixed** in "Refuse a seed that is not a Radicle index, and shape-check
      the stored URL", as suggested and as the spec's new "A malformed seed URL
      is refused before any request" now requires: the candidate is built
      first (no request), and `SettingsStore::validate` is asked of both the
      given URL and `candidate.seedUrl()` before the probe. Regression test
      `a_scheme_with_no_host_is_refused_before_any_request` (`https://` and
      `http://`; error naming both schemes, no request, stored seed kept, no
      event) was watched failing first on `fake.requested.empty()`.

- [x] **`dev-writer`** — `radicle/src/radicle_impl.cpp:480-481` via
      `radicle/src/settings_store.cpp:343-361` — a seed pick now rewrites the
      whole settings file from `load()`, which normalises what a newer build
      wrote, so choosing a seed can destroy the stored mode
      **Scenario:** a profile's settings file was written by a newer build and
      carries `"mode":"someNewMode"` (or an extra key). `load()` returns
      `mode: explore` for it and drops keys it does not know, and its own comment
      promises "Nothing is REWRITTEN on disk here ... switching back to a build
      that understands it loses nothing". The seed picker's `setRemoteSeed` now
      calls `m_settings.set`, which saves that normalised object, so the user's
      stored mode becomes `explore` and the newer build's keys are gone, after one
      pick of a seed. This path already existed through `setSetting`, but it was a
      deliberate settings action; the seed picker is an everyday control, and
      before this change picking a seed never wrote the file.
      **Measured:** by reading `load()`/`set()` only; not run. Severity: low to
      medium and narrow (needs a file from a newer or hand-edited build). Not
      covered by a test or named in `design.md`'s risks; either record it as an
      accepted consequence there or have `set` leave keys and values it did not
      change as it found them.
      **Rejected** as a fix in this change; recorded as an accepted consequence
      under `design.md`'s Risks ("A seed pick rewrites the whole settings file
      from `load()`"), in "Fold the review findings into design.md". Having
      `set` preserve what it did not change is a behaviour of every write
      through the store, `setSetting`'s included, and no requirement asks for
      it, so building it here would be building past the spec; whether a
      write must preserve a newer build's keys and mode is an observable
      behaviour for the `spec-writer` to decide, and is in the hand-back as
      such. No issue tracks it yet (#63 covers a read leaving the file alone,
      not a write); filing one is a `gh issue` write, left to the runner.

## Round 2

Dimension reviewed: **correctness** only, against the spec as it now stands
(`git diff 1973b69 HEAD`, concentrating on `SeedClient::probe`/`getJson`,
`setRemoteSeed`, `SettingsStore::set`'s no-path message). Method: read the
diff, then a throwaway test through `nix build
./radicle#checks.x86_64-linux.unit-tests` (the log read from `nix log`, since
the tool result truncates). Baseline is the same two socket-cap failures as
round 1; nothing new fails except the probe below.

Clean areas: the three round 1 fixes hold. The dual shape check
(`seedUrl` and `candidate.seedUrl()`) refuses `https://` and `http://` before
any request, and cannot reject a URL the store would accept: `validate`'s only
`remoteSeed` rule is the scheme prefix, and normalisation only removes trailing
`/`, so a URL that passes as typed fails as stored only when it was nothing but
a scheme plus slashes. `probe()` clears `m_apiVersion`/`m_nid` first and is
called only on the candidate (its sole caller is `setRemoteSeed`), so the
clearing cannot wipe what the seed in force learned. `nid()` is empty for an
absent or non-string `nid`. The `getJson` change closes the
`{"error":5}` throw (`parsed["error"].get<std::string>()` on a non-string),
and a non-2xx status never reaches the index check. `SettingsStore::set`'s
empty-path branch comes after `validate` and the unknown-key refusal, so it
changes no earlier refusal; its comment's claim that only an unset `XDG_DATA_HOME`
and `HOME` give an empty path matches `settingsPathFor`. `openspec validate
persist-remote-seed --strict` passes. A scheme-plus-host-less URL that carries a
path (`https:///x`) passes both shape checks, as the spec requires (it says
"`setSetting` would refuse"); that is consistent, not a defect.

**The tester's open question, answered.** A 2xx body
`{"apiVersion":"6.2.0","error":"x"}` is a JSON object with a string
`apiVersion`. The spec, as written, says that answers: "A seed answers its
probe only with a Radicle seed index" lists the failures exhaustively ("Every
other outcome is a failed probe ... a transport error, any other status, a body
that is not JSON, JSON that is not an object, or an object with no string
`apiVersion`"), and an `error` member is none of them. So under the spec the
call must be accepted, and the code is the side that disagrees: `getJson`
returns the parsed body for a 2xx, and `probe()` then asks `isError(index)`,
which is true for any object with an `error` key, whatever else it carries.

- [ ] **`dev-writer`** — `radicle/src/seed_client.cpp:142-143` (`probe()`) —
      a 2xx index that carries an `error` member is refused although the spec
      says a body with a string `apiVersion` answers, and the refusal is not
      `{"error":"..."}`
      **Scenario:** `setRemoteSeed("https://other.example.test")` with a
      settings file carrying `https://kept.example.test` and the probe
      succeeding with 2xx and the body `{"apiVersion":"6.2.0","error":"x"}`
      (also `{"apiVersion":"6.2.0","error":5}`). The spec's "A body carrying a
      string apiVersion answers" requires a reply with no `error` member and
      `remoteSeed` stored; the call instead returns the seed's own body
      unchanged, `{"apiVersion":"6.2.0","error":"x"}`, which is neither the
      acceptance nor the `{"error":"..."}` shape the failure scenarios require
      (for `"error":5` the `error` member is not even a string, so a caller
      reading `.error` as text gets a number). Nothing is stored and no event
      is emitted, so state is intact; it is the reply and the verdict that are
      wrong. The cause is that `isError` conflates "the transport reported a
      failure" with "the seed's JSON happens to have an `error` key", and
      `probe()` returns `index` itself on the first.
      **Measured:** a throwaway test, `setRemoteSeed` over a fake answering
      `/api/v1` with `{"apiVersion":"6.2.0","error":"x"}`: reply
      `{"apiVersion":"6.2.0","error":"x"}`, stored `remoteSeed` still
      `https://kept.example.test`, and `ASSERT failed: !(parse(raw).contains("error"))`
      (the second body was not reached because the assertion aborts the test).
      No existing test covers a 2xx index with an `error` member; the
      `{"error":5}` test is on a 500. Severity: low, because a real seed's
      index does not carry `error`; but the code and spec disagree on a body
      the spec names, so one of them changes. Either route is small: have
      `probe()` decide on the shape of a 2xx body (`is_object` plus string
      `apiVersion`) before asking `isError`, keeping a transport failure
      distinct from a body with an `error` key, or, if refusing such a seed is
      the intent, have the `spec-writer` add "and no `error` member" to the
      requirement and a scenario for it, and have the refusal reply be
      `makeError(...)` rather than the seed's body. Whichever is chosen, the
      refusal must not hand the seed's raw body back as the reply.
