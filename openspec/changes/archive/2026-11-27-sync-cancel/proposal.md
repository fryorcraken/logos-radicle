# Proposal

## Why

Cancelling a sync stops the progress counter but not the work. Every tree and
file read the sync already dispatched is still served, one after another, and
anything asked of the module after the cancel waits behind them: a
`ListBranches` issued after cancelling a sync of a 33-entry repository on the
live seed took 116 s to answer. A user who cancels and then switches branch,
opens a file or polls for staleness sees nothing happen for that long, and
nothing says why. No requirement today says what a cancel does to requests
already sent, so this is specified before it is fixed.

## What Changes

- A sync reads from the seed: the sync control is shown only while the derived
  method prefix is `remote`. This was already the behaviour, and stated
  nowhere; it is what makes "the node a sync reads from" one node.
- A running sync keeps at most 4 of its requests outstanding at once, however
  large the repository.
- A cancelled sync stops reaching the seed: after the cancel, at most 4 of its
  requests are served, however large the repository and however much of the
  sync remained.
- A request made after a cancel is answered without waiting for the cancelled
  sync's remaining work.
- Every request a cancel stops still settles: the busy indicator clears and no
  error is shown for work the user abandoned.
- A cancel stops only the sync. A request the sync did not make is answered as
  it would have been, and a sync started after the cancel runs in full.
- What a cancel leaves behind is pinned: the sync's in-progress state ends at
  once, the repository is not marked as synced, and the commit last recorded as
  synced is unchanged. This was already the behaviour; `sync.yaml` carried it
  as two `NO SPEC` markers.
- `radicle-ui/tests/ui/sync.yaml` goes back to the order a user follows —
  start, cancel, then poll for staleness — which becomes the regression test.

Out of scope: leaving a repository or switching branch while a sync is still
running, without cancelling it first. That abandons the sync through a
different path, and nothing here specifies what it does to the sync's
requests.

## Capabilities

### New Capabilities

- `repo-sync`: the repository sync — the download of every directory and file
  on the selected branch into the view's cache, started and cancelled from one
  control — and what it costs every other request the module serves. This
  change gives it the requirements for cancelling, and the two it rests on:
  that the control is the seed's, and how many requests a running sync keeps
  outstanding.

### Modified Capabilities

None.

## Impact

- `radicle-ui/src/qml/SourceTab.qml`: `syncAll()`, `cancelSync()` and the
  sync's request dispatch. Where the cancel is enforced is `design.md`'s
  decision; the core module and `radicle/src/radicle_impl.h` are unchanged.
- `radicle-ui/tests/ui/sync.yaml`: the step order and its header comment, and
  the two cancel-related `NO SPEC` markers, which this spec answers.
- Component tests around sync: `tst_sync_cancel.qml` and
  `tst_sync_cancel_view.qml`, beside the existing `tst_sync_epoch.qml` and
  `tst_repo_switch.qml`.
