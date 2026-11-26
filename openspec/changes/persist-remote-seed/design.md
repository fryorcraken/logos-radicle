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
- What counts as a seed answering is a stated bar, because it is now the gate
  on what gets persisted.

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

The doors differ in a third way, which reusing the shape rule alone would have
hidden: **an empty URL.** `SettingsStore::validate("remoteSeed", "")` accepts
it, and through `setSetting` it clears the stored seed. `setRemoteSeed` refuses
it before the shape rule runs, because there is no seed at `""` to probe, so
there it can only be a mistake; the spec states the refusal. Delegating the
empty case to `validate` would have made `setRemoteSeed("")` either probe
`/api/v1` against nothing or, had it been special-cased as a reset, become a
second way to clear the seed that the spec does not ask for. **Guard:**
deleting the empty check turns
`an_empty_url_is_refused_and_leaves_the_stored_seed_alone` red, and only that
test (measured by the `spec-test-reviewer`).

### The order is shape, probe, store, adopt

Each gate's position is a guard, measured by moving it and running the core
unit tests (`nix build ./radicle#checks.x86_64-linux.unit-tests`, log read in
full with `nix log`). The two sandbox socket-cap failures the overlay names are
present in every run and are left out below.

- **Shape before probe.** A URL the store would refuse is not worth a round
  trip, and the caller should hear the shape's reason ("must start with
  http:// or https://") rather than whatever the transport makes of a URL with
  no scheme. The spec now requires both halves ("A malformed seed URL is
  refused before any request"); the `spec-writer` judged them observable — a
  typed URL is or is not sent over the network, the message does or does not
  name the shape — and the live "A settings refusal names what was wrong"
  already holds `setSetting` to the second. **Guard:** removing the shape
  check turns `a_url_with_no_scheme_is_refused_before_any_request_and_says_why`
  and `a_scheme_with_no_host_is_refused_before_any_request` red, and only
  those. In both the failing assertion is `fake.requested.empty()`. Their
  message assertions stay green, because the probe then answers and the
  store's own `validate` refuses the URL inside `set`, and the wrapped refusal
  also names both schemes. So the request count pins the order; the message
  assertions pin the reason only while that order holds.
- **Probe before store.** Storing first and undoing the write on a failed
  probe means two writes for one call, and an undo that can itself fail —
  which would leave exactly the failed seed persisted that the issue forbids.
  Probing first means a failed probe never reaches the store. **Guard:** a
  store ahead of the probe turns
  `a_failed_probe_stores_nothing_and_keeps_the_seed_in_force`,
  `a_success_that_is_not_a_seed_index_is_a_failed_probe` and
  `a_failure_status_is_a_failed_probe_whatever_its_body` red, each on the
  stored `remoteSeed`.
- **Store before adopt.** The in-force client is replaced only once the write
  has succeeded, so the reply, the event and `getCapabilities` never describe
  a seed the next start will not have. **Guard:** adopting before the store
  turns `a_seed_that_answers_but_cannot_be_stored_is_refused` and
  `a_seed_that_answers_but_whose_settings_file_cannot_be_written_is_refused`
  red, each on `getCapabilities().remoteSeed`.

### The shape rule is asked of the URL as given and as stored

What `setRemoteSeed` stores is the normalised URL (trailing slashes removed),
so checking only the typed form let `https://` through: it passes as typed,
normalises to `https:`, cost a request, and was then refused by the store with
"the seed answered, but it could not be saved", which blamed the store for a
malformed URL and claimed an answer from something that was never a seed. The
candidate client is therefore built first (it makes no request), and
`SettingsStore::validate` is asked of both `seedUrl` and
`candidate.seedUrl()`.

Checking the normalised form alone would satisfy today's rule, since a
normalised URL is a prefix of the given one and the rule is a prefix test. Both
are asked because the spec names both, and a later rule that is not a prefix
test (a length cap, say) could refuse the given form while accepting the
normalised one. **Guard:** the code that checked only the typed form turned
`a_scheme_with_no_host_is_refused_before_any_request` red on
`fake.requested.empty()`; it was watched failing before the fix.

### What answering the probe means

`SeedClient::probe` accepts only a 2xx reply whose body is a JSON object with a
string `apiVersion`; anything else is `{"error":"..."}`, with `reachable()`
false and `apiVersion()` / `nid()` empty. Before, any JSON at all answered: a
mistyped URL that landed on another JSON API was adopted, and a non-object
body (`[]`, `null`, a scalar) made `setRemoteSeed` throw from
`index.value("nid", "")` after the seed had been stored and adopted, with no
`catch` anywhere between there and the QtRO slot. Persisting is what turned
both from a one-session oddity into a standing misconfiguration, so "returns
JSON" stopped being enough. The `spec-writer` set the bar in "A seed answers
its probe only with a Radicle seed index"; not measured is that every real
seed's `/api/v1` carries a string `apiVersion` (the module's contract and
fixtures assume it, and no seed was reachable from the sandbox).

