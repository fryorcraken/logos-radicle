# Readability review: sync-cancel

Dimension: **readability only** (naming, comments, one-function-one-job,
duplication). Correctness, security and architecture are other instances'.

Reviewed: the diff of `radicle-ui/src/qml/SourceTab.qml` against `main`, read
against the whole file; `tst_sync_cancel.qml`, `tst_sync_cancel_view.qml`;
`sync.yaml` and the `docs/e2e.md` paragraph; the comments in `design.md` that
the code points at.

No mutation run: there is no mutation tool for QML (overlay), and no Qt 6
`qmltestrunner` or `qmllint` in this pod, so nothing here was executed. These
findings are from reading.

Clean, and worth saying so: the `syncMaxInFlight` comment explains the one
non-obvious thing (the module serves in order, so a cancel cannot take back what
was handed over) and points at the design for the alternatives; `pumpSync()`'s
comment names its two legitimate callers and why a cancelled sync is never
pumped again; the "marked at dispatch, not at discovery" comment in
`fetchSyncBlob()` says what would go wrong otherwise; the `cancelSync()` comment
says plainly that `dropSyncQueue()` only lets go and the epoch is what stops the
pump. The test fakes carry their own reason for being input-dependent. The
`sync.yaml` header and step comments say what changed and why the order is the
user's. Test names read as sentences and say what they pin.

Defects are marked **defect**; the rest are **preference**.

