# Design

## Context

See `proposal.md` — Why. The module already adopts a stored `remoteSeed` at
start (`RadicleImpl::adoptPersistedSettings`), and `SettingsStore::set` already
writes atomically. What was missing is the one write on the path the seed
picker uses: `Main.qml`'s `setSeed` calls `setRemoteSeed`, which probed the
seed and adopted it in memory but never called the store.

Two constraints shape the approach:

- `setSetting("remoteSeed", …)` checks shape only and never touches the
  network (`module-settings`: "A seed URL is checked for its shape only"). That
  stays true; it is the other door into the same key.
- A failed probe must keep the previous seed in force and stored. The issue put
  it as: *"Today's rollback must not regress into persisting a seed that
  failed."*

## Goals / Non-Goals

**Goals:**

- `setRemoteSeed` is one operation in the core: shape, probe, store, adopt, in
  that order, with every refusal leaving the module as it was.
- The shape rule `setRemoteSeed` applies is the one `setSetting` applies, by
  construction rather than by two copies kept in step.

**Non-Goals:**

- `setSetting("remoteSeed", …)`'s own adoption, including what an empty value
  does to the seed in force, is untested and stays as it is (#65).
- No UI change. `Main.qml` already reports a refusal and snaps the picker back;
  a component test pinning that call is #78.

## Decisions

### `setRemoteSeed` persists; the UI does not call two entry points

The issue left this open: *"Whether `setRemoteSeed` should persist, or the UI
should call both entry points, or the two should merge into one."* The
`spec-writer` decided it in the proposal, and the code follows.

- **The UI calling both** was rejected. Two calls across the QtRO boundary
  are not one operation: the probe can pass and the write fail, or a view can
  make one call and forget the other, which is the bug as it stood. Logic that
  needs no view belongs in the core, where it is testable without Qt.
- **Merging** was rejected both ways. Making `setSetting("remoteSeed")` probe
  contradicts the shape-only rule that keeps a settings write off the network;
  removing `setRemoteSeed` drops the reply the picker reports from
  (`apiVersion`, `nid`) and narrows a public method in `radicle_impl.h`.
- What remains is two doors with two jobs into one key: `setSetting` stores a
  shape-checked URL without the network; `setRemoteSeed` proves a seed answers,
  then stores it.

### The order is shape, probe, store, adopt

- **Probe before store.** Storing first and undoing the write on a failed
  probe means two writes for one call, and an undo that can itself fail —
  which would leave exactly the failed seed persisted that the issue forbids.
  Probing first means a failed probe never reaches the store at all.
- **Shape before probe.** A URL the store would refuse is not worth a round
  trip, and the caller should hear the shape's reason ("must start with
  http:// or https://") rather than whatever the transport makes of a URL with
  no scheme. Letting the probe run and the store's own validation refuse
  afterwards was considered: it satisfies the spec, since the store would
  refuse and nothing would be adopted, but it spends a request and reports the
  wrong reason. **Guard:** removing the shape check turns
  `a_url_with_no_scheme_is_refused_before_any_request_and_says_why` red, and
  only that test — `…_even_when_the_seed_would_answer` stays green, because
  the store then refuses the same URL.
- **Store before adopt.** The in-force client is replaced only once the write
  has succeeded, so the reply, the event and `getCapabilities` never describe
  a seed the next start will not have.

### A candidate client, not mutate-then-roll-back

`setRemoteSeed` copies the in-force `SeedClient`, points the copy at the new
URL, probes the copy, stores, and only then moves the copy into `m_seed`. Every
refusal returns before that move, so there is no rollback to get right.

The code this replaced set the new URL on `m_seed` and, on a failed probe, set
the old URL back. `SeedClient::setSeedUrl` clears `reachable`, `apiVersion` and
`nid`, so that rollback restored the URL and silently forgot that the seed
answered: a failed pick made a working seed read as unreachable until something
probed it again. With a store write now also able to fail after a successful
probe, a second rollback site would have been needed, each one a place to get
the restore subtly wrong. The copy makes "a refusal changes nothing" true of the
whole client by construction. `SeedClient` is a URL, three cached fields and a
`std::function` transport, so the copy is cheap and shares the transport.
**Guard:** going back to mutate-and-restore turns
`a_failed_probe_leaves_the_seed_in_force_reachable` red; it was watched failing
against the code this replaced.

### A seed that cannot be stored is refused

A store with no path (no `XDG_DATA_HOME` and no `HOME`) refuses every write.
`setRemoteSeed` now refuses such a seed, where it used to adopt it for the
session. That is a behaviour change, stated in the proposal and the spec; the
alternative is the silent loss at the next start this change exists to remove.
The refusal wraps the store's own message ("the seed answered, but it could not
be saved, so it was not adopted: …") so a user is not left wondering why a seed
that answers was turned down. **Guard:** adopting despite a failed write turns
`a_seed_that_answers_but_cannot_be_stored_is_refused` red.

### `SettingsStore::validate` is the one shape rule

The per-key checks moved out of `SettingsStore::set` into a public static
`validate(key, value)`, which `set` calls and `setRemoteSeed` calls before its
probe. It landed as its own commit with no behaviour change. A copy of the
`http://`/`https://` test inside `setRemoteSeed` was the alternative, and is
the two-places-deciding-what-a-good-seed-is shape the store's comment already
warned against. A narrower `checkSeedUrl` was also possible; the per-key
function was preferred because "what `setSetting` would refuse" is what the
spec names, and that is exactly what `validate` answers.

### The stored URL is the one in force

The seed client strips a trailing slash. `setRemoteSeed` stores
`candidate.seedUrl()` rather than its argument, so `getSettings().remoteSeed`,
`getCapabilities().remoteSeed` and the reply's `seed` agree. This is
unspecified behaviour and is pinned by a `NO SPEC:` test.

### Claims about persistence that had gone stale

Phase 1's commit message claimed the chosen seed survived a restart, and the
claim was copied into the foundations `design.md` before review corrected it.
The archived documents are history and stay as they are. The live places that
described `setRemoteSeed` as never storing, or as only rolling back, were
re-read and corrected: the `setRemoteSeed` contract in `radicle_impl.h`, the
class comment in `settings_store.h`, the shape-only comment in
`settings_store.cpp`, a test comment in `test_radicle_impl.cpp`, and the
annotation in `docs/M3-embedded-node-plan.md`, which would otherwise have gone
on calling the claim false after this change made it true.

## Risks / Trade-offs

- [A module with no settings path can no longer change seed at all] → only
  reachable with neither `XDG_DATA_HOME` nor `HOME`, where every other setting
  is already unwritable; the refusal says the seed answered and why it was not
  adopted.
- [The core unit tests cannot see the picker's call] → unchanged by this change;
  `Main.qml` still calls `setRemoteSeed`, and its component test is #78. No
  end-to-end spec restarts Basecamp, so none observes persistence either; the
  restart is modelled in the unit tests as a second instance over one file.
- [`setSetting("remoteSeed", "")` clears the stored seed but leaves the one in
  force] → pre-existing and out of scope (#65); this change does not touch that
  path.