- **In `probe`, not in `setRemoteSeed`.** The probe's job is to say whether
  this is a seed, and `probe` is where `apiVersion` and `nid` are already
  read. A check in `setRemoteSeed` would have left `probe` returning a
  "success" its only caller then has to second-guess.
- **The reply's `nid` is `SeedClient::nid()`.** `probe` already decides that
  only a string `nid` counts; reading `index.value("nid", "")` a second time
  was a second, throwing opinion about the same field (`"nid":5` threw
  `type_error.302`). The spec now says a `nid` that is absent or not a string
  is reported empty.
- **`probe` clears `apiVersion` and `nid` before it asks**, so a probe that
  fails leaves nothing from an earlier answer behind. For the candidate client
  `setSeedUrl` has already cleared them; this keeps `probe` honest on its own.

**Guard:** accepting any JSON turns
`a_success_that_is_not_a_seed_index_is_a_failed_probe` red (the code before
this fix threw `type_error.306` on its first row), and reading `nid` from the
index again turns `a_nid_that_is_absent_or_not_a_string_is_reported_empty` red
(`type_error.302`).

### A seed's error body is a message only when `error` is a string

`SeedClient::getJson` surfaces a seed's own `{"error":"..."}` on a failure
status, because it is more useful than the bare status. It used to call
`get<std::string>()` on whatever `error` held, so `{"error":5}` threw. It now
surfaces only a string and otherwise falls back to the status. The fix is in
`getJson` rather than in `setRemoteSeed` because every `remote*` call goes
through it: one check covers them all, where a check in the caller would cover
one. The spec now says a failure status is a failed probe whatever its body,
including a body that is itself a seed index. **Guard:** removing the
`is_string()` check turns
`an_error_status_whose_error_is_not_a_string_still_becomes_an_error_object`
(`test_seed_client.cpp`) and `a_failure_status_is_a_failed_probe_whatever_its_body`
red; both were watched failing with `type_error.302`.
`an_error_status_surfaces_the_seeds_own_message` is the control that the body
is still read.

### A candidate client, not mutate-then-roll-back

`setRemoteSeed` copies the in-force `SeedClient`, points the copy at the new
URL, shape-checks, probes the copy, stores, and only then moves the copy into
`m_seed`. Every refusal returns before that move, so there is no rollback to
get right.

The obvious alternative is to set the new URL on `m_seed` and set the old one
back on failure. `SeedClient::setSeedUrl` clears `reachable`, `apiVersion` and
`nid`, so that restore keeps the URL and forgets that the seed answered: a
failed pick makes a working seed read as unreachable until something probes it
again. With a store write also able to fail after a successful probe, a second
restore site would be needed, each one a place to get the restore subtly
wrong. The copy makes "a refusal changes nothing" true of the whole client by
construction, and the spec now says the seed in force includes the
`remoteReachable` and `remoteApiVersion` reported for it.

The copy depends on `SeedClient` staying a cheap value type that shares
nothing mutable with its source beyond the stateless transport, which its class
comment now says. **Guard:** going back to mutate-and-restore turns
`a_failed_probe_leaves_the_seed_in_force_reachable` red; it was watched
failing against the code that did it.

### A seed that cannot be stored is refused

A store can fail to write in two ways: it has no path (no `XDG_DATA_HOME` and
no `HOME`), which refuses before touching any file, or it has a path whose file
cannot be written. Both reach the same `isError(stored)` branch, and both are
tested: `a_seed_that_answers_but_cannot_be_stored_is_refused` (no path) and
`a_seed_that_answers_but_whose_settings_file_cannot_be_written_is_refused` (a
path whose parent is a regular file, so the directory cannot be made whoever
runs the test). `setRemoteSeed` refuses such a seed, where it used to adopt it
for the session. That is a behaviour change, stated in the proposal and the
spec; the alternative is the silent loss at the next start this change exists
to remove. The guard is "Store before adopt", above.

The refusal wraps the store's own message: "the seed answered, but it could not
be saved, so it was not adopted: …". Two choices about that message:

