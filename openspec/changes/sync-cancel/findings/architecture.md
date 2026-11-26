# Architecture review: sync-cancel

Dimension: architecture only (correctness, security and readability are other
instances). Reviewed against `CLAUDE.md`'s "How to shape a change" and the
overlay's `## Hazards` entry on the hand-written staleness guard.

Reviewed: `radicle-ui/src/qml/SourceTab.qml` (the whole sync path, including
`reset()`, `cancelSync()`, `syncAll()`, `finishSyncIfDone()` and the sibling
`openEntry()`/`prefetch()`), `design.md`, `tasks.md`, the `docs/e2e.md` change
and both new test files. `tst_sync_cancel.qml` runs here under
`/usr/lib64/qt6/bin/qmltestrunner`: 18 passed, 0 failed.

Clean areas:

- **The shape of the fix is right.** Enforcing the bound where the view
  dispatches, and leaving `radicle_impl.h` and the core module alone, keeps the
  interface narrow. Decision 1's three ruled-out alternatives hold up. The core
  cancel would arrive behind the backlog on the same ordered channel.
- **The reshape is the right one for the problem.** Splitting the old
  `syncDir()`/`syncBlob()` into queue (`syncDir`/`syncBlob`), dispatch
  (`pumpSync`), fetch (`fetchSyncTree`/`fetchSyncBlob`) and settle
  (`settleSyncRead`) gives each one job.
- **Decision 6 holds.** `GetTree`/`GetBlob` for a sync are sent only from
  `fetchSyncTree()` and `fetchSyncBlob()`, and only `pumpSync()` calls those.
  `loadTree`, `openEntry` and `prefetch` are user-driven and rightly outside the
  bound.
- **The dispatch bookkeeping is a set of properties, not a new type.** That
  matches the repo's habit and is proportionate; no new abstraction was added.
- **`docs/e2e.md`'s replacement paragraph** is true of the code and names the
  hazard it leaves, in the trigger doc where the next person will look.
- **No CI gate moved.** The two new test files match `run-qml-tests.sh`'s
  `tst_*.qml` glob, and no file was renamed.
- **No new dependency.**

Findings, all minor to moderate; none is a defect in behaviour I could break.

- [x] **`dev-writer`** — `SourceTab.qml:478-483, 513, 516, 551, 556` and `pumpSync` at `:467` — the epoch guard that Decision 5 rests on is hand-written at four reply sites, and `pumpSync(epoch)` does not check the epoch it is given
      **Scenario:** `settleSyncRead()` frees a slot, counts, and pumps, and its comment says "Only for a reply under the running sync's epoch — the caller checks." The four callers each write `if (epoch !== syncEpoch) return;` before it (`GetTree` ok and fail, `GetBlob` ok and fail). A fifth caller, or one of these four edited from memory, that drops the line frees a slot of a cancelled sync inside the next sync and pumps a cancelled sync's queue; nothing in `pumpSync` stops it. This is the overlay's hazard (a guard hand-written across sites, dropping a term) one level down. The design records that the mistake needs a test per reply path (the two-row late-reply test exists because the `GetBlob` path was once unexercised). **Suggested shape:** move the check into `settleSyncRead(epoch)` (`if (epoch !== syncEpoch) return;` first), and have `pumpSync` ignore a stale epoch the same way. The invariant then holds by construction at one site. The `GetBlob` paint must stay ahead of the check, which it does in the current order. Genuine design weakness, not taste, but the current code is correct and tested.
      **Measured:** not mutated; the design's own guard note says the check removed from either reply path turns the late-reply row red.
      **Fixed** in the commit "Sync: one epoch check, one completion, one blob decode, clearer names", in your suggested shape: `settleSyncRead()` and `pumpSync()` each return at once for a stale epoch (now a stale `run`, after the security fix), and three of the four reply callbacks call `settleSyncRead()` bare. The `GetTree` success callback keeps its own check, ahead of walking the listing, so a stale listing is not walked; the `GetBlob` paint stays ahead of `settleSyncRead()`. Measured by mutation (removing the check from `settleSyncRead()`): the "files outstanding" row of `test_a_late_reply_does_not_free_a_slot_in_the_next_sync` goes red (8 reads at once); the "listings outstanding" row stays green because of the `GetTree` callback's own check. Design Decision 5 records both.

