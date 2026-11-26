# Readability review — persist-remote-seed

Dimension: **readability** only (correctness, security and architecture are other instances).

Read: `git diff origin/main` in full (`radicle_impl.cpp/.h`, `settings_store.cpp/.h`,
`test_radicle_impl.cpp`, `docs/M3-embedded-node-plan.md`, the change folder), plus the
surrounding code `setRemoteSeed` and `validate` sit in, and `grep` over `radicle/src`
and `radicle-ui/src` for comments still describing the old behaviour.

Clean areas:

- `setRemoteSeed` itself reads top to bottom as shape, probe, store, adopt, and each
  comment says why (why shape before probe, why a candidate client, why store before
  adopt). One function, one job; no ambient state reached for.
- The `validate` extraction is a pure move: `set` keeps the unknown-key check and the
  write, `validate` holds only the per-key rules. The two names say what they do.
- The `radicle_impl.h` contract for `setRemoteSeed` states the order, the refusal
  guarantees and the difference from `setSetting`, which is what a caller needs.
- Test names are full sentences naming the behaviour; the `NO SPEC:` markers are
  present on exactly the tests that pin unspecified choices; the helpers
  (`freshSettingsPath`, `settingsCarryingSeed`, `kSeedIndex`) are small and
  each explains why it exists. The control assertions (`an_accepted_seed_emits_one…`,
  the `setSetting` control in the malformed-URL loop) are explained where they sit.
- `docs/M3-embedded-node-plan.md`'s annotation is rewritten to say what is true now
  and points at the capability rather than restating it.

Findings:

- [x] **`dev-writer`** — `radicle/src/radicle_impl.cpp:483-485` — the refusal for an
      unstorable seed ends mid-sentence in the one case the design says is reachable
      **Scenario:** a module with no settings path (`SettingsStore{""}`, no
      `XDG_DATA_HOME` and no `HOME`, which `design.md` names as the only way to reach
      this branch) gets a seed that answers. `SettingsStore::set` returns
      `"could not write settings to " + m_path` with `m_path` empty, so the user reads
      `the seed answered, but it could not be saved, so it was not adopted: could not
      write settings to ` — a trailing "to" with nothing after it, and nothing saying
      the cause is a missing data directory. The wrapper was written so a user "is not
      left wondering"; here they are. Not a logic defect; a message nobody can act on.
      **Severity:** low (message quality). Either the store's message should name the
      empty-path case, or the wrapper should not splice a message that can end empty.
      **Fixed** at the source, in "Refuse a seed that is not a Radicle index, and
      shape-check the stored URL": `SettingsStore::set` with an empty path now
      refuses with "settings cannot be saved: there is no settings file, because
      neither XDG_DATA_HOME nor HOME is set", which `setSetting` and the
      `setRemoteSeed` wrapper both pass on. The wrapper is unchanged. Regression test
      `a_store_with_no_path_says_why_it_cannot_write` (`test_settings_store.cpp`,
      marked `NO SPEC:`, since the spec does not say what the refusal says) was
      watched failing first: "[could not write settings to ] does not contain
      [XDG_DATA_HOME]".

- [x] **`dev-writer`** — `radicle-ui/src/qml/Main.qml:1015-1020` — comment says
      `setRemoteSeed` "reverts to the previous one if it does not answer", which is no
      longer a description of what happens
      **Scenario:** a reader of the `onSeedChosen` handler learns that the core adopts
      the seed and then reverts it. After this change the core never switches to a
      refused seed at all (a candidate client is replaced only on success), so there is
      nothing to revert; the picker snapping back is the view's own doing at line 580.
      `tasks.md` 2.4 scoped its stale-comment sweep to `radicle/src` and `docs`, so the
      QML comment was not looked at. A comment-only edit, no behaviour change.
      **Severity:** low (comment drift, a defect not a preference).
      **Fixed** in "Say what the settings store and the seed picker do, not what they
      did": the comment now says `setRemoteSeed` refuses a seed that does not answer
      or cannot be saved and keeps the one in force, and that `setSeed` reports it and
      puts the picker back. `check-qml-syntax.sh` passes. Comment only; no test can
      see it. `tasks.md` 2.4's sweep now names `radicle-ui/src` too.