- **The store names the empty-path cause itself.** Its message for a store
  with no path was "could not write settings to " followed by the empty path,
  a sentence ending on nothing. `SettingsStore::set` now says "there is no
  settings file, because neither XDG_DATA_HOME nor HOME is set" — fixed at the
  source, so `setSetting` says it too, rather than in the wrapper, which would
  have fixed one of the two doors. Pinned by
  `a_store_with_no_path_says_why_it_cannot_write`, a `NO SPEC:` test, since no
  requirement says what that refusal says.
- **The settings file's path is passed through to the view.** For a real path
  the store's message names the file, and the security review asked whether a
  sandboxed view should learn it. It is kept: the path is the module's own file
  under the profile's XDG data directory, not key material and not the user's
  Radicle home; `setSetting` already returns the same string to the same view;
  the live spec requires a settings refusal to be shown verbatim and to name
  the path tried; and "could not be saved" with no path leaves a user nothing
  to fix.

### `SettingsStore::validate` is the one shape rule

The per-key checks moved out of `SettingsStore::set` into a public static
`validate(key, value)`, which `set` calls and `setRemoteSeed` calls before its
probe. It landed as its own commit with no behaviour change. A copy of the
`http://`/`https://` test inside `setRemoteSeed` was the alternative, and is
the two-places-deciding-what-a-good-seed-is shape the store's comment already
warned against. A narrower `checkSeedUrl` was also possible; the per-key
function was preferred because "what `setSetting` would refuse" is what the
spec names, and that is exactly what `validate` answers. Its contract says it
is not a pure check for every key: a non-empty `gitPath` is validated by
running it.

### The URL is normalised once, and everything reports that form

The seed client strips trailing slashes. `setRemoteSeed` stores
`candidate.seedUrl()` rather than its argument, so the stored seed,
`getCapabilities().remoteSeed`, the reply's `seed` and the one
`remoteSeedChanged` all carry the same string. The spec now defines the
normalised URL and requires each of them to be it ("An accepted seed is
reported as its normalised URL"). The normalisation stays in `SeedClient`,
which already owned it for building request URLs; a second trim in
`setRemoteSeed` would be a second rule for the same string.

### Claims about persistence that had gone stale

Phase 1's commit message claimed the chosen seed survived a restart, and the
claim was copied into the foundations `design.md` before review corrected it.
The archived documents are history and stay as they are. The live places that
described `setRemoteSeed` as never storing, or as only rolling back, were
re-read and corrected: the `setRemoteSeed` contract in `radicle_impl.h`, the
class comment in `settings_store.h`, the shape-only comment in
`settings_store.cpp`, test comments in `test_radicle_impl.cpp`, the seed
picker's comment in `Main.qml`, and the annotation in
`docs/M3-embedded-node-plan.md`, which would otherwise have gone on calling the
claim false after this change made it true.

## Risks / Trade-offs

- [A module with no settings path can no longer change seed at all] → only
  reachable with neither `XDG_DATA_HOME` nor `HOME`, where every other setting
  is already unwritable; the refusal says the seed answered and names that
  cause.
- [The two doors store the seed in different forms] → `setRemoteSeed` stores
  and adopts the normalised URL; `setSetting("remoteSeed", "https://x/")`
  stores the string verbatim and adopts `https://x` (`m_seed.setSeedUrl`
  strips it), so `getSettings().remoteSeed` and
  `getCapabilities().remoteSeed` disagree after a `setSetting` write with a
  trailing slash. "The URL is normalised once" above holds for
  `setRemoteSeed` only. It is a consequence of leaving `setSetting`'s seed
  path alone, which is #65's; that issue does not yet name this divergence.
- [A seed pick rewrites the whole settings file from `load()`] → `load()`
  sends a mode this build does not know to `explore` and drops keys it does not
  know; `set` saves that normalised object, so the first write after a newer
  build (or a hand edit) left `"mode":"someNewMode"` or an extra key loses
  them. This was already true of every `setSetting`; what is new is that the
  seed picker, an everyday control, is now a write. Accepted for this change:
  preserving what `set` did not change is a behaviour of every write through
  the store, not of `setRemoteSeed`, and no requirement asks for it. Not
  tracked by an issue yet; the live "The unreadable-mode fallback is not a
  file rewrite" covers a read only.
- [The core unit tests cannot see the picker's call] → unchanged by this change;
  `Main.qml` still calls `setRemoteSeed`, and its component test is #78. No
  end-to-end spec restarts Basecamp, so none observes persistence either; the
  restart is modelled in the unit tests as a second instance over one file.
- [`setSetting("remoteSeed", "")` clears the stored seed but leaves the one in
  force] → pre-existing and out of scope (#65); this change does not touch that
  path.
