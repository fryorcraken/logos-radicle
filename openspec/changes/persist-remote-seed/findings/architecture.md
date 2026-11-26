# Architecture review — persist-remote-seed

Dimension reviewed: **architecture** only (shape, data structure, one-function-one-job,
comments, the CLAUDE.md "How to shape a change" rules). Correctness, security and
readability are other instances'.

**Clean areas.**

- The ordering (shape, probe, store, adopt) and the candidate-client shape are
  sound. Copying `m_seed` into `candidate`, probing it, storing, then
  `m_seed = std::move(candidate)` makes "a refusal changes nothing" true by
  construction, with no rollback site to get right. This is the "complexity in
  the data structure, not the logic" habit done correctly, and it removes the
  old defect (the rollback's `setSeedUrl` cleared `reachable`/`apiVersion`).
- Extracting `SettingsStore::validate` as a no-behaviour-change commit before
  the behaviour change follows "make the change easy, then make the easy
  change". `set` now reads as load, refuse-unknown, validate, write, save, and
  `setRemoteSeed` asks the same function rather than keeping a copy of the
  `http(s)://` rule.
- `setRemoteSeed` is one job (change the seed) with four ordered gates; it has
  no `And` in its name and no second caller. The interface in `radicle_impl.h`
  is unchanged (`std::string` in, JSON out), so nothing was widened across the
  FFI boundary.
- No new dependency, no build-file or CI-gate path moved.
- A repo-wide search of `radicle/src`, `radicle-ui/src`, `docs/writes.md`,
  `docs/rust-ffi.md` and `CLAUDE.md` for "roll back", "in-memory" and "never
  persist" found no remaining live comment that says `setRemoteSeed` fails to
  store; the only hits are the new comments themselves.
- Test helpers (`freshSettingsPath`, `settingsCarryingSeed`, `kSeedIndex`) are
  small, single-purpose and named for what they do.

Findings below are low severity. None is a defect in behaviour. The first,
second and fourth are stylistic; the third is a documented-claim accuracy gap;
the fifth is a missing contract note.

- [x] **`dev-writer`** — `radicle/src/settings_store.h:13-22` — the class comment now carries a changelog paragraph
      **Scenario:** the added paragraph ("for a while after this store landed
      adopted a seed in memory only, so the chosen seed was still lost on
      restart") describes a past state of `setRemoteSeed`, not what this class
      does. It says nothing a reader of `SettingsStore` needs, and it is
      exactly the "accuracy with a short half-life" the repo's CLAUDE.md
      ("Keeping this file true", specflow "Keeping documents true") tells
      writers to stop appending; "for a while" is already undated. The lasting
      content is the second sentence (both `setRemoteSeed` and
      `setSetting("remoteSeed", …)` write the key; see `radicle_impl.h`); the
      history belongs in `design.md`, which already has it. **Severity:** low,
      stylistic preference.
      **Fixed** in "Say what the settings store and the seed picker do, not
      what they did": the paragraph keeps only the lasting content (both
      methods write the key; `radicle_impl.h` says how they differ). Comment
      only.

- [x] **`dev-writer`** — `radicle/src/radicle_impl.cpp:462-468`, `radicle/tests/test_radicle_impl.cpp` (the `a_failed_probe_stores_nothing_and_keeps_the_seed_in_force` doc comment, `a_seed_that_answers_but_cannot_be_stored_is_refused`, the `NO SPEC:` block above `a_failed_probe_leaves_the_seed_in_force_reachable`) — comments narrate the code this change replaced
      **Scenario:** "The rollback this replaced re-set the old URL…", "The
      rollback this milestone protects, now for…", "Before this change the seed
      was adopted for the session…". Each is true today and meaningless once the
      old code is out of the reader's memory ("what rollback? what milestone?").
      The reason to prefer the candidate client over the obvious
      mutate-and-restore is a legitimate "why this and not the obvious
      alternative" comment and should stay, but phrased as the property
      (`setSeedUrl` clears `reachable`/`apiVersion`, so restoring the URL would
      also forget that the old seed answered) rather than as a diff against
      deleted code. **Severity:** low, stylistic preference.
      **Fixed** in "Refuse a seed that is not a Radicle index, and shape-check
      the stored URL", all four sites. `radicle_impl.cpp` states the
      `setSeedUrl` property; in `test_radicle_impl.cpp` the
      `a_failed_probe_stores_nothing…` doc says what it pins, the
      `a_seed_that_answers_but_cannot_be_stored…` comment drops "Before this
      change", and the block above `a_failed_probe_leaves_the_seed_in_force_reachable`
      (now a requirement, no longer `NO SPEC:`) names the property and the
      implementation it rules out. The two test comments are also the
      `tester`'s box in `readability.md`, which is left for the `tester`.
      Comments only; no test can see them.

- [ ] **`dev-writer`** — `radicle/src/radicle_impl.cpp:441-443`, `radicle/src/radicle_impl.cpp:478-479`, `design.md` "The stored URL is the one in force" — the claim that stored, in-force and reported URLs "all agree" holds for one of the two doors only
      **Scenario:** `setSetting("remoteSeed", "https://x.example.test/")` stores
      the raw string (`m_settings.set(key, value)`) and adopts the normalised
      one (`m_seed.setSeedUrl` strips the slash), so `getSettings().remoteSeed`
      is `https://x.example.test/` while `getCapabilities().remoteSeed` is
      `https://x.example.test`. `setRemoteSeed` stores the normalised value, so
      the same key now holds two representations depending on which door wrote
      it, and the comment at 478-479 and the design section read as a general
      invariant. This was latent before the change; the change makes it a
      stated property that is half-true and leaves the second writer on a
      different rule. It is out of scope to fix here (the `setSetting` seed path
      is #65), but `design.md` should say so under Non-Goals or Risks, so the
      claim is scoped to `setRemoteSeed`. **Severity:** low; documentation
      accuracy, not a regression.

- [x] **`dev-writer`** — `radicle/src/seed_client.h:21-89`, `radicle/src/radicle_impl.cpp:469` — `setRemoteSeed` now depends on `SeedClient` staying a cheap, copyable value type, and the class does not say so
      **Scenario:** `design.md` justifies the copy ("a URL, three cached
      fields and a `std::function` transport, so the copy is cheap and shares
      the transport"). `seed_client.h` carries no note of it. A later change
      that adds a member such as a response cache (a `shared_ptr` the copy would
      silently share with the in-force client, defeating the "candidate" isolation),
      an in-flight request handle, or a mutex either breaks the build here for a
      reason far from the edit, or compiles and stops isolating. One sentence on
      the class, "copied by `RadicleImpl::setRemoteSeed` to probe a candidate;
      keep it a value type", puts the constraint where the next editor of the
      class will read it. **Severity:** low.
      **Fixed** in "Refuse a seed that is not a Radicle index, and shape-check
      the stored URL": the class comment in `seed_client.h` now opens a "Keep
      this a value type" paragraph naming `RadicleImpl::setRemoteSeed`'s copy
      and the two ways a new member breaks it (shared mutable state reaching
      the client in force, or a copy that stops compiling far from the edit).
      Comment only; no test can see it.

- [x] **`dev-writer`** — `radicle/src/settings_store.h:109-119` — the public `validate` contract is incomplete about what it does
      **Scenario:** the doc says validation happens "without storing anything",
      which suggests a pure check. For `gitPath` it runs `LocalReader::gitProbe`,
      which spawns the configured executable; a caller reading only the header
      who passes arbitrary `(key, value)` pairs (it is now a public static taking
      any key) would not expect a subprocess. Success is also signalled by an
      empty object, which is consistent with the module's `isError` convention
      but unstated. `setRemoteSeed` only ever passes `remoteSeed`, so nothing is
      wrong today; a clause on the header ("`gitPath` is checked by running it")
      is what keeps the next caller from learning it the hard way. A narrower
      `validateRemoteSeed` was weighed and rejected in `design.md` with a
      reason that holds, so this is not a request to reshape. **Severity:** low.
      **Fixed** in "Say what the settings store and the seed picker do, not
      what they did": the header now says success is an empty object (which
      `isError` reads as an acceptance) and that a non-empty `gitPath` is
      checked by running it, spawning a process. Comment only.
