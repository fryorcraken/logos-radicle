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

- [x] **`dev-writer`** — `radicle/src/radicle_impl.cpp:441-443`, `radicle/src/radicle_impl.cpp:478-479`, `design.md` "The stored URL is the one in force" — the claim that stored, in-force and reported URLs "all agree" holds for one of the two doors only
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
      **Fixed** in "Fold the review findings into design.md": the Decisions
      entry is now "The URL is normalised once, and everything reports that
      form", citing the spec's "An accepted seed is reported as its normalised
      URL", which is scoped to `setRemoteSeed`; a Risks entry, "The two doors
      store the seed in different forms", states the `setSetting` divergence
      and that "normalised once" holds for `setRemoteSeed` only, with #65 as
      the issue owning `setSetting`'s seed path (it does not yet name this;
      flagged to the runner). The comment at `radicle_impl.cpp` beside the
      store now says only that this call's stored seed, in-force seed and
      reply agree. The same point as `design-review.md`'s second box.

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

## Round 2

Dimension reviewed: **architecture** only, at HEAD `35f8d35`, concentrating on
what changed since `1973b69` (`git diff 1973b69 HEAD -- radicle/src
radicle-ui/src`, `design.md`, the spec delta). Round 1's five boxes above are
answered and not re-raised.

**Clean areas.**

- **Moving the seed-ness check into `SeedClient::probe` is the right owner.**
  `probe` is where `apiVersion` and `nid` were already read, so "is this a
  seed" and "what did it say" are decided in one function; `setRemoteSeed`
  cannot second-guess a "success" because `probe` no longer returns one it
  would have to distrust. The reply reading `m_seed.nid()` removes the second,
  throwing opinion about `nid` that round 1's shape had. `probe` has exactly
  one production caller (`setRemoteSeed`, on the candidate), so tightening its
  contract widened nothing.
- **`getJson`'s `is_string()` guard sits at the shared choke point**, so every
  `remote*` call gets it rather than only the probe. That is the right place,
  and `design.md` says why.
- **The candidate-client shape survived the rework.** The new shape check runs
  on `candidate.seedUrl()` after the copy, still before any request, and still
  leaves `m_seed` untouched on every refusal. The "Keep this a value type"
  comment is now on the class, which answers round 1's fourth box.
- **Who owns the normalised URL is coherent.** `SeedClient` normalises (ctor
  and `setSeedUrl`, one `trimTrailingSlash`); `setRemoteSeed` stores and
  reports `candidate.seedUrl()` and never trims itself; the persisted value is
  re-normalised on load by the same `setSeedUrl`. There is one trim, and no
  second copy in `radicle_impl.cpp` or `settings_store.cpp`. The only
  disagreement is the `setSetting` door, which `design.md` Risks records.
- **The empty-store-path message is fixed in `SettingsStore::set`**, not in
  `setRemoteSeed`'s wrapper, so both doors benefit; no interface widened
  (`std::string` in, JSON out), no dependency added, no CI gate path moved.

Findings below are low severity; none is a regression. The first is a
documentation-accuracy gap with a stylistic reshape attached, the second a
contract the new `probe` doc states and the code does not keep.

