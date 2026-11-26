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
- No UI behaviour change. `Main.qml` already reports a refusal and snaps the
  picker back; the one edit there is the comment above `onSeedChosen`, and a
  component test pinning that call is #78.

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

Each form catches a URL the other lets through, so neither check is
redundant. The normalised form catches `https://` and `http://`. The given form
catches a URL made only of slashes: `/` and `///` normalise to the empty
string, which the rule accepts because through `setSetting` it means "clear".
Checked as normalised alone, `setRemoteSeed("/")` would probe `/api/v1`
against nothing and, should that answer, store an empty seed. So "a normalised
URL is a prefix of the given one, and the rule is a prefix test" does not make
the given-form check redundant: the empty string is the prefix the rule lets
through.

The reshape that review offered was considered: refuse an empty *normalised*
URL in place of the empty-given check, and validate the normalised form alone,
one check on one form. It was not taken. The spec requires the refusal to carry
the reason `setSetting` gives, and for `/` that reason is the shape rule's
("must start with http:// or https://"), not "seed URL is required"; and the
reshape is correct only because the empty string is the rule's one gap under
normalisation, which is knowledge of the rule's internals held outside it.
Asking `validate` of both forms keeps that knowledge in `validate`.

**Guard:** checking only the typed form turned
`a_scheme_with_no_host_is_refused_before_any_request` red on
`fake.requested.empty()` (watched failing before that fix). Checking only the
normalised form turns
`every_url_set_setting_refuses_is_refused_by_set_remote_seed_though_it_would_answer`
red on its `/` row (measured in one run of the core unit tests alongside three
other mutations, each reaching different tests, log read in full; this is the
only test there that the other three cannot reach). That table now also
asserts no request and
that the refusal contains `setSetting`'s own reason for each row.

### What answering the probe means

`SeedClient::probe` accepts only a 2xx reply whose body is a JSON object with a
string `apiVersion` and no `error` member; anything else is this client's own
`{"error":"<non-empty message>"}`, with `reachable()` false and `apiVersion()`
/ `nid()` empty. Before, any JSON at all answered: a mistyped URL that landed
on another JSON API was adopted, and a non-object body (`[]`, `null`, a scalar)
made `setRemoteSeed` throw from `index.value("nid", "")` after the seed had
been stored and adopted, with no `catch` anywhere between there and the QtRO
slot. Persisting is what turned both from a one-session oddity into a standing
misconfiguration, so "returns JSON" stopped being enough. The `spec-writer` set
the bar in "A seed answers its probe only with a Radicle seed index"; not
measured is that every real seed's `/api/v1` carries a string `apiVersion` (the
module's contract and fixtures assume it, and no seed was reachable from the
sandbox).

The bar was chosen from these, each ruled out for the reason given:

- **Any JSON** (what the code did): a foreign JSON API is persisted, and a
  non-object body threw after the store.
- **Any JSON object**: `{}` from any JSON API passes, so a foreign server is
  still persisted; it closes the throw and nothing else.
- **An object with a string `apiVersion`** (chosen, with the `error` rule
  below): `apiVersion` is the field the module already reads from the index
  and reports as `remoteApiVersion`, so the bar asks for exactly what the
  module uses, and a foreign API is unlikely to carry it by accident.
- **Also requiring a string `nid`**: the spec reports an absent or non-string
  `nid` as empty, so a seed without one is a seed; and the module only shows
  `nid`, it never routes by it.
- **A `Content-Type: application/json` check**: `HttpResponse` carries no
  headers, so it would widen `httpGet` and the transport type for every
  `remote*` call, and it would add nothing the body check does not already
  decide: the right type says nothing about the body being a Radicle index.

**What the bar forecloses, and the escape.** If the assumption above is wrong
and a real seed's index lacks a string `apiVersion`, that seed can no longer
be picked in the UI at all, where before it was adopted for the session. The
escape is `setSetting("remoteSeed", …)`, which stores any well-shaped URL
without a probe and is adopted at the next start; no view exposes it for the
seed today, so the escape is an API caller's, not a user's. That is the cost of
the bar if the assumption fails, and it was accepted because the alternative
cost, a foreign server persisted across restarts, is paid silently.

