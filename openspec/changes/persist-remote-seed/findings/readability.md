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

- [ ] **`dev-writer`** — `radicle/src/radicle_impl.cpp:483-485` — the refusal for an
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

- [ ] **`dev-writer`** — `radicle-ui/src/qml/Main.qml:1015-1020` — comment says
      `setRemoteSeed` "reverts to the previous one if it does not answer", which is no
      longer a description of what happens
      **Scenario:** a reader of the `onSeedChosen` handler learns that the core adopts
      the seed and then reverts it. After this change the core never switches to a
      refused seed at all (a candidate client is replaced only on success), so there is
      nothing to revert; the picker snapping back is the view's own doing at line 580.
      `tasks.md` 2.4 scoped its stale-comment sweep to `radicle/src` and `docs`, so the
      QML comment was not looked at. A comment-only edit, no behaviour change.
      **Severity:** low (comment drift, a defect not a preference).

- [ ] **`dev-writer`** — `radicle/src/settings_store.h:17-22` — the new class-comment
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

- [ ] **`dev-writer`** — `radicle/src/settings_store.h:109-119` — `validate`'s
      contract implies a cheap pure check, but for `gitPath` it runs a child process
      **Scenario:** the doc says "without storing anything" and justifies making it
      public so callers ask it before doing anything else. A reader taking that as pure
      may call `validate("gitPath", …)` on a UI thread or in a hot path; it executes
      `LocalReader::gitProbe(value)` (`settings_store.cpp:381-382`), spawning
      `<value> --version`. The only current caller passes `kKeyRemoteSeed`, so nothing
      is broken, but the next caller is invited to be surprised. One sentence in the
      doc ("may run the configured git to check it") closes it. **Severity:** low;
      a missing "why/what a reader would ask".

- [ ] **`dev-writer`** — `radicle/src/radicle_impl.cpp:465-468` — "The rollback this
      replaced" narrates the diff rather than the code
      **Scenario:** once the old code is gone from history's front page, "this
      replaced" has no referent. The reason it is worth keeping is the mechanism:
      `SeedClient::setSeedUrl` clears `reachable`, `apiVersion` and `nid`, so
      re-setting the old URL forgets that the old seed answered. That can be stated as
      a fact about `setSeedUrl` without reference to a previous revision, and is then
      still true and still useful beside the copy-then-move. **Severity:** low;
      stylistic.

- [ ] **`tester`** — `radicle/tests/test_radicle_impl.cpp` (comment above
      `a_failed_probe_stores_nothing_and_keeps_the_seed_in_force`) — "The rollback this
      milestone protects" names a milestone and a mechanism that no longer exist
      **Scenario:** the test sits under the heading's "a seed it refuses changes
      neither the seed in force nor the stored one", yet its own doc speaks of a
      rollback, which `design.md` says the change removed, and of "this milestone",
      which a reader cannot resolve. Same for `a_seed_that_answers_but_cannot_be_stored…`
      ("Before this change the seed was adopted…"). State what the test pins, not what
      it used to guard. **Severity:** low; stylistic.