- [ ] **`dev-writer`** — `SourceTab.qml:485` — **defect** (misleading comment).
      `walkSyncListing()`'s doc says "Counts the listing itself as done."
      It does not: it only queues the children. The listing is counted by
      `settleSyncRead()` on the reply path and by an inline `syncDone++` in
      `fetchSyncTree()`'s cached branch.
      **Scenario:** a reader who trusts the comment drops the `syncDone++` from
      the cached branch as redundant, or adds one in `walkSyncListing()`, and
      the cached listing is counted twice or not at all; `syncDone >=
      syncQueued` then fires early or never. Reword to what it does ("Queue a
      listed directory's children; the caller counts the listing").

- [ ] **`dev-writer`** — `SourceTab.qml:117`, `:503`, `:532-533` — **defect**
      (name collision). `syncInFlight` is a count of the running sync's
      outstanding reads; the existing `inFlight` is a map of blob keys being
      fetched by anyone (click, hover, sync). They sit on adjacent lines in
      `fetchSyncBlob()` (`inFlight[bkey] = true; syncInFlight++;`) and mean
      different things, and `syncMaxInFlight` is a third. **Scenario:** a
      reader fixing the next bug in this function reads `syncInFlight` as "the
      sync's slice of `inFlight`" and clears or tests one where the other is
      meant. A name that does not echo the map (for example the count of
      outstanding reads, `syncOutstanding`) removes the question.

- [ ] **`dev-writer`** — `SourceTab.qml:443-458` vs `:494`, `:520` —
      **preference**, leaning defect. `syncDir()` and `syncBlob()` now only
      *queue*; `fetchSyncTree()` and `fetchSyncBlob()` dispatch. The old names
      read as "do the sync of a directory/file" and now describe neither, and
      the pair uses "Dir" on one side and "Tree" on the other for the same
      thing (the `kind` is `"tree"`). The doc comments carry the difference, so
      it is recoverable, but it is the verb a reader scans for.
      `queueSyncTree`/`queueSyncBlob` against `fetchSyncTree`/`fetchSyncBlob`
      would make the two stages legible at the call site in `walkSyncListing()`
      and `pumpSync()`. Only `SourceTab.qml` mentions either name (grep over
      `radicle-ui/`), so the rename is local.

- [ ] **`dev-writer`** — `SourceTab.qml:546` against `:336` and `:365` —
      **preference**. `fetchSyncBlob()`'s reply is now the third copy of the
      "decode the blob, store it, paint it if the user is waiting" block
      (`openEntry()`, `prefetch()`, here), and the "is the user waiting on this
      file" test is written two ways: `tab.rid === wantRid && tab.selectedFile
      === entry.path` in the first two, `cacheKey(tab.selectedFile) === bkey`
      here. The design says the key form is deliberate (it carries rid and
      branch); no comment in the code says so, and the other two copies do not
      carry the branch term, so a reader cannot tell whether the difference is
      a decision or drift. This repo's own standing example is exactly this
      idiom hand-copied and a term dropped. Either give the three one helper
      (the decode, and the "paint if selected" step) or leave a line at the
      `cacheKey` comparison saying why it differs from `prefetch()`'s, so the
      next copy starts from the right one. Whether the helper belongs here or
      in a separate change is the architecture reviewer's call.

- [ ] **`dev-writer`** — `SourceTab.qml:119-124` — **preference**.
      `dropSyncQueue()` also sets `syncInFlight = 0`, which is the slot count,
      not the queue. The doc comment says "empty queue and every slot", so it
      is documented; but the name and the one job disagree (the tell in
      CLAUDE.md is a name that does not say what the function does), and its
      two callers that matter (`reset()`, `syncAll()`) then zero
      `syncQueued`, `syncDone`, `syncProgress` on the next three lines, so the
      sync's bookkeeping is reset half inside the function and half beside it.
      A name that covers both (`clearSyncDispatch()`) or moving the zeroing of
      the counters in with it would make "what does starting a sync reset?"
      answerable from one place.

- [ ] **`dev-writer`** — `SourceTab.qml:524-528` — **preference**. The skipped
      blob in `fetchSyncBlob()` is counted done and takes no slot; the
      matching branch in `fetchSyncTree()` says why ("A cached listing is no
      request: walk it without taking a slot") and this one does not. The
      pair `syncDone++; finishSyncIfDone();` is now written three times
      (here, `fetchSyncTree()`, `settleSyncRead()`), only the last freeing a
      slot. A one-line comment here ("not a request: counts as done, takes no
      slot") is enough; a reader otherwise has to work out why this branch does
      not call `settleSyncRead()` and why that is right.

- [ ] **`tester`** — `tst_sync_cancel.qml:22` — **preference**. The header says
      the bound is "recorded in the change's design.md". After the archive
      commit that file is at `openspec/changes/archive/<date>-sync-cancel/`,
      so the pointer stops resolving by name. The component and `docs/e2e.md`
      say "the `sync-cancel` design's Decisions", which survives the move by
      grep. Use the same wording here.

- [ ] **`tester`** — `tst_sync_cancel.qml:47`, `:67`, `:134`, `:377`;
      `tst_sync_cancel_view.qml:36`, `:64`, `:117-123` — **preference**. The
      request key `r.method + ":" + (r.args.length > 2 ? r.args[2] : "")` is
      built inline twice in each fake (`serveNext`, `serveThrough`) and the
      fake itself is copied between the two files; "is this a sync read"
      (`method === "GetTree" || "GetBlob"`) is written three times in
      `tst_sync_cancel.qml` (`isSyncRead` on a key, `queuedSyncReads`,
      `mineWithTheNode`) in two forms. A `keyOf(r)` on the fake removes the
      first; the second wants one predicate. Separately,
      `tst_sync_cancel_view.qml:117` and `:123` assert `page.syncing === true`
      twice with nothing between that can change it (only a `verify` on the
      queue), so the second line says nothing the first did not. Low value;
      take it if the file is touched again.

- [ ] **`dev-writer`** — `sync.yaml:213-214` — **preference** (wording). The
      comment calls the poll's wait "the file's default of 30 s", but the step
      right below sets `timeout: 30s` itself, so it is not the default, and
      the `tasks.md` text ("each given 30 s") says it is set. If the file's
      default (line 38) moves, the comment will claim the step follows it when
      it does not. Say "30 s" and drop "default".