**An `error` member refuses the seed whatever else the body holds**, a string
`apiVersion` included. This was decided by the owner's delegation in review
round 2, when the text and the code disagreed (the code refused such a body by
accident, by reading the seed's `error` key as its own failure signal, and
relayed the body). Such a body is more plausibly an error reply than a healthy
index, because `{"error":...}` is this module's own failure shape and every
caller reads it as one; and a wrongly accepted seed is now persisted across
restarts, while a wrongly refused one costs a retry. The refusal is built by
`probe`, never the seed's body relayed: a seed can send `"error":null`, `""`,
`0` or `false`, which the view's reply parser (`doc && doc.error`) reads as
success, so a relayed body made the picker report a refused seed as chosen.
The message carries the seed's own `error` text when that is a non-empty
string; the spec fixes the shape, not the wording.

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
  fails leaves nothing from an earlier answer behind, as its header promises.
  `setRemoteSeed` cannot show this, since the candidate's `setSeedUrl` has
  already cleared them, so it is pinned at the `SeedClient` level instead.

**Guard:** accepting any JSON turns
`a_success_that_is_not_a_seed_index_is_a_failed_probe` red (the code before
this fix threw `type_error.306` on its first row), and reading `nid` from the
index again turns `a_nid_that_is_absent_or_not_a_string_is_reported_empty` red
(`type_error.302`). Deleting the two `clear()` lines turns
`a_failed_probe_leaves_nothing_from_an_earlier_answer` (`test_seed_client.cpp`)
red on its `[]` row, with `apiVersion()` still `6.2.0` (measured, in the
shared run). Dropping the
`error`-member check turns `a_success_carrying_an_error_member_is_a_failed_probe`
red, the reply being the seed's two-member body relayed, and the same
`SeedClient` test red on its second row, with `reachable()` left true; those
two and nothing else (measured, the mutation alone). Both new tests were first
watched failing against the code before the fix, on the same assertions.

### A seed's error body is a message only when `error` is a non-empty string

`SeedClient` surfaces a seed's own `{"error":"..."}` on a failure status,
because it is more useful than the bare status. It used to call
`get<std::string>()` on whatever `error` held, so `{"error":5}` threw; and it
passed an empty string on as the message, so a 500 with `{"error":""}` became
a refusal whose `error` a view reads as success. It now surfaces only a
non-empty string and otherwise falls back to the status. The check is in the
one function every `remote*` call goes through (`fetch`, below), so one check
covers them all, where a check in `setRemoteSeed` would cover one. The spec now
says a failure status is a failed probe whatever its body, including a body
that is itself a seed index, and that every failed probe's `error` is a
non-empty string. **Guard:** removing the `is_string()` check turns
`an_error_status_whose_error_is_not_a_string_still_becomes_an_error_object`
(`test_seed_client.cpp`) and `a_failure_status_is_a_failed_probe_whatever_its_body`
red; both were watched failing with `type_error.302`. Removing the non-empty
check turns `an_error_status_whose_error_is_empty_falls_back_to_the_status`
(`test_seed_client.cpp`) and the `{"error":""}` row of
`a_failure_status_is_a_failed_probe_whatever_its_body` red (measured, in the
shared run; both were also watched failing before the fix). `an_error_status_surfaces_the_seeds_own_message`
is the control that the body is still read.

### `probe` knows whose body it holds

`getJson` returns the seed's parsed 2xx body and its own `{"error":...}` as
the same type, told apart only by `isError`, which is true for any object with
an `error` key. So a seed could forge the discriminator: `probe` read a 2xx
index carrying `error` as its own failure and handed the seed's body back as
the refusal, with `reachable()` left true. A private `fetch(path)` now returns
`Fetched{seedAnswered, body}`, and `probe` judges the body only when
`seedAnswered` is true; `getJson` is `fetch(path).body`, so no other caller
changes. Considered:

