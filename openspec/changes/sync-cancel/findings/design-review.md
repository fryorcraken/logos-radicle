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

- [x] **`dev-writer`** — `design.md` Decision 6 (and Decision 5's "at most 8") is
      contradicted by `SourceTab.qml:411`. Decision 6 says "the two `app.call`
      sites in the sync path, both counted in `syncInFlight`". `syncAll()` makes
      a third sync-path call, `ListBranches`, which is sent directly, outside
      `pumpSync()` and uncounted. It is sent first, so it cannot sit behind a
      cancelled sync's reads, but the claim "does anything bypass the bound?" now
      has an answer other than the one recorded, and after a cancel and
      restart the module can hold 4 + 4 + 1 reads, not 8. Correct Decision 6 to
      name the exception and why it is harmless (one request, the same one
      `checkForUpdate()` makes), and Decision 5's figure.
      **Fixed** in the commit "sync-cancel: design and tasks after review;
      findings answered": Decision 6 is now about a sync's *tree and file*
      reads, and names `ListBranches` as the one exception with why it is
      harmless (not one of the spec's reads, sent before any of them, the same
      single request `checkForUpdate()` makes). Decision 5's figure is now 4
      + 4 + 1, a request made then waiting behind at most 9 calls.
- [x] **`dev-writer`** — `design.md` Decision 2 says 4 "is the spec's fixed
      bound". The spec delta never names 4 (grep finds no `4` or "four"); it
      requires "a fixed number" that is the same for every mode. The number is
      the design's choice, and the tests copy it as a literal. Reword so the
      reader does not look for it in the spec.
      **Fixed**, partly by the spec: the `spec-writer` has since put the
      number in `repo-sync` (at most 4 outstanding while running, at most 4
      reaching the seed after a cancel), so "the spec's bound" is now true.
      Decision 2 is retitled "Why the spec's bound is 4" and says the spec
      states it and the design records why 4 was the number put to it, in
      the same commit as above.
- [x] **`dev-writer`** — a choice with a real alternative is made in code and not
      recorded: a refused read (`GetTree` or `GetBlob` error callback,
      `SourceTab.qml:514` and `:552`) frees its slot and counts as done, so the
      sync carries on and can finish as synced past a failed read. The
      alternatives are to stop the sync, or to retry. That is existing
      behaviour, but this change routes it through `settleSyncRead()` and so
      decides it for the queue too (a failed read must still pump, or a single
      error would wedge the window). One line in Decision 5 or 6 saying so, and
      whether a test pins "an error frees the slot".
      **Fixed**: Decision 5 now has a paragraph on it (a refusal settles as an
      answer; stopping or retrying were the alternatives and would change
      what a sync promises, which the spec does not state; what the queue
      adds is that a refusal must free its slot). A test now pins it:
      `tst_sync_walk.qml`'s `test_a_refused_read_does_not_stall_the_sync`,
      marked `NO SPEC:`, which goes red when `settleSyncRead()` is removed
      from `fetchSyncBlob()`'s failure callback (the read after the four
      refusals is never made). Test in the commit "Sync: list each directory
      once per sync, and keep every read on its branch"; prose in the
      design commit. The behaviour itself is a spec question and is reported
      to the runner for the `spec-writer`.
- [x] **`dev-writer`** — Decision 2's case against 1 states as fact that against
      the Radicle home "the round trip is the larger part of each read". No
      number is behind it, unlike the throughput caveat two paragraphs later,
      which is flagged as reasoning. Mark it the same way, or measure one local
      read. It is the only argument for 4 over 1 on the `local*` path.
      **Fixed** by marking it, in the design commit: the spec now makes the
      sync the seed's ("The sync control is the seed's"), so there is no
      `local*` path for the argument to be about. Decision 2's case against 1
      is restated for the seed, labelled "reasoning, not a measurement", and
      says the earlier local-read argument no longer applies and why.
- [x] **`dev-writer`** — the hazard "another view that sends many calls at once
      reintroduces the backlog for every later call" is recorded in
      `design.md` Risks and `docs/e2e.md:83-89`, which the `CLAUDE.md` routing
      table sends a reader to only when running or editing a sitometres spec.
      Someone adding a view that fans out calls will not read it. It belongs
      where the next toucher of the call path looks: a sentence at `app.call()`
      in `Main.qml` or in the overlay's `## Hazards` (the owner's file, so
      propose it to the owner rather than edit it), not only in an archived
      `design.md`.
      **Fixed** at the first of the places you named, in the commit "Pin the
      running sync's cap and the seed-only sync control": `Main.call()`'s doc
      comment now says the module serves calls one at a time in arrival
      order, so a view that hands over many at once makes every later call
      wait for all of them, and to bound a fan-out where it is dispatched, as
      `SourceTab.syncMaxOutstanding` does. The overlay line is proposed to
      the owner in the hand-back, not written: `.claude/` is theirs.
