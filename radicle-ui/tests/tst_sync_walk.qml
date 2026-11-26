import QtQuick
import QtTest
import "../src/qml" as Ui

/*
 * How a sync walks what the seed lists, when the listing or the tab does
 * something the plain case does not: a directory that lists itself, a read the
 * seed refuses, a branch changed under a running sync.
 *
 * tst_sync_cancel.qml covers the `repo-sync` spec's bound and cancel
 * requirements; this file covers the walk those rest on. The seed is untrusted
 * input, so what it lists is the test's to choose, including listings no honest
 * seed sends.
 *
 * The fake answers ONE request at a time, when the test serves it, and every
 * reply depends on the request: a listing on its path, a blob's content on its
 * branch and path. The branch is in each served key, so a test can say which
 * branch a read went to.
 */
Item {
    width: 900
    height: 600

    QtObject {
        id: node
        /// Requests received and not yet served, FIFO.
        property var queue: []
        /// "<method>:<branch>:<path>" for each request served, in order.
        property var served: []
        /// Directory path -> its entries. "" is the root.
        property var trees: ({})
        /// Keys ("<method>:<branch>:<path>") the node refuses.
        property var refuse: ({})

        function keyOf(r) {
            if (r.method === "ListBranches") return "ListBranches::";
            return r.method + ":" + r.args[1] + ":" + r.args[2];
        }

        function call(method, args, onOk, onFail) {
            queue.push({ method: method, args: args, onOk: onOk, onFail: onFail });
        }

        function serveNext() {
            var r = queue.shift();
            var k = keyOf(r);
            served.push(k);
            if (refuse[k]) { r.onFail(); return; }
            if (r.method === "ListBranches")
                r.onOk({ items: [{ name: "main", head: "main-head" },
                                 { name: "dev", head: "dev-head" }], default: "main" });
            else if (r.method === "GetTree")
                r.onOk({ entries: trees[r.args[2]] || [] });
            else if (r.method === "GetBlob")
                r.onOk({ path: r.args[2], content: r.args[1] + " content of " + r.args[2] });
            else
                r.onFail();
        }

        function serveAll() {
            while (queue.length > 0) serveNext();
        }

        /// Serve up to and including the first request for `key`.
        function serveThrough(key) {
            while (queue.length > 0) {
                var k = keyOf(queue[0]);
                serveNext();
                if (k === key) return;
            }
        }

        function reset() {
            queue = [];
            served = [];
            trees = ({});
            refuse = ({});
        }
    }

    Ui.SourceTab {
        id: source
        anchors.fill: parent
        visible: false
        app: node
        rid: "rad:zWALK"
        branch: "main"
    }

    TestCase {
        name: "SourceTabSyncWalk"
        when: windowShown

        function init() {
            source.branch = "main";
            source.reset();
            node.reset();
        }

        function timesServed(key) {
            var n = 0;
            for (var i = 0; i < node.served.length; i++)
                if (node.served[i] === key) n++;
            return n;
        }

        // NO SPEC: the spec does not say what a listing that names a directory
        // already listed in the same sync does; this lists each directory once
        // per sync and completes.
        //
        // A hostile or broken seed lists `a` inside `a`. Once `a` is cached, a
        // walk that re-queued it would walk the cached copy, re-queue it, and
        // never yield, freezing the view with Cancel undeliverable. The entry's
        // `kind` is a getter that counts its reads and throws past a ceiling, so
        // that walk fails this test instead of hanging the runner.
        function test_a_directory_that_lists_itself_is_listed_once() {
            var reads = 0;
            var self = {
                get kind() {
                    reads++;
                    if (reads > 1000) throw new Error("the sync walked `a` " + reads + " times");
                    return "tree";
                },
                path: "a",
                name: "a"
            };
            node.trees = {
                "": [{ kind: "tree", path: "a", name: "a" }],
                "a": [self, { kind: "blob", path: "a/x", name: "x" }]
            };

            source.syncAll();
            node.serveAll();

            compare(timesServed("GetTree:main:"), 1);
            compare(timesServed("GetTree:main:a"), 1);
            compare(timesServed("GetBlob:main:a/x"), 1);
            compare(source.syncing, false);
            compare(source.syncedOnce, true);
        }

        // NO SPEC: the same, for a cycle through two directories, each listing
        // the other.
        function test_two_directories_listing_each_other_are_each_listed_once() {
            var reads = 0;
            function tripwire(path) {
                return {
                    get kind() {
                        reads++;
                        if (reads > 1000) throw new Error("the sync walked the cycle " + reads + " times");
                        return "tree";
                    },
                    path: path,
                    name: path
                };
            }
            node.trees = {
                "": [{ kind: "tree", path: "a", name: "a" }],
                "a": [tripwire("b")],
                "b": [tripwire("a")]
            };

            source.syncAll();
            node.serveAll();

            compare(timesServed("GetTree:main:a"), 1);
            compare(timesServed("GetTree:main:b"), 1);
            compare(source.syncing, false);
            compare(source.syncedOnce, true);
        }

        // NO SPEC: the spec does not say what a read the seed refuses during a
        // running sync does; this counts it as done, frees its slot, and the
        // sync carries on and completes.
        function test_a_refused_read_does_not_stall_the_sync() {
            var root = [];
            for (var i = 0; i < 12; i++)
                root.push({ kind: "blob", path: "f" + i, name: "f" + i });
            node.trees = { "": root };
            // f1 is inside the first window; f9 is dispatched only once a slot
            // frees, so a refusal that kept its slot would strand it.
            node.refuse = { "GetBlob:main:f1": true, "GetBlob:main:f2": true,
                            "GetBlob:main:f3": true, "GetBlob:main:f4": true };

            source.syncAll();
            node.serveAll();

            compare(timesServed("GetBlob:main:f11"), 1, "the read after the refusals must still be made");
            compare(source.syncing, false);
            compare(source.syncedOnce, true);
        }

        // NO SPEC: the spec does not say what a branch changed under a running
        // sync without a reset does (RepoView resets the tab on every branch
        // change, so production never does this); this keeps every read on the
        // branch the sync started under.
        function test_every_read_of_a_sync_goes_to_the_branch_it_started_on() {
            var root = [];
            for (var i = 0; i < 20; i++)
                root.push({ kind: "blob", path: "f" + i, name: "f" + i });
            node.trees = { "": root };

            source.syncAll();
            node.serveThrough("GetTree:main:");
            source.branch = "dev";
            node.serveAll();

            var dev = [];
            for (var j = 0; j < node.served.length; j++)
                if (node.served[j].indexOf("GetBlob:dev:") === 0) dev.push(node.served[j]);
            compare(dev.length, 0, "reads sent to dev: " + dev.join(", "));
            compare(timesServed("GetBlob:main:f19"), 1);
        }
    }
}
