import QtQuick
import QtTest
import "../src/qml" as Ui

/*
 * What cancelling a sync does to the requests it already made (the
 * `repo-sync` spec's cancel requirements).
 *
 * The fake below is a node that answers ONE request at a time, in the order it
 * received them — which is how the module serves calls, and why a cancel that
 * only orphans replies left every later request waiting behind the cancelled
 * sync's backlog. The test decides when each request is served, so it can
 * count exactly which requests reached the node after a cancel, and in what
 * order relative to a request made afterwards.
 *
 * Every reply is input-dependent: a tree listing depends on the path asked
 * for, a blob's content names its path, and the branch head is the fake's own
 * value. A fake that answered everything identically could not tell "this
 * file's contents arrived" from "some file's contents arrived".
 *
 * The bound is 4: SourceTab.syncMaxOutstanding, recorded in the change's
 * design.md. It is written here as a literal, not read off the component, so
 * that a change to the bound is a change to this file as well.
 */
Item {
    width: 900
    height: 600

    readonly property int bound: 4

    QtObject {
        id: node
        /// Requests received and not yet served, FIFO.
        property var queue: []
        /// "<method>:<path>" for each request served, in the order served.
        property var served: []
        /// Directory path -> its entries. "" is the root.
        property var trees: ({})
        property string head: "head-on-the-node"

        function call(method, args, onOk, onFail) {
            queue.push({ method: method, args: args, onOk: onOk, onFail: onFail });
        }

        function serveNext() {
            var r = queue.shift();
            served.push(r.method + ":" + (r.args.length > 2 ? r.args[2] : ""));
            if (r.method === "ListBranches")
                r.onOk({ items: [{ name: "main", head: head }], default: "main" });
            else if (r.method === "GetTree")
                r.onOk({ entries: trees[r.args[2]] || [] });
            else if (r.method === "GetBlob")
                r.onOk({ path: r.args[2], content: "content of " + r.args[2] });
            else
                r.onFail();
        }

        /// Serve until nothing is left, including what serving itself asks.
        function serveAll() {
            while (queue.length > 0) serveNext();
        }

        /// Serve up to and including the first request for `key`.
        function serveThrough(key) {
            while (queue.length > 0) {
                var r = queue[0];
                var k = r.method + ":" + (r.args.length > 2 ? r.args[2] : "");
                serveNext();
                if (k === key) return;
            }
        }

        function reset() {
            queue = [];
            served = [];
            trees = ({});
            head = "head-on-the-node";
        }
    }

    Ui.SourceTab {
        id: source
        anchors.fill: parent
        visible: false
        app: node
        rid: "rad:zCANCEL"
        branch: "main"
    }

    TestCase {
        name: "SourceTabSyncCancel"
        when: windowShown

        function init() {
            source.reset();
            node.reset();
        }

        /// A root holding `files` files and `dirs` directories of three files
        /// each. Returns the number of tree and file reads a full sync makes.
        function makeRepo(files, dirs) {
            var t = {};
            var root = [];
            for (var d = 0; d < dirs; d++) {
                var dp = "d" + d;
                root.push({ kind: "tree", path: dp, name: dp });
                t[dp] = [];
                for (var j = 0; j < 3; j++) {
                    var fp = dp + "/f" + j;
                    t[dp].push({ kind: "blob", path: fp, name: "f" + j });
                }
            }
            for (var i = 0; i < files; i++)
                root.push({ kind: "blob", path: "f" + i, name: "f" + i });
            t[""] = root;
            node.trees = t;
            return 1 + dirs + files + 3 * dirs;
        }

        function isSyncRead(key) {
            return key.indexOf("GetTree:") === 0 || key.indexOf("GetBlob:") === 0;
        }

        function syncReadsServedSince(mark) {
            var n = 0;
            for (var i = mark; i < node.served.length; i++)
                if (isSyncRead(node.served[i])) n++;
            return n;
        }

        function queuedSyncReads() {
            var n = 0;
            for (var i = 0; i < node.queue.length; i++)
                if (node.queue[i].method === "GetTree" || node.queue[i].method === "GetBlob") n++;
            return n;
        }

        /// Start a sync and let the node answer its first directory listing.
        function startAndAnswerRoot() {
            source.syncAll();
            node.serveThrough("GetTree:");
            verify(queuedSyncReads() > 0,
                   "the sync must have asked for something after its root listing");
        }

        function cancelAndCountReadsThatReachTheNode() {
            source.cancelSync();
            var mark = node.served.length;
            node.serveAll();
            return syncReadsServedSince(mark);
        }

        // -- A cancelled sync stops reaching its node ---------------------

        function test_a_large_sync_cancelled_early_stops_well_short_of_its_end() {
            var total = makeRepo(60, 10);
            startAndAnswerRoot();
            var remaining = total - 1;   // everything but the root listing
            var after = cancelAndCountReadsThatReachTheNode();
            verify(after <= bound,
                   after + " of the sync's reads reached the node after the cancel; the bound is " + bound);
            verify(after < remaining,
                   "a cancel must stop short of the " + remaining + " reads the sync had left");
        }

        function test_the_bound_does_not_grow_with_the_repository() {
            makeRepo(20, 0);
            startAndAnswerRoot();
            var small = cancelAndCountReadsThatReachTheNode();

            source.reset();
            node.reset();
            makeRepo(200, 30);
            startAndAnswerRoot();
            var large = cancelAndCountReadsThatReachTheNode();

            verify(small <= bound, "small repository: " + small + " reads after the cancel");
            verify(large <= bound, "large repository: " + large + " reads after the cancel");
        }

        // -- A request made after a cancel does not wait for the sync -----

        function test_a_branch_listing_after_a_cancel_overtakes_the_backlog() {
            makeRepo(60, 10);
            source.lastSyncedCommit = "an-older-commit";
            startAndAnswerRoot();

            source.cancelSync();
            var mark = node.served.length;
            source.checkForUpdate();
            node.serveThrough("ListBranches:");

            var ahead = syncReadsServedSince(mark);
            verify(ahead <= bound,
                   ahead + " of the cancelled sync's reads were served ahead of the branch listing");
            // The reply was a real answer, read and compared: the fake's head
            // differs from the one recorded, so the flag can only be set by a
            // reply that carried the node's branches.
            compare(source.updateAvailable, true);
        }

        // -- What a cancel leaves behind ----------------------------------

        function test_a_cancel_ends_the_sync_before_any_further_reply() {
            makeRepo(60, 10);
            startAndAnswerRoot();
            source.cancelSync();
            compare(source.syncing, false);
        }

        function test_a_cancelled_first_sync_is_not_reported_as_synced() {
            makeRepo(10, 2);
            startAndAnswerRoot();
            source.cancelSync();
            node.serveAll();
            compare(source.syncing, false);
            compare(source.syncedOnce, false);
        }

        function test_the_last_synced_commit_survives_a_cancel() {
            makeRepo(10, 2);
            source.lastSyncedCommit = "recorded-before";
            // The sync's own head lookup is answered first, before the cancel,
            // so the head it captured is in hand; it must still not be
            // recorded.
            startAndAnswerRoot();
            source.cancelSync();
            node.serveAll();
            compare(source.lastSyncedCommit, "recorded-before");
        }

        // -- A cancel stops only the sync ---------------------------------

        function test_a_file_queued_by_the_sync_still_opens_after_the_cancel() {
            makeRepo(60, 0);
            startAndAnswerRoot();
            // f59 sorts last in the root and is far beyond the bound, so the
            // sync has not asked for it yet.
            source.openEntry({ name: "f59", kind: "blob", path: "f59" });
            source.cancelSync();
            node.serveAll();
            compare(source.viewerTitle, "f59");
            compare(source.viewerBodyLength, "content of f59".length);
        }

        // The same file, opened after the cancel rather than before it. The
        // sync had discovered f59 and held it back; a cancel that left f59
        // marked as already being fetched would make this open wait for a
        // reply that is never coming.
        function test_a_file_the_cancelled_sync_had_queued_opens_after_the_cancel() {
            makeRepo(60, 0);
            startAndAnswerRoot();
            source.cancelSync();
            source.openEntry({ name: "f59", kind: "blob", path: "f59" });
            node.serveAll();
            compare(source.viewerTitle, "f59");
            compare(source.viewerBodyLength, "content of f59".length);
        }

        // A request the sync did not make is not the sync's to stop, whenever
        // it is made: here a directory, listed either side of the cancel while
        // the cancelled sync's own reads are still with the node.
        function test_a_directory_opened_either_side_of_the_cancel_lists_data() {
            return [
                { tag: "opened before the cancel", before: true },
                { tag: "opened after the cancel", before: false }
            ];
        }

        function test_a_directory_opened_either_side_of_the_cancel_lists(data) {
            makeRepo(60, 10);
            startAndAnswerRoot();
            // d9 is far beyond the bound in the sync's walk, so no read of it
            // was ever handed over: this request is the only one for it.
            if (data.before) source.openEntry({ name: "d9", kind: "tree", path: "d9" });
            source.cancelSync();
            if (!data.before) source.openEntry({ name: "d9", kind: "tree", path: "d9" });
            node.serveAll();
            compare(source.path, "d9");
            compare(source.entryNames, "f0,f1,f2");
        }

        function test_a_file_the_sync_is_already_fetching_still_opens() {
            makeRepo(60, 0);
            startAndAnswerRoot();
            // f0 is the first file in the root, so the sync's read of it is
            // already with the node when the user opens it.
            source.openEntry({ name: "f0", kind: "blob", path: "f0" });
            source.cancelSync();
            node.serveAll();
            compare(source.viewerTitle, "f0");
            compare(source.viewerBodyLength, "content of f0".length);
        }

        // Run twice: the next sync starts once the node has answered what the
        // cancelled one left with it, or while those requests are still ahead
        // of everything the next sync asks for.
        function test_a_sync_after_a_cancel_downloads_everything_data() {
            return [
                { tag: "after the node has drained", drainFirst: true },
                { tag: "while the cancelled sync's reads are still with the node", drainFirst: false }
            ];
        }

        function test_a_sync_after_a_cancel_downloads_everything(data) {
            var total = makeRepo(30, 5);
            startAndAnswerRoot();
            source.cancelSync();
            if (data.drainFirst) node.serveAll();

            source.syncAll();
            node.serveAll();

            compare(source.syncing, false);
            compare(source.syncedOnce, true);
            // Every directory and file was asked of the node, by one sync or
            // the other: none was skipped because the cancelled sync had
            // queued it.
            var asked = {};
            for (var i = 0; i < node.served.length; i++)
                if (isSyncRead(node.served[i])) asked[node.served[i]] = true;
            compare(Object.keys(asked).length, total);
        }

        function test_a_late_reply_from_the_cancelled_sync_is_ignored() {
            makeRepo(60, 10);
            startAndAnswerRoot();
            source.cancelSync();
            var stale = node.queue.length;
            verify(stale > 0, "the cancelled sync must still have reads with the node");

            // The next sync starts while the cancelled one's reads are still
            // with the node; they sit ahead of everything it asks for.
            source.syncAll();
            var queued = source.syncQueued, done = source.syncDone;
            var progress = source.syncProgress;
            for (var i = 0; i < stale; i++)
                node.serveNext();

            compare(source.syncQueued, queued, "a late reply must not add to the new sync's queued count");
            compare(source.syncDone, done, "a late reply must not add to the new sync's done count");
            compare(source.syncProgress, progress, "a late reply must not move the new sync's progress");
        }

        // Run twice: with directories in the root, the cancelled sync's
        // outstanding reads are listings; with none, they are files. Each
        // reply path has its own epoch check.
        function test_a_late_reply_does_not_free_a_slot_in_the_next_sync_data() {
            return [
                { tag: "listings outstanding", files: 60, dirs: 10 },
                { tag: "files outstanding", files: 60, dirs: 0 }
            ];
        }

        function test_a_late_reply_does_not_free_a_slot_in_the_next_sync(data) {
            makeRepo(data.files, data.dirs);
            startAndAnswerRoot();
            source.cancelSync();
            // What is with the node now belongs to the cancelled sync.
            var old = node.queue.slice();
            verify(old.length > 0, "the cancelled sync must still have reads with the node");
            // The cancelled sync's answered root listing is in the cache, so
            // the next sync walks it at once and has a full queue of its own
            // BEFORE the old replies arrive. Without that the old replies
            // would be served ahead of the next sync's first read, and a slot
            // wrongly freed by one of them would have nothing to dispatch.
            source.syncAll();

            // Count the NEW sync's reads that are with the node unanswered,
            // now and after each request served. A late reply from the
            // cancelled sync that was taken as one of the new sync's own
            // would let it put a fifth read out.
            function mineWithTheNode() {
                var n = 0;
                for (var i = 0; i < node.queue.length; i++) {
                    var r = node.queue[i];
                    if (old.indexOf(r) < 0 && (r.method === "GetTree" || r.method === "GetBlob"))
                        n++;
                }
                return n;
            }
            var most = mineWithTheNode();
            verify(most > 0, "the new sync must have asked for something before any old reply arrived");
            while (node.queue.length > 0) {
                node.serveNext();
                most = Math.max(most, mineWithTheNode());
            }
            verify(most <= bound, "the new sync had " + most + " reads with the node at once");
            compare(source.syncedOnce, true);
        }
    }
}