- [x] **`dev-writer`** — `radicle/src/settings_store.h:17-22` — the new class-comment
      paragraph is a time-relative changelog entry, which `CLAUDE.md`'s "Keeping
      documents true" says not to write
      **Scenario:** "which for a while after this store landed adopted a seed in
      memory only" is accurate today and meaningless to a reader a year on, who cannot
      tell how long "a while" was or whether it still applies. It also opens with an
      abstract sentence ("A store is not the same as every caller writing to it")
      before reaching the fact. The durable content is one clause: both
      `setRemoteSeed` and `setSetting("remoteSeed", …)` write the `remoteSeed` key, and
      `radicle_impl.h` says how they differ. The history is already in `design.md` and
      the M3 annotation. **Severity:** low; stylistic against this repo's stated rule.
      **Fixed** in "Say what the settings store and the seed picker do, not what they
      did": the paragraph is now the durable clause only — the two module methods
      that write `remoteSeed`, and that `setRemoteSeed` in `radicle_impl.h` says how
      they differ. Comment only.

- [x] **`dev-writer`** — `radicle/src/settings_store.h:109-119` — `validate`'s
      contract implies a cheap pure check, but for `gitPath` it runs a child process
      **Scenario:** the doc says "without storing anything" and justifies making it
      public so callers ask it before doing anything else. A reader taking that as pure
      may call `validate("gitPath", …)` on a UI thread or in a hot path; it executes
      `LocalReader::gitProbe(value)` (`settings_store.cpp:381-382`), spawning
      `<value> --version`. The only current caller passes `kKeyRemoteSeed`, so nothing
      is broken, but the next caller is invited to be surprised. One sentence in the
      doc ("may run the configured git to check it") closes it. **Severity:** low;
      a missing "why/what a reader would ask".
      **Fixed** in "Say what the settings store and the seed picker do, not what they
      did": `validate`'s doc now says it is not a pure check for every key — a
      non-empty `gitPath` is validated by running `<value> --version` through
      `LocalReader::gitProbe` — and that the other keys are checked in memory.
      Comment only.

- [x] **`dev-writer`** — `radicle/src/radicle_impl.cpp:465-468` — "The rollback this
      replaced" narrates the diff rather than the code
      **Scenario:** once the old code is gone from history's front page, "this
      replaced" has no referent. The reason it is worth keeping is the mechanism:
      `SeedClient::setSeedUrl` clears `reachable`, `apiVersion` and `nid`, so
      re-setting the old URL forgets that the old seed answered. That can be stated as
      a fact about `setSeedUrl` without reference to a previous revision, and is then
      still true and still useful beside the copy-then-move. **Severity:** low;
      stylistic.
      **Fixed** in "Refuse a seed that is not a Radicle index, and shape-check the
      stored URL": the comment now states the mechanism as a fact ("Setting the old
      URL back would not be one anyway: `SeedClient::setSeedUrl` clears that state,
      so a failed pick would make a working seed read as unreachable"), with no
      reference to a previous revision. Comment only; no test can see it.