- **Testing object-with-`apiVersion` before `isError`**, the architecture
  review's first suggestion. It works only because this client's own errors
  happen to carry no `apiVersion`, and it still leaves `{"error":5}` with no
  `apiVersion` to be told apart by the same forgeable key.
- **Reading `m_reachable` after `getJson`**, which `getJson` already sets true
  only on a parsed 2xx. It would work, but gives a field that `getCapabilities`
  reports to views a second, internal meaning, and makes `probe` depend on an
  ordering inside `getJson` that nothing states.
- **Changing `getJson`'s return type** for every caller. Wider than this
  change; the other `remote*` calls pass a seed's reply on rather than judging
  it, and a 2xx body with an `error` key is read by their callers as a failure,
  which is what it most plausibly is.

The guard is the `error`-member check's, above: without `seedAnswered`, there
is no way for `probe` to build that refusal rather than relay the body.

### A seed URL carrying credentials is refused, by the shape rule

`SettingsStore::validate` refuses a non-empty `remoteSeed` whose authority
(between `://` and the first `/`, `?` or `#`) contains `@`, with a message that
names the rule and never the URL. Decided in review round 2. Before this
change a URL typed into the picker (`SeedPicker.qml` is editable, so
`https://user:token@host` reaches `setRemoteSeed`) lived in memory only;
persisting it would write the secret in plain text to `settings.json`, created
by `std::ofstream` with the default umask, normally `0644`, and hand it back
from every `getSettings`. A Radicle seed's HTTP API takes no credentials, so
refusing costs no seed the module can otherwise use.

- **Accepting it and recording that the credential is the user's to place**
  was rejected for that reason: it makes a secret persist where the user did
  not ask for it, for no seed that needs one.
- **Stripping the userinfo and storing the rest** was not taken: it stores and
  reports a URL the user did not type, silently, where a refusal tells them
  why. Probing as typed and stripping only on store would be worse: the probe
  would send the credential (Qt turns userinfo into Basic auth) to a seed
  that takes none.
- **Refusing in `setRemoteSeed` only** would leave the `setSetting` door
  writing the same secret to the same file, and two places deciding what a
  good seed is. In `validate`, both doors refuse it, and `setRemoteSeed`
  refuses it before any request because it asks `validate` first.
- **The message names no part of the URL.** Echoing the URL would hand the
  secret back in the refusal meant to keep it out of storage, and the other
  `remoteSeed` refusal does not echo it either.

Out of scope, as the proposal says: a settings file edited by hand to carry
userinfo (`load()` does not re-run `validate`), and the view's own "Cannot use
<url>" status line, which shows what the user typed on their own screen.
**Guard:** disabling the check turns
`a_seed_url_with_credentials_is_refused_and_its_secret_not_echoed` and
`set_remote_seed_sends_no_request_for_a_url_with_credentials` red (measured,
in the run shared with the other mutations that reach different tests; both
were watched failing before the fix). `an_at_after_the_host_is_not_a_credential`
is the control that the rule is on the authority, one row per terminator.

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

The copy depends on `SeedClient` staying a cheap value type whose only shared
member is the transport, which its class comment now says. The transport is
not required to be stateless: the tests' fake records every request in one log
that the candidate and the client in force share, which is how a test names
the candidate's request. What it must not do is make the candidate's probe
change what the client in force reports; production's default calls
`httpGet`, which keeps nothing. **Guard:** going back to mutate-and-restore turns
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
  trailing slash, and its `remoteSeedChanged` names the slashed form, not the
  one in force. "The URL is normalised once" above holds for `setRemoteSeed`
  only. It is a consequence of leaving `setSetting`'s seed path alone, which
  is #65's. Because the next person to touch either door needs it, it is also
  stated where they will read it: both contracts in `radicle_impl.h`, and a
  comment at the `setSetting` branch beside the `setRemoteSeed` body that
  normalises. Whether #65 names the divergence is for `gh issue view 65` to
  say; if it does not, that is where it belongs.
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
