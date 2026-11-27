import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "Radicle.js" as R
import "Theme.js" as Theme

/*
 * File tree on the left, README or the selected file on the right.
 *
 * Two things keep this from jumping the way it used to:
 *  - The sidebar is a FIXED width and rows are a FIXED height, so the tree
 *    does not resize itself as names or directory depth change.
 *  - Blob contents are cached and directories are prefetched on hover, so
 *    clicking a file usually paints from cache with no intermediate
 *    empty-then-fill flash.
 */
Item {
    id: tab

    // A StackLayout child: must fill, or it is sized 0x0.
    Layout.fillWidth: true
    Layout.fillHeight: true

    property var app: null
    property string rid: ""
    property string branch: ""

    /// Current directory, "" for the repository root.
    property string path: ""
    property string selectedFile: ""

    /// Caches are keyed by rid + branch + path, never by path alone: a bare
    /// path collides across repositories, which served one repo's README as
    /// another's. Clearing on repo change is not sufficient by itself, because
    /// a reply already in flight lands after the clear.
    property var blobCache: ({})
    property var treeCache: ({})

    /// Keys currently being fetched, so a click during a hover-prefetch does
    /// not start a second identical request.
    property var inFlight: ({})

    function cacheKey(path) {
        return cacheKeyIn(rid, branch, path);
    }

    /// cacheKey() for a given repository and branch rather than the tab's
    /// current ones. A sync keys what it fetches by where it started.
    function cacheKeyIn(r, b, path) {
        return r + "\u0000" + b + "\u0000" + path;
    }

    /// What the viewer shows, and the cache holds, for a GetBlob reply. One
    /// copy for the three readers of a blob (openEntry(), prefetch() and the
    /// sync), which used to write it out each.
    function blobBody(data, entry) {
        return data.binary
            ? "(binary file — " + (data.name || entry.name) + ")"
            : (data.content || "");
    }

    ListModel { id: entries }

    /// Entries in the current directory — read by the UI tests.
    readonly property int entryCount: entries.count

    /// The current directory's entry names, comma-joined.
    ///
    /// Exists for the end-to-end specs, and specifically so one can assert
    /// WHICH branch's tree is on screen rather than merely that a tree
    /// arrived. That distinction is the whole of the resolution bug: an
    /// unresolvable ref falls back to the repo head and returns a populated
    /// tree with no error, so `entryCount > 0` is satisfied just as well by
    /// the wrong branch as the right one. A spec that names a file unique to
    /// the branch it picked can tell them apart; one that counts cannot.
    readonly property string entryNames: {
        var out = [];
        for (var i = 0; i < entries.count; i++)
            out.push(entries.get(i).name);
        return out.join(",");
    }

    /// The viewer pane's state, surfaced so RepoView (and through it the
    /// end-to-end specs) can assert on what the right-hand pane is showing
    /// without reaching into a nested id. Aliases, not a second copy: the
    /// viewer remains the only thing that sets them.
    readonly property string viewerTitle: viewer.title
    /// The length, not the text — a spec asks whether content arrived, and a
    /// whole blob pasted into a test report is noise.
    readonly property int viewerBodyLength: viewer.body.length

    /// Sync: walk the whole tree up front and pull every file into the cache,
    /// so navigation afterwards is instant rather than a request per click.
    property bool syncing: false
    /// Bumped whenever a sync starts, is cancelled, or the repository
    /// changes. Every in-flight request carries the epoch it began under and
    /// ignores its own reply once that epoch has moved on.
    property int syncEpoch: 0
    property int syncQueued: 0
    property int syncDone: 0
    // syncQueued grows as the tree is discovered breadth-first, so the raw
    // ratio can drop after a reply whose own children turn out to add more
    // to the denominator than they immediately finish — reported live as
    // the progress bar jumping backwards mid-sync. syncProgress is a ceiling
    // over the raw ratio, updated explicitly (see bumpSyncProgress()) rather
    // than computed inline, so the displayed value only ever advances.
    property real syncProgress: 0

    function bumpSyncProgress() {
        var raw = syncQueued > 0 ? syncDone / syncQueued : 0;
        if (raw > syncProgress) syncProgress = raw;
    }

    /// The most of one sync's tree and file reads that are outstanding (with
    /// the module, not yet answered) at once. The module answers one call at a
    /// time, in the order it received them, so whatever a sync has already
    /// handed over is served before anything asked after it — a cancel cannot
    /// take it back. Holding the rest here, and handing over the next read
    /// only as one is answered, is what lets a cancel stop the work rather
    /// than just ignore its replies: after a cancel at most this many reads
    /// reach the seed, and a request made after it waits behind at most this
    /// many. The value is the `repo-sync` spec's; why it is 4, and the
    /// alternatives, are the `sync-cancel` design's Decisions.
    readonly property int syncMaxOutstanding: 4
    /// The running sync's reads discovered but not yet handed to the module,
    /// oldest first, which keeps the walk breadth-first. Each is
    /// `{ kind: "tree", path }` or `{ kind: "blob", entry }`. Already counted
    /// in syncQueued.
    property var syncPending: []
    /// How many of the running sync's reads are outstanding: handed to the
    /// module, not yet answered or refused. A count, and nothing to do with
    /// the `inFlight` map above, which holds the blob keys any request (click,
    /// hover or sync) is fetching. Only settleSyncRead() decrements it, and
    /// only under the running sync's epoch, so a late reply from a cancelled
    /// sync cannot free a slot in the next one.
    property int syncOutstanding: 0

    /// Empty the sync's dispatch window: the queue of reads not yet handed
    /// over, and the count of those outstanding. Called wherever the epoch
    /// moves on, so the next sync starts with an empty queue and every slot
    /// free. The progress counters are not part of the window: a cancel
    /// leaves them, and only syncAll() and reset() zero them.
    function resetSyncDispatch() {
        syncPending = [];
        syncOutstanding = 0;
    }
    /// True once a sync has completed for this repository, so the idle button
    /// label can read "first download" vs "refresh" differently.
    property bool syncedOnce: false

    /// The branch's head commit at the moment the last sync completed, so a
    /// later poll (see checkForUpdate() / pollTimer below) has something to
    /// compare a fresh head against. "" until a sync has completed.
    property string lastSyncedCommit: ""
    /// True once a poll finds the branch's head has moved past
    /// lastSyncedCommit. Deliberately does NOT disable or relabel the sync
    /// button — re-syncing is never wrong, just sometimes unnecessary — it
    /// only flags that one is worth doing. Cleared by any syncAll() the same
    /// tick it starts, since a fresh sync makes the flag's answer moot until
    /// it completes and repolls.
    property bool updateAvailable: false
    /// Bumped on reset(), mirroring syncEpoch's role for sync requests: a
    /// poll reply that lands after this tab has been reset must not flip
    /// updateAvailable for data nobody is looking at anymore. reset() is the
    /// only bump site — there is no separate "cancel a poll" path, since a
    /// poll is one cheap request with nothing to unwind.
    property int pollEpoch: 0

    /// True while a directory listing is in flight.
    property bool treeLoading: false
    /// True once a listing has completed at least once for this repo.
    property bool treeLoaded: false

    /// Drop everything from the previous repository without fetching.
    /// Called when the repo changes, so no stale tree or file is on screen
    /// while the new repo's data is in flight.
    function reset() {
        path = "";
        selectedFile = "";
        blobCache = ({});
        treeCache = ({});
        inFlight = ({});
        entries.clear();
        // Orphan any sync still running for the previous repository, so its
        // replies cannot corrupt the counters of a sync started here.
        syncEpoch++;
        syncing = false;
        resetSyncDispatch();
        syncQueued = 0;
        syncDone = 0;
        syncProgress = 0;
        syncedOnce = false;
        treeLoading = false;
        treeLoaded = false;
        viewer.title = "";
        viewer.body = "";
        viewer.loading = false;
        // Orphan any poll in flight for the previous repository/branch, the
        // same way syncEpoch orphans a running sync's requests above — a
        // reply landing after this reset must not flip updateAvailable for
        // data nobody is looking at anymore.
        pollEpoch++;
        lastSyncedCommit = "";
        updateAvailable = false;
        pendingSyncHead = "";
    }

    function load() {
        // Clear everything from the previous repository BEFORE fetching. The
        // caches are keyed by path only, so carrying them across repos would
        // serve one repo's file under another's name; and leaving the old
        // tree and README on screen made a new repo look like it had the
        // previous one's contents until the replies landed.
        path = "";
        selectedFile = "";
        blobCache = ({});
        treeCache = ({});
        inFlight = ({});
        entries.clear();
        treeLoading = true;
        treeLoaded = false;
        viewer.title = "";
        viewer.body = "";
        viewer.loading = true;

        loadTree("");
        loadReadme();
    }

    function applyEntries(list) {
        entries.clear();
        // Directories first, then files; each alphabetical.
        var sorted = list.slice().sort(function (a, b) {
            if (a.kind !== b.kind) return a.kind === "tree" ? -1 : 1;
            return a.name.localeCompare(b.name);
        });
        for (var i = 0; i < sorted.length; i++)
            entries.append(sorted[i]);
    }

    function loadTree(p) {
        if (!app || rid === "") return;

        var tkey = cacheKey(p);
        if (treeCache[tkey] !== undefined) {
            tab.path = p;
            applyEntries(treeCache[tkey]);
            tab.treeLoading = false;
            tab.treeLoaded = true;
            return;
        }
        tab.treeLoading = true;
        var wantRid = rid;
        // Unlike loadReadme()/openEntry() below (and every branch-scoped
        // loader elsewhere: CommitsTab, etc.), this used to capture only
        // wantRid, not wantBranch. RepoView.onBranchChanged's reset() clears
        // treeCache before the new branch's request goes out, which stops a
        // stale reply from being served FROM the cache — but it does nothing
        // about a stale reply already in flight when the branch changes: the
        // rid is unchanged, so the old guard let it through and repainted the
        // just-loaded new branch's tree with the old branch's entries.
        // Confirmed reachable with a deferred-reply fake before adding
        // wantBranch here (see tst_branch_switch.qml's
        // SourceTabLoadTreeBranchGuard suite).
        var wantBranch = branch;
        app.call("GetTree", [rid, branch, p], function (data) {
            var list = data.entries || [];
            treeCache[tkey] = list;
            // Drop a reply that arrived after the user moved to another repo
            // or branch.
            if (tab.rid !== wantRid || tab.branch !== wantBranch) return;
            tab.treeLoading = false;
            tab.treeLoaded = true;
            tab.path = p;
            applyEntries(list);
        }, function () {
            if (tab.rid !== wantRid || tab.branch !== wantBranch) return;
            tab.treeLoading = false;
            tab.treeLoaded = true;
        });
    }

    function loadReadme() {
        if (!app || rid === "") return;
        viewer.loading = true;
        var wantRid = rid;
        app.call("GetReadme", [rid, branch], function (data) {
            var path = data.path || "README";
            var body = data.content || "";

            // Cache it under the key GetBlob would use, so clicking the README
            // in the tree is a hit rather than a second fetch. Worth doing even
            // when we do not display it.
            if (data.path && tab.rid === wantRid)
                blobCache[cacheKey(data.path)] = body;

            // Only paint if this reply is still what the user is looking at.
            // Two ways it might not be:
            //   - they moved to another repository, or
            //   - they clicked a file while the README was still in flight.
            // The second is the one that bit: the README landed afterwards and
            // relabelled the pane, leaving the clicked file's contents under
            // "README.md".
            if (tab.rid !== wantRid || tab.selectedFile !== "") return;

            viewer.loading = false;
            viewer.title = path;
            viewer.body = body;
        }, function () {
            if (tab.rid !== wantRid || tab.selectedFile !== "") return;
            // Plenty of repos have no README; not an error worth showing.
            viewer.loading = false;
            viewer.title = "";
            viewer.body = "";
        });
    }

    function openEntry(entry) {
        if (entry.kind === "tree") {
            loadTree(entry.path);
            return;
        }

        tab.selectedFile = entry.path;

        // Retitle immediately so the header names the file being opened, not
        // the one still on screen.
        viewer.title = entry.path;

        var bkey = cacheKey(entry.path);
        var cached = blobCache[bkey];
        if (cached !== undefined) {
            viewer.loading = false;
            viewer.body = cached;
            return;
        }

        // Clear the previous file's text while the new one is in flight.
        // Leaving it up made a slow load look like the wrong file had opened.
        viewer.body = "";
        viewer.loading = true;

        // A hover-prefetch for this same file may already be running; let it
        // finish and paint rather than issuing a duplicate request.
        if (inFlight[bkey]) return;
        inFlight[bkey] = true;

        var wantRid = rid;
        app.call("GetBlob", [rid, branch, entry.path], function (data) {
            var body = blobBody(data, entry);
            blobCache[bkey] = body;
            delete inFlight[bkey];
            // Only paint if this is still the file the user wants, in the repo
            // they are still looking at; a slow reply for an earlier click
            // must not overwrite a later one.
            if (tab.rid === wantRid && tab.selectedFile === entry.path) {
                viewer.loading = false;
                viewer.body = body;
            }
        }, function () {
            delete inFlight[bkey];
            if (tab.rid === wantRid && tab.selectedFile === entry.path)
                viewer.loading = false;
        });
    }

    /// Warm the cache for a row the pointer is resting on, so the click that
    /// usually follows paints immediately.
    function prefetch(entry) {
        if (!app || rid === "" || entry.kind === "tree") return;
        var pkey = cacheKey(entry.path);
        if (blobCache[pkey] !== undefined || inFlight[pkey]) return;
        inFlight[pkey] = true;
        var wantRid = rid;
        app.call("GetBlob", [rid, branch, entry.path], function (data) {
            var body = blobBody(data, entry);
            blobCache[pkey] = body;
            delete inFlight[pkey];
            // If the user clicked this file while the prefetch was in flight,
            // the click found no cache entry and started waiting. Nothing else
            // will paint it, so this reply must — otherwise the pane sits on
            // "Loading…" until the file is clicked a second time.
            if (tab.rid === wantRid && tab.selectedFile === entry.path) {
                viewer.loading = false;
                viewer.body = body;
            }
        }, function () { delete inFlight[pkey]; });
    }

    /// Fetch every directory and file in the repository into the local cache.
    /// Directories are walked breadth-first; each blob is fetched once and
    /// stored under the same key an on-demand click would use, so a synced
    /// repo needs no further requests to browse.
    function syncAll() {
        if (!app || rid === "" || syncing) return;
        syncEpoch++;
        syncing = true;
        resetSyncDispatch();
        syncQueued = 0;
        syncDone = 0;
        syncProgress = 0;
        // Whatever prompted this sync, it is about to catch up to the
        // branch's current head — the stale flag from a previous poll no
        // longer means anything the moment fresh data starts arriving.
        updateAvailable = false;

        // Capture the branch's head SHA now, under this sync's own epoch, so
        // finishSyncIfDone() can record it as lastSyncedCommit once the sync
        // actually completes. ListBranches is the same lightweight call
        // checkForUpdate() polls with — one request either way, no dedicated
        // "get branch head" endpoint needed.
        // Clear the previous sync's captured head before asking for this
        // one's. Without this, a sync whose ListBranches FAILS leaves
        // pendingSyncHead holding the head an EARLIER sync captured, and
        // finishSyncIfDone() below then records that stale value as this
        // sync's lastSyncedCommit — claiming the repo is synced to a commit
        // this sync never fetched. reset() cleared it, but reset() only runs
        // on a repo/branch change, not between two syncs of the same repo.
        pendingSyncHead = "";

        // wantBranch is captured, not read live in the callback. The epoch
        // alone is not enough: it only moves in reset()/cancelSync(), so a
        // caller that changes `branch` without either — which is exactly what
        // assigning the property directly does — would leave the loop below
        // matching the NEW branch's name and storing ITS head as the head of
        // a sync that downloaded the OLD branch's files.
        var epoch = syncEpoch;
        var wantRid = rid, wantBranch = branch;
        app.call("ListBranches", [rid], function (data) {
            if (epoch !== syncEpoch) return;   // sync already abandoned
            if (tab.rid !== wantRid || tab.branch !== wantBranch) return;
            var items = (data && data.items) ? data.items : [];
            for (var i = 0; i < items.length; i++) {
                if (items[i].name === wantBranch) { pendingSyncHead = items[i].head || ""; break; }
            }
        }, function () { /* head unknown; pendingSyncHead stays "" */ });

        // Everything this sync's reads depend on, fixed now and handed to
        // every step rather than read live: the epoch that says whether it is
        // still running, the repository and branch each read is sent for and
        // cached under, and the directories it has already queued. Production
        // resets the tab on any repository or branch change, which moves the
        // epoch; these hold even for a caller that assigns `branch` directly,
        // which used to send the rest of the queue to the new branch.
        var run = { epoch: epoch, rid: wantRid, branch: wantBranch,
                    listed: Object.create(null) };
        queueSyncTree("", run);
        pumpSync(run);
    }

    /// Set by syncAll()'s ListBranches lookup, consumed by
    /// finishSyncIfDone(). A plain property rather than a local closure
    /// variable so it survives being read from a different call stack (the
    /// ListBranches reply and the last GetTree/GetBlob reply usually land in
    /// either order).
    property string pendingSyncHead: ""

    function cancelSync() {
        // Bumping the epoch orphans every request already in flight, so their
        // replies cannot touch the counters of a later sync — nor dispatch
        // more of this one: only a reply under the running epoch pumps the
        // queue (see pumpSync()). So at most syncMaxOutstanding of this
        // sync's reads reach the seed after this line, and emptying the
        // window lets go of the rest.
        syncEpoch++;
        syncing = false;
        resetSyncDispatch();
    }

    /// Queue a directory listing for the running sync, once per sync. The
    /// seed is untrusted: one that lists a directory inside itself, or two
    /// that list each other, would otherwise have every cached listing
    /// re-queue the next, and pumpSync() walk them for ever without yielding
    /// to the event loop, so the Cancel click could never be delivered.
    function queueSyncTree(dirPath, run) {
        if (!syncing || run.epoch !== syncEpoch) return;
        if (run.listed[dirPath]) return;
        run.listed[dirPath] = true;
        syncQueued++;
        syncPending.push({ kind: "tree", path: dirPath });
    }

    /// Queue a file read for the running sync, unless the file is already
    /// cached or already being fetched.
    function queueSyncBlob(entry, run) {
        if (!syncing || run.epoch !== syncEpoch) return;
        var bkey = cacheKeyIn(run.rid, run.branch, entry.path);
        if (blobCache[bkey] !== undefined || inFlight[bkey]) return;
        syncQueued++;
        syncPending.push({ kind: "blob", entry: entry });
    }

    /// Hand the running sync's queued reads to the module until
    /// syncMaxOutstanding of them are outstanding. The only place a sync's
    /// GetTree or GetBlob is dispatched, so the bound has one place to hold.
    ///
    /// Does nothing for a run that is no longer the running sync, so a
    /// cancelled sync is never pumped again, and what it still held is never
    /// sent.
    function pumpSync(run) {
        if (run.epoch !== syncEpoch) return;
        while (syncOutstanding < syncMaxOutstanding && syncPending.length > 0) {
            var job = syncPending.shift();
            if (job.kind === "tree") fetchSyncTree(job.path, run);
            else                     fetchSyncBlob(job.entry, run);
        }
    }

    /// One of the running sync's reads is finished: count it, and end the
    /// sync if it was the last. Frees no slot; a read that took one finishes
    /// through settleSyncRead().
    function completeSyncRead() {
        syncDone++;
        finishSyncIfDone();
    }

    /// A read the sync handed to the module has been answered or refused.
    /// Under the running sync's epoch: free its slot, complete it, and hand
    /// over the next. Under an earlier epoch, nothing: that sync's slots were
    /// given back when the epoch moved (resetSyncDispatch()), so freeing one
    /// again would let the next sync put a fifth read out. The check is here,
    /// once, rather than at each reply callback.
    ///
    /// A refusal settles exactly as an answer does. It must free its slot, or
    /// one refused read would wedge the window; and it counts as done, so the
    /// sync carries on past it rather than stopping or retrying.
    function settleSyncRead(run) {
        if (run.epoch !== syncEpoch) return;
        syncOutstanding--;
        completeSyncRead();
        pumpSync(run);
    }

    /// Queue a listed directory's children. The listing itself is counted by
    /// the caller: settleSyncRead() for a reply, completeSyncRead() for a
    /// cached listing.
    function walkSyncListing(list, run) {
        for (var i = 0; i < list.length; i++) {
            var e = list[i];
            if (e.kind === "tree") queueSyncTree(e.path, run);
            else                   queueSyncBlob(e, run);
        }
    }

    function fetchSyncTree(dirPath, run) {
        var tkey = cacheKeyIn(run.rid, run.branch, dirPath);
        if (treeCache[tkey] !== undefined) {
            // A cached listing is no request: walk it without taking a slot.
            walkSyncListing(treeCache[tkey], run);
            completeSyncRead();
            return;
        }
        syncOutstanding++;
        app.call("GetTree", [run.rid, run.branch, dirPath], function (data) {
            var list = data.entries || [];
            treeCache[tkey] = list;
            // Count only while this reply still belongs to the running sync.
            // Counting unconditionally let a previous repository's replies
            // push syncDone past syncQueued, so a fresh sync reported itself
            // finished while it was still fetching.
            if (run.epoch !== syncEpoch) return;
            walkSyncListing(list, run);
            settleSyncRead(run);
        }, function () { settleSyncRead(run); });
    }

    function fetchSyncBlob(entry, run) {
        var bkey = cacheKeyIn(run.rid, run.branch, entry.path);
        // Checked again here, not only when queued: a click or a hover may
        // have fetched the file while it waited its turn. Either way this
        // read sends nothing, so it completes without taking a slot.
        if (blobCache[bkey] !== undefined || inFlight[bkey]) {
            completeSyncRead();
            return;
        }
        // Marked only now, at dispatch. Marked when queued, a read the cancel
        // then dropped would stay "in flight" for ever, and openEntry() would
        // wait on a reply that is never coming.
        inFlight[bkey] = true;
        syncOutstanding++;
        app.call("GetBlob", [run.rid, run.branch, entry.path], function (data) {
            delete inFlight[bkey];
            // The cache key carries the rid and branch the read was sent for,
            // so storing a late reply is harmless and saves a refetch; only
            // the counters are epoch sensitive.
            var body = blobBody(data, entry);
            blobCache[bkey] = body;
            // openEntry() leaves a file already in flight to the request that
            // is fetching it, so this reply must paint it if the user is
            // waiting on it — cancelled sync or not, the file is the user's.
            // Compared by key, not by rid and path as openEntry() and
            // prefetch() compare: this read was sent for the branch the sync
            // started on, which the tab's current key may no longer be.
            // Painting comes before settleSyncRead(), which ignores a
            // cancelled sync.
            if (cacheKey(tab.selectedFile) === bkey) {
                viewer.loading = false;
                viewer.body = body;
            }
            settleSyncRead(run);
        }, function () {
            delete inFlight[bkey];
            if (cacheKey(tab.selectedFile) === bkey) viewer.loading = false;
            settleSyncRead(run);
        });
    }

    function finishSyncIfDone() {
        bumpSyncProgress();
        if (syncing && syncDone >= syncQueued) {
            syncing = false;
            syncedOnce = true;
            // pendingSyncHead may still be "" if its ListBranches lookup is
            // slower than every GetTree/GetBlob in the sync (a small repo)
            // or failed outright — lastSyncedCommit then simply stays at
            // whatever it was, and the next poll (or the next sync) gets
            // another chance to record it.
            //
            // That is only safe because syncAll() clears pendingSyncHead
            // before each sync. It used to be cleared solely in reset(), so a
            // failed lookup here would silently commit the PREVIOUS sync's
            // head and claim the repo was current at a commit it had never
            // fetched — which is the one thing this must never do. Staying
            // stale for a cycle only makes updateAvailable fire late.
            if (pendingSyncHead !== "") lastSyncedCommit = pendingSyncHead;
        }
    }

    /// Lightweight staleness check: ask what the branch's head is right now
    /// and compare against the commit captured at the last completed sync.
    /// Does NOT fetch the tree or any file — one ListBranches call, the same
    /// one syncAll() itself makes to record lastSyncedCommit in the first
    /// place. Never touches syncing/syncQueued/syncDone: a sync in progress
    /// and a staleness poll are independent questions answered by
    /// independent requests, and the button stays clickable regardless of
    /// either.
    function checkForUpdate() {
        if (!app || rid === "" || lastSyncedCommit === "") return;
        var epoch = pollEpoch;
        var wantRid = rid, wantBranch = branch;
        app.call("ListBranches", [rid], function (data) {
            // Drop a reply for a repository/branch/sync the user has already
            // moved on from — the same guard shape as every other loader
            // here, just against pollEpoch instead of syncEpoch.
            //
            // The pollEpoch term is NOT redundant with the rid/branch terms,
            // though it looks that way: reset() is the only place it is
            // bumped, and reset() usually accompanies a rid or branch change.
            // The case it alone catches is a reset with rid and branch
            // UNCHANGED — re-opening the same repository — where the other two
            // terms match and the stale reply would otherwise be compared
            // against a baseline reset() has just cleared. Pinned by
            // tst_staleness.qml's ..._after_a_reset_on_the_same_repo_... test,
            // which fails with this term removed and the other two kept.
            if (epoch !== pollEpoch || tab.rid !== wantRid || tab.branch !== wantBranch) return;
            var items = (data && data.items) ? data.items : [];
            for (var i = 0; i < items.length; i++) {
                if (items[i].name === wantBranch) {
                    tab.updateAvailable = (items[i].head || "") !== lastSyncedCommit;
                    return;
                }
            }
        }, function () { /* transient failure: leave updateAvailable as-is */ });
    }

    /// Polls checkForUpdate() every five minutes while this tab has synced
    /// data to compare against. Deliberately not started until the first
    /// sync completes — polling before there is a lastSyncedCommit to
    /// compare against has nothing to report, and would just be a periodic
    /// no-op request against every repository ever opened, synced or not.
    Timer {
        interval: 5 * 60 * 1000
        running: tab.lastSyncedCommit !== ""
        repeat: true
        onTriggered: tab.checkForUpdate()
    }

    function goUp() {
        var i = path.lastIndexOf("/");
        loadTree(i < 0 ? "" : path.substring(0, i));
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // ---- tree (fixed width) ----
        Rectangle {
            Layout.preferredWidth: Theme.sidebarWidth
            Layout.minimumWidth: Theme.sidebarWidth
            Layout.maximumWidth: Theme.sidebarWidth
            Layout.fillHeight: true
            color: Theme.surface

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                // Breadcrumb (fixed height).
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Theme.rowHeightSm
                    color: Theme.surfaceAlt
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: Theme.gapSm
                        anchors.right: parent.right
                        anchors.rightMargin: Theme.gapSm
                        text: tab.path === "" ? "/" : "/" + tab.path
                        color: Theme.textDim
                        font.pixelSize: Theme.fontXs
                        font.family: Theme.mono
                        elide: Text.ElideLeft
                    }
                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width; height: 1
                        color: Theme.border
                    }
                }

                LoadingState {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    visible: entries.count === 0
                    loading: tab.treeLoading
                    loaded: tab.treeLoaded
                    count: entries.count
                    emptyText: "Empty directory"
                    loadingText: "Loading files…"
                }

                ListView {
                    id: treeList
                    visible: entries.count > 0
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    model: entries
                    clip: true
                    spacing: 0
                    cacheBuffer: Theme.rowHeightSm * 40
                    boundsBehavior: Flickable.StopAtBounds
                    ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

                    header: Item {
                        width: treeList.width
                        height: tab.path === "" ? 0 : Theme.rowHeightSm
                        visible: tab.path !== ""
                        Rectangle {
                            anchors.fill: parent
                            color: upMouse.containsMouse ? Theme.surfaceAlt : "transparent"
                        }
                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: parent.left
                            anchors.leftMargin: Theme.gapSm
                            text: "../"
                            color: Theme.textDim
                            font.pixelSize: Theme.fontMd
                            font.family: Theme.mono
                        }
                        MouseArea {
                            id: upMouse
                            objectName: "fileUpRow"
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: tab.goUp()
                        }
                    }

                    delegate: Rectangle {
                        required property string name
                        required property string kind
                        required property string path

                        readonly property bool selected: tab.selectedFile === path

                        width: treeList.width
                        height: Theme.rowHeightSm
                        color: selected ? Theme.raised
                             : (rowMouse.containsMouse ? Theme.surfaceAlt : "transparent")
                        Behavior on color { ColorAnimation { duration: Theme.animFast } }

                        // Selection marker on the left edge.
                        Rectangle {
                            anchors.left: parent.left
                            width: 2
                            height: parent.height
                            color: parent.selected ? Theme.accent : "transparent"
                        }

                        Row {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: parent.left
                            anchors.leftMargin: Theme.gapSm
                            anchors.right: parent.right
                            anchors.rightMargin: Theme.gapSm
                            spacing: Theme.gapSm

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                width: 10
                                text: kind === "tree" ? "▸" : "·"
                                color: kind === "tree" ? Theme.accent : Theme.textFaint
                                font.pixelSize: Theme.fontSm
                            }
                            Text {
                                // Named so a spec can address a row by the
                                // file's NAME and still be sure it hit the
                                // tree: the viewer's title bar shows the same
                                // string once a file is open, and a bare text
                                // selector matched that instead.
                                objectName: "fileRowName"
                                anchors.verticalCenter: parent.verticalCenter
                                width: parent.width - 26
                                text: name
                                color: parent.parent.selected ? Theme.text : Theme.textDim
                                font.pixelSize: Theme.fontMd
                                font.family: Theme.mono
                                elide: Text.ElideRight
                            }
                        }

                        MouseArea {
                            id: rowMouse
                            // Named on the MouseArea, not the delegate Rectangle:
                            // UI specs select by objectName AND clickability, and
                            // naming a non-clickable parent matches one element
                            // instead of one per row (see SectionTabs.qml).
                            objectName: "fileRow"
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: tab.openEntry({ name: name, kind: kind, path: path })
                            onEntered: prefetchTimer.restart()
                            onExited: prefetchTimer.stop()

                            Timer {
                                id: prefetchTimer
                                interval: 220     // don't fetch on a passing cursor
                                onTriggered: tab.prefetch({ name: rowMouse.parent.name,
                                                            kind: rowMouse.parent.kind,
                                                            path: rowMouse.parent.path })
                            }
                        }
                    }
                }
            }
        }

        Rectangle { Layout.preferredWidth: 1; Layout.fillHeight: true; color: Theme.border }

        // ---- file / readme ----
        FileViewer {
            id: viewer
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }
}