- [x] **`tester`** — `radicle/tests/test_radicle_impl.cpp` (comment above
      `a_failed_probe_stores_nothing_and_keeps_the_seed_in_force`) — "The rollback this
      milestone protects" names a milestone and a mechanism that no longer exist
      **Scenario:** the test sits under the heading's "a seed it refuses changes
      neither the seed in force nor the stored one", yet its own doc speaks of a
      rollback, which `design.md` says the change removed, and of "this milestone",
      which a reader cannot resolve. Same for `a_seed_that_answers_but_cannot_be_stored…`
      ("Before this change the seed was adopted…"). State what the test pins, not what
      it used to guard. **Severity:** low; stylistic.
      **Fixed.** The two comments the entry quotes ("The rollback this milestone
      protects", "Before this change the seed was adopted…") were already gone from
      `test_radicle_impl.cpp` at the start of this pass (rewritten in
      `e01c1e1`); `git grep -i` for `rollback`, `before this change`, `this
      replaced` and `this milestone protects` over the three test files returns
      nothing. Two comments of the same shape remained on the section 3 tests and
      are rewritten to say what each pins: `a_success_that_is_not_a_seed_index…`
      ("Each of these was once adopted for the session…") and
      `a_failure_status_is_a_failed_probe…` ("which once made the call throw…").
      Comment-only; no test can see it. Pre-existing "this milestone" and "used to"
      comments elsewhere in the file (lines predating this change) are not this
      piece's and are left.

## Round 2

Dimension: **readability** only. Reviewed `git diff 1973b69 HEAD` for the code,
comments, tests, `design.md`, `proposal.md`, `tasks.md` and the spec delta, and
re-read each changed function whole rather than as a hunk. `openspec validate
persist-remote-seed --strict` passes. Nothing here is a logic defect.

Clean areas:

- The round 1 fixes hold: the empty-path message in `SettingsStore::set` is
  accurate (`moduleDataDir` returns empty only when neither variable is set) and
  its comment says why it is a separate check; `validate`'s "not a pure check"
  paragraph, the QML `onSeedChosen` comment and the `settings_store.h` class
  paragraph read as durable statements.
- `setRemoteSeed` still reads shape, probe, store, adopt. The new loop over
  `{seedUrl, candidate.seedUrl()}` is explained by the `https://` example beside
  it, and the empty-URL refusal now says why it is not left to the shape rule.
- `SeedClient::probe` and `getJson`: the new comments say why (a foreign JSON
  API answers 200; the body is the seed's, so only a string `error` is a
  message), and the `probe` doc lists every outcome it refuses.
- Test comments: the `///` headers quote requirement and scenario names, and I
  checked each quoted name against `specs/module-settings/spec.md`; all match.
  The `NO SPEC:` markers remain only on the test whose behaviour the spec still
  does not state (`a_store_with_no_path_says_why_it_cannot_write`). The two
  store-refusal tests share setup by design and each says why it exists.
- `git grep` for rollback / revert / "for the session" wording over
  `radicle/src`, `radicle-ui/src`, `radicle/tests` and the M3 plan finds no
  comment still describing the old behaviour.
- Not raised: `proposal.md` line 79 and spec line 124 are ragged after an edit
  (cosmetic, no reader is misled), and `design.md` uses "now" and "before"
  throughout; as the decision record that is what it is for.

Findings:

- [ ] **`dev-writer`** — `radicle/src/seed_client.h:23` — the "keep this a value
      type" comment says a copy shares nothing mutable "beyond the stateless
      transport", and the transport is not stateless
      **Scenario:** `Transport` is a `std::function`, and the tests' own
      (`FakeSeed::transport()`, `test_radicle_impl.cpp:195-217`) captures `this`
      and appends to `requested`. The candidate copy and the client in force
      therefore share that mutable log, which is exactly why
      `fake.requested.back()` can name the candidate's request in the new
      tests. A maintainer reading the comment concludes a capturing transport is
      forbidden, or that it cannot leak between the two clients; neither is what
      the code needs. The true invariant is that the transport is the one thing
      a copy shares, and a transport must not make the candidate's probe
      observable through the client in force (production's default is empty and
      calls `httpGet`, which keeps no state). `design.md` repeats "the stateless
      transport" and "shares the transport" in the candidate-client section.
      **Severity:** low; a comment that is wrong on a point a reader would act
      on, a defect rather than a preference.

- [ ] **`dev-writer`** — `radicle/src/radicle_impl.cpp:437-443` — the
      `setSetting("remoteSeed", …)` branch emits `remoteSeedChanged(value)`
      verbatim beside a `setRemoteSeed` that now emits the normalised URL, with
      no comment saying the difference is known
      **Scenario:** `setSetting("remoteSeed", "https://x/")` stores
      `https://x/`, adopts `https://x` (`m_seed.setSeedUrl` strips it) and
      emits `https://x/`, so the event names a URL that is not the one in force.
      The new spec requires an accepted `setRemoteSeed` to emit the normalised
      URL, so a reader moving from the `setRemoteSeed` body (which spends a
      comment on "stored as the client normalised it ... agree") to the `setSetting`
      branch just above it will take this as an oversight and "fix" it, or take it as
      settled. `design.md` records it under Risks as a consequence of leaving
      `setSetting`'s seed path to #65, but the code, where a reader asks, does
      not. A one-line comment at the call ("stored and announced as given, unlike
      `setRemoteSeed`, which normalises; see the design's Risks") closes it.
      **Severity:** low; a missing "why" at a place the change itself made
      conspicuous.