- [x] **`dev-writer`** — `SourceTab.qml:478-483, 499-500, 525-526` — "a read is finished" is written three times: `syncDone++; finishSyncIfDone()` in the cached-listing path of `fetchSyncTree()`, in the already-cached-or-fetching path of `fetchSyncBlob()`, and inside `settleSyncRead()`
      **Scenario:** these differ only in whether a slot is freed, so `settleSyncRead()` is "free a slot, then complete" and the other two are "complete". A fourth completion path (a skipped read, say) is another copy to get right, and the next edit that adds to what completing means (a progress step, an event) has to find all three. **Suggested shape:** one `completeSyncRead()` holding `syncDone++; finishSyncIfDone()`, called by the two slotless paths and by `settleSyncRead()`. Weak finding: three lines, and the repo says not to refactor speculatively. It is raised because this change added two of the three copies.
      **Fixed** in the commit "Sync: one epoch check, one completion, one blob decode, clearer names": `completeSyncRead()` holds `syncDone++; finishSyncIfDone()`, called by the cached-listing path, the skipped-blob path and `settleSyncRead()`. No behaviour change; every sync test file passes unchanged.

- [x] **`dev-writer`** — `SourceTab.qml:119-124` — `dropSyncQueue()` resets two things, the queue and `syncInFlight`, and its name says one
      **Scenario:** a reader of `cancelSync()` sees "drop the queue" and has to read the body to learn that the dispatch window is also reset to empty. That reset is the line Decision 5 depends on (a late reply from a cancelled sync must not free a slot, because the counter was zeroed). The name hides the part the design leans on. **Suggested shape:** rename to say both (`resetSyncDispatch`), or inline the two assignments at the three call sites. Style, not a defect.
      **Fixed** in the same commit: renamed `resetSyncDispatch()`, and its doc comment says it empties both the queue and the outstanding count, and why the progress counters are not part of it. The same finding is readability's fifth entry.

- [x] **`dev-writer`** — `SourceTab.qml:485` — the comment above `walkSyncListing()` says it "Counts the listing itself as done", and it does not
      **Scenario:** the function only queues children. The counting is done by its two callers (`syncDone++` in the cached path, `settleSyncRead()` in the reply path). A reader trusting the comment will either add a second count or leave one out when adding a caller. The comment is the kind the repo's rule on comments calls noise at best, and here it is wrong. Fix the comment. Defect in prose only.
      **Fixed** in the same commit; the same finding as readability's first entry, whose outcome has the wording.

- [x] **`dev-writer`** — `SourceTab.qml:538-546` vs `:327-345` and `:355-369` — `fetchSyncBlob()` is the third hand-written copy of "decode a `GetBlob` reply, cache it, paint it if the user is waiting on it"
      **Scenario:** the binary placeholder string `"(binary file — " + (data.name || entry.name) + ")"` now sits in three places (`openEntry`, `prefetch`, `fetchSyncBlob`), as do three subtly different paint guards: `rid && selectedFile === entry.path` in the first two, `cacheKey(selectedFile) === bkey` here. The new one is the better guard (it carries rid and branch), but it was added beside the other two rather than shared. This is the same drift issue #29 describes, and the first fix of the "sync did not paint" defect (design Decision 4) is itself a case of one copy lacking what another had. **Suggested shape:** a `blobBody(data, entry)` helper at minimum; the paint guards differ on purpose, so do not unify them without reading issue #29. Not blocking; deferring to a follow-up issue is reasonable.
      **Fixed** (the minimum you named) in the commit "Sync: one epoch check, one completion, one blob decode, clearer names": `blobBody(data, entry)` is the one decode, used by `openEntry()`, `prefetch()` and `fetchSyncBlob()`. The paint guards stay different, as you advised; the sync's carries a comment saying why, and design Decision 4 points at issue #29 for why they are not unified.
