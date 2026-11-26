import QtQuick
import QtTest
import "../src/qml" as Ui

/*
 * What a cancel leaves on the sync CONTROL (the `repo-sync` spec's "What a
 * cancel leaves behind"): the label, read off the Text item that renders it,
 * after the control itself has been clicked twice.
 *
 * tst_sync_cancel.qml drives SourceTab directly, so it cannot see either half
 * of this: that the second click on the button reaches cancelSync(), or what
 * the button then says. Both are RepoView's, and only a RepoView can show
 * them.
 *
 * The fake node answers nothing until the test serves it, and every reply
 * depends on the request: a listing on its path, a branch list on its own
 * head. A fake that answered everything at once could not tell a sync that is
 * still running from one that has stopped.
 */
Item {
    id: harness
    width: 1000
    height: 700

    /// RepoView shows the sync control for the remote source only.
    property string source: "remote"
    property var queue: []
    property var served: []

    function call(method, args, onOk, onFail) {
        queue.push({ method: method, args: args, onOk: onOk, onFail: onFail });
    }

    function serveNext() {
        var r = queue.shift();
        served.push(r.method + ":" + (r.args.length > 2 ? r.args[2] : ""));
        if (r.method === "ListBranches") {
            r.onOk({ items: [{ name: "main", head: "head-on-the-node",
                               remote: "z6MkSelf", isLocal: true }],
                     default: "main" });
        } else if (r.method === "GetTree") {
            var list = [];
            if (r.args[2] === "")
                for (var i = 0; i < 12; i++)
                    list.push({ kind: "blob", path: "f" + i, name: "f" + i });
            r.onOk({ entries: list });
        } else if (r.method === "GetBlob") {
            r.onOk({ path: r.args[2], content: "content of " + r.args[2] });
        } else if (r.method === "GetReadme") {
            r.onFail();
        } else {
            r.onOk({ items: [], hasMore: false });
        }
    }

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

    Ui.RepoView {
        id: page
        anchors.fill: parent
        app: harness
        active: true
    }

    TestCase {
        name: "SyncControlAfterCancel"
        when: windowShown

        function init() {
            queue = [];
            served = [];
            page.rid = "";
            page.repo = null;
        }

        function openRepo(rid) {
            page.rid = rid;
            page.repo = {
                rid: rid,
                payloads: { "xyz.radicle.project": {
                    data: { name: "view", defaultBranch: "main", description: "" },
                    meta: { issues: { open: 0 }, patches: { open: 0 } }
                } }
            };
            // The view starts its loads a turn later than the assignment, and
            // the tree only once the branch list has been answered, so serve
            // as requests appear until the tree is there.
            tryVerify(function () { serveAll(); return page.treeCount > 0; }, 4000,
                      "the tree must have loaded; served: " + JSON.stringify(served));
            serveAll();
        }

        function clickSync() {
            var button = findChild(page, "syncButton");
            verify(button !== null, "the sync control must exist");
            verify(button.visible, "the sync control must be shown for the remote source");
            mouseClick(button);
        }

        function test_a_cancelled_first_sync_offers_download_all() {
            openRepo("rad:zVIEWFIRST");
            compare(page.syncLabel, "Download All", "a repository never synced offers the first download");

            clickSync();
            compare(page.syncing, true);
            verify(/^[0-9]+%$/.test(page.syncLabel),
                   "a running sync shows a percentage, not '" + page.syncLabel + "'");
            // The view already holds the root listing, so the sync has walked
            // it and handed over its first reads without asking for it again.
            verify(queue.length > 1, "the sync must have reads with the node");
            compare(page.syncing, true, "the sync is still running with reads outstanding");

            clickSync();
            // At once: nothing has been served since the click.
            compare(page.syncing, false);
            compare(page.syncLabel, "Download All");

            serveAll();
            compare(page.syncing, false);
            compare(page.syncedOnce, false);
            compare(page.syncLabel, "Download All");
        }
    }
}
