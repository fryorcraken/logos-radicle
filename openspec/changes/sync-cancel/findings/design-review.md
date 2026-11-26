# Design review: sync-cancel

Reviewed `design.md` Decisions 1 to 6 against `radicle-ui/src/qml/SourceTab.qml`
(`syncMaxInFlight`, `syncPending`, `syncInFlight`, `dropSyncQueue`, `syncAll`,
`cancelSync`, `pumpSync`, `settleSyncRead`, `fetchSyncTree`, `fetchSyncBlob`),
against issue #92 (read fresh with `gh issue view 92`) and against the
change's spec delta.

Decisions 1, 3, 4 and 5 are implemented as written, at every call site I
found: epoch check before `settleSyncRead` in all four reply callbacks, the
in-flight mark set only in `fetchSyncBlob`, the paint on key match in both the
`GetBlob` success and failure paths. The change stays inside the issue's scope
(QML only, spec first, `sync.yaml` back in the user's order) and the issue's
own open question is answered with the alternatives it named. The guard
evidence in Decisions 1, 3, 4 and 5 is specific (named tests, a row count) and
Decision 1 honestly says the `dropSyncQueue()` call in `cancelSync()` is not
pinned. I did not re-run the mutations; they are the `tester`'s claims.

What is left is one inaccuracy in Decision 6, and gaps and thin spots.

- [ ] **`dev-writer`** — `design.md` Decision 6 (and Decision 5's "at most 8") is
      contradicted by `SourceTab.qml:411`. Decision 6 says "the two `app.call`
      sites in the sync path, both counted in `syncInFlight`". `syncAll()` makes
      a third sync-path call, `ListBranches`, which is sent directly, outside
      `pumpSync()` and uncounted. It is sent first, so it cannot sit behind a
      cancelled sync's reads, but the claim "does anything bypass the bound?" now
      has an answer other than the one recorded, and after a cancel and
      restart the module can hold 4 + 4 + 1 reads, not 8. Correct Decision 6 to
      name the exception and why it is harmless (one request, the same one
      `checkForUpdate()` makes), and Decision 5's figure.
- [ ] **`dev-writer`** — `design.md` Decision 2 says 4 "is the spec's fixed
      bound". The spec delta never names 4 (grep finds no `4` or "four"); it
      requires "a fixed number" that is the same for every mode. The number is
      the design's choice, and the tests copy it as a literal. Reword so the
      reader does not look for it in the spec.
- [ ] **`dev-writer`** — a choice with a real alternative is made in code and not
      recorded: a refused read (`GetTree` or `GetBlob` error callback,
      `SourceTab.qml:514` and `:552`) frees its slot and counts as done, so the
      sync carries on and can finish as synced past a failed read. The
      alternatives are to stop the sync, or to retry. That is existing
      behaviour, but this change routes it through `settleSyncRead()` and so
      decides it for the queue too (a failed read must still pump, or a single
      error would wedge the window). One line in Decision 5 or 6 saying so, and
      whether a test pins "an error frees the slot".
- [ ] **`dev-writer`** — Decision 2's case against 1 states as fact that against
      the Radicle home "the round trip is the larger part of each read". No
      number is behind it, unlike the throughput caveat two paragraphs later,
      which is flagged as reasoning. Mark it the same way, or measure one local
      read. It is the only argument for 4 over 1 on the `local*` path.
- [ ] **`dev-writer`** — the hazard "another view that sends many calls at once
      reintroduces the backlog for every later call" is recorded in
      `design.md` Risks and `docs/e2e.md:83-89`, which the `CLAUDE.md` routing
      table sends a reader to only when running or editing a sitometres spec.
      Someone adding a view that fans out calls will not read it. It belongs
      where the next toucher of the call path looks: a sentence at `app.call()`
      in `Main.qml` or in the overlay's `## Hazards` (the owner's file, so
      propose it to the owner rather than edit it), not only in an archived
      `design.md`.
