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

- [ ] **`dev-writer`** — `radicle/src/radicle_impl.cpp:494` (reached after the
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

- [ ] **`dev-writer`** — `radicle/src/radicle_impl.cpp:458-460` and
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

- [ ] **`dev-writer`** — `radicle/src/radicle_impl.cpp:480-481` via
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