- [x] **`dev-writer`** — `design.md` "The shape rule is asked of the URL as given and as stored", `radicle/src/radicle_impl.cpp:467-477` — the stated reason for checking the typed form is wrong, and no test pins it
      **Scenario:** `design.md` says "Checking the normalised form alone would
      satisfy today's rule, since a normalised URL is a prefix of the given
      one and the rule is a prefix test", and gives a hypothetical length cap as
      the reason to ask both. It is not true of today's rule. `validate`
      accepts an empty `remoteSeed` (it means "clear" through `setSetting`), and
      `setRemoteSeed("/")` or `setRemoteSeed("///")` normalises to `""`, which
      passes the rule. Asking the normalised form alone would then probe
      `"" + "/api/v1"` and report "whatever the transport makes of it", the very
      thing the comment above the loop says to avoid. The typed-form half of the
      loop is what refuses these today, and the only reason it does is that the
      typed `/` fails the `http(s)://` prefix. No row in
      `every_url_set_setting_refuses_is_refused_by_set_remote_seed_though_it_would_answer`
      is slash-only (`//seed.example.test` normalises to itself), so by reading,
      dropping `seedUrl` from the initializer list survives the suite; I did not
      run that mutation (a Nix build for one edit), so treat "survives" as read,
      not measured.
      **Ask:** correct the design sentence to name the real reason (a
      slash-only URL normalises to the empty string the rule accepts), and
      either add a slash-only row to that table or, simpler and the reshape the
      "complexity in the data" habit points at, refuse an empty *normalised* URL
      in place of the typed `seedUrl.empty()` check and validate the normalised
      form alone. That is one check on one form, and it makes the loop and the
      two-form comment unnecessary. **Severity:** low; accuracy of a recorded
      decision, and a stylistic preference for the reshape.
      **Fixed** by correcting the reason and adding the rows; the reshape is
      **rejected**, with the argument now in `design.md`. You were right on
      both counts. `design.md` now says each form catches what the other lets
      through, with `/` and `///` (normalising to the empty string the rule
      accepts) as the given form's case, and the comment beside the loop in
      `radicle_impl.cpp` says the same (`3ec2b22`). `/` and `///` are rows of
      `every_url_set_setting_refuses_is_refused_by_set_remote_seed_though_it_would_answer`,
      which now also asserts no request and that the refusal contains
      `setSetting`'s own reason on every row. Measured, so no longer "read,
      not measured": dropping `seedUrl` from the initializer list turns that
      test red on the `/` row (`out.contains("error")` false; the seed was
      probed and stored empty). The reshape was weighed and not taken for two
      reasons. The spec requires the refusal to carry `setSetting`'s reason,
      which for `/` is "must start with http:// or https://", not "seed URL is
      required", and the new reason assertion holds the code to it. And the
      reshape is correct only because the empty string is the rule's one gap
      under normalisation, which puts knowledge of the rule's internals in
      `setRemoteSeed`; asking `validate` of both forms keeps it in `validate`.

- [x] **`dev-writer`** — `radicle/src/seed_client.cpp:142-143`, `radicle/src/seed_client.h:51-58` — `probe` runs `isError(index)` on the seed's own body, so the documented acceptance bar and the code disagree for a 2xx index that carries an `error` key
      **Scenario:** `probe`'s header says a seed answers with "a 2xx reply whose
      body is a JSON object carrying a string `apiVersion`" and that anything
      else leaves `reachable()` false. A 200 body `{"apiVersion":"6.2.0","error":5}`
      meets that bar, but `getJson` returns it as a success (and sets
      `m_reachable = true`) and `probe`'s `isError(index)` then reads the seed's
      key as *our* error signal and returns the index unchanged.
      `setRemoteSeed` dumps it, so the caller gets `{"error":5,"apiVersion":"6.2.0"}`:
      a failure whose `error` is not a string, which breaks "failures are always
      `{"error":"..."}`", with `reachable()` left true. With a string `error`
      the seed is refused under the seed's own message, which is defensible but
      not what the header says. Nothing refused is stored, so no state is
      corrupted; the defect is the contract. The cause is structural: `getJson`
      returns the untrusted body and our error object as the same type with the
      same discriminator, so a seed can forge the discriminator. A seed index
      with an `error` key is unlikely in practice, hence low.
      **Ask:** either test the object/`apiVersion` bar before `isError` (an
      error produced by `getJson` is `{"error":"<string>"}` with no
      `apiVersion`, so the order can be object-with-`apiVersion` first), or say
      in the header that an index carrying `error` is refused. Not asked: a
      reshape of `getJson`'s return type, which is wider than this change.
      **Severity:** low; contract accuracy.
      **Fixed** in `3ec2b22`. The `spec-writer` settled the verdict in round 2:
      a 2xx body with an `error` member is a failed probe, and the refusal is
      a non-empty string `error` and nothing else. So the bar is not tested
      before `isError` (that would accept `{"apiVersion":"6.2.0","error":5}`,
      which the spec now refuses); instead the forgeable discriminator is
      removed from `probe`'s path. A private `SeedClient::fetch` returns
      `Fetched{seedAnswered, body}`, `getJson` is `fetch(path).body` for every
      other caller (its type unchanged, as you said not to widen it), and
      `probe` judges a body only when it is the seed's, building every refusal
      itself. The header now states the `error` rule and that a refusal is
      never the seed's body. Regression tests
      `a_success_carrying_an_error_member_is_a_failed_probe` (five bodies) and
      the error-member row of `a_failed_probe_leaves_nothing_from_an_earlier_answer`,
      both watched failing first (reply size 2, `reachable()` true); dropping
      the check turns exactly those two red (measured). Alternatives and why
      each was not taken are under "`probe` knows whose body it holds" in
      `design.md`.

Noted, not boxed: `save()` (`settings_store.cpp:271`) and the new pre-check in
`set()` (`:360`) both know "no settings path". The pre-check is correct, its
comment says why, and `set()` is the only caller of `save()`, so nobody needs to
act; if `save()` is touched again it could report why it failed rather than a
bool.
