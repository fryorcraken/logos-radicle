# Proposal

## Why

The seed a user picks is lost on restart. The picker calls `setRemoteSeed`,
which probes the seed and adopts it in memory but never writes the settings
store; the entry point that does persist, `setSetting("remoteSeed", …)`, is
one the UI never calls. The module already adopts a persisted seed at start,
so the only missing link is the write.

## What Changes

- `setRemoteSeed` persists the seed it accepts as the `remoteSeed` setting,
  so a module constructed afresh over the same settings file has it in force.
- `setRemoteSeed` refuses, and changes nothing, for every way a seed can fail:
  an empty URL (refused today), a failed probe (rolled back today, and must
  still persist nothing), a URL `setSetting` would refuse for its shape, and a
  seed the settings store cannot write. The last two are new refusals.
  **Behaviour change:** a module whose settings store cannot be written
  (for example one constructed with no settings path) now refuses a seed it
  previously adopted for the session only. The alternative, adopting it
  anyway, is exactly the silent non-persistence this change exists to remove.
  **Behaviour change** for `setSetting` too: a `remoteSeed` carrying
  credentials, which it accepted, is now refused (see below).
- What "the seed answers its probe" means is stated, because it is now the
  gate on what gets persisted: one GET of `/api/v1` that succeeds with a 2xx
  status and a JSON object carrying a string `apiVersion` and no `error`
  member. **Behaviour change:** a server that answers with any other JSON (an
  array, a scalar, an object with no `apiVersion`) was adopted for the session,
  or made the call throw after adopting it; it is now a failed probe.
  Persisting turns such a mistyped or foreign URL into a standing
  misconfiguration, so "returns JSON" is no longer enough.
- **A 2xx body carrying an `error` member is a failed probe**, whatever else it
  holds (`{"apiVersion":"6.2.0","error":"x"}` included). Decided by the owner's
  delegation in review round 2, because the text and the code disagreed. Such
  a body is more plausibly an error reply than a healthy seed index, since
  `{"error":...}` is this module's own failure shape and every caller reads it
  as one; and a wrongly accepted seed is now persisted across restarts, while
  a wrongly refused one costs a retry.
- Every failed probe replies with only an `error` member, a non-empty string,
  never the seed's own body relayed. The text may be the seed's own `error`
  message when that is a non-empty string; the spec fixes the shape, not the
  wording. A seed's body can
  hold `"error":null`, `""`, `0` or `false`, which the view's reply parser
  reads as success, so a relayed body could make the picker report a refused
  seed as chosen. The non-empty rule applies to a failure status too: a 500
  with `{"error":""}` is refused with a non-empty message.
- A failure status is a failed probe whatever its body says, including an
  error body whose `error` is not a string, so the call always replies.
- **A seed URL carrying credentials (`https://user:token@host`) is refused by
  `setSetting`**, so `setRemoteSeed` refuses it before any request, and the
  refusal does not echo the credential. Decided in review round 2. Before this
  change a URL typed into the picker lived in memory only; persisting it would
  write the secret in plain text to `settings.json`, which is created with the
  default umask, and hand it back from every `getSettings`. A Radicle seed's
  HTTP API takes no credentials, so refusing costs no seed the module can
  otherwise use, while accepting would make a secret persist where the user
  did not ask for it. The alternative, accepting it and recording that the
  credential is the user's to place, was rejected for that reason. The rule
  is on the authority only (an `@` in the path is accepted), and covers writes:
  a settings file edited by hand to carry userinfo is out of scope, as is the
  view's own "Cannot use <url>" status line, which shows what the user typed
  on their own screen and stores nothing.
- A URL that would be stored in a shape `setSetting` refuses is refused before
  any request, with `setSetting`'s reason. That covers the URL after its
  trailing slashes are removed (`https://` becomes `https:`), not only as
  typed.
- A refusal emits no `remoteSeedChanged`, so a view never hears of a seed
  that is not in force; an acceptance emits exactly one, naming the seed. A
  refusal also leaves what was learned about the seed in force
  (`remoteReachable`, `remoteApiVersion`) as it was.
- The stored seed, the seed in force, the reply's `seed` and the event all
  carry the URL with trailing slashes removed, so they agree. The reply's
  `nid` is the probe's when it is a string, and empty otherwise.
- `setSetting("remoteSeed", …)` keeps its shape-only, no-network check; its
  requirement is reworded to name `setSetting` rather than "a settings write",
  because `setRemoteSeed` is now also a settings write and does probe.
- No change to UI behaviour. The picker already calls `setRemoteSeed`, already
  reports a refusal and snaps back to the seed in force. The one UI edit is the
  comment above `onSeedChosen` in `radicle-ui/src/qml/Main.qml`, reworded from
  "validates the seed and reverts to the previous one" to "refuses a seed that
  does not answer (or cannot be saved) and keeps the one in force"; no code
  line in the file changes, so no UI test is due.

### The open question, decided: `setRemoteSeed` persists

The issue left three shapes open. `setRemoteSeed` persisting was chosen.

- **The UI calling both entry points** was rejected. It is two calls across
  the QtRO boundary with nothing making them one operation: the probe can
  pass and the write fail, or a view can make one call and forget the other,
  which is the bug as it stands. Every other caller of `setRemoteSeed` would
  inherit the same trap, and logic that needs no view belongs in the core
  module, where it is testable without Qt.
- **Merging the two into one** was rejected in both directions. Making
  `setSetting("remoteSeed")` probe would contradict "A seed URL is checked for
  its shape only", which keeps a settings write off the network. Removing
  `setRemoteSeed` would drop the reply the picker reports from (`apiVersion`,
  `nid`) and narrow a public method in `radicle_impl.h`.
- **Two entry points with two distinct jobs** is what remains:
  `setSetting("remoteSeed")` stores a shape-checked URL without touching the
  network; `setRemoteSeed` proves a seed answers and then stores it. Both end
  in the same stored key, which the module adopts at start.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `module-settings`: adds that a seed accepted by `setRemoteSeed` survives a
  restart and is reported as its normalised URL, what answering the probe
  means and the shape of the refusal when it does not, that a malformed URL is
  refused before any request, that a seed URL carrying credentials is refused,
  and that a seed it refuses changes neither the seed in force nor the
  persisted one; rewords "A seed URL is checked for its shape only" to scope
  the no-network rule to `setSetting`.

## Impact

- `radicle/src/radicle_impl.cpp` (`setRemoteSeed`) and its contract comment in
  `radicle/src/radicle_impl.h`, which today says only that it validates.
- Comments in `radicle/src/settings_store.cpp`, `settings_store.h` and
  `radicle_impl.cpp` that describe `setRemoteSeed` as never persisting, or as
  the only place a seed is proved, need re-reading against the new behaviour.
- `radicle/src/seed_client.cpp` (`probe`), which decides what answers and
  builds the refusal; and `SettingsStore`'s `remoteSeed` validation, which
  gains the credentials rule for both entry points.
- `radicle/tests/test_radicle_impl.cpp`: the existing `setRemoteSeed` tests
  build the module with a settings store that has no path, which now refuses
  every seed; they need a scratch settings path.
- Out of scope: `setSetting("remoteSeed", …)`'s own untested adoption, and
  `Main.qml`'s lack of a component test that would pin the picker's call, are
  separate open issues (#65 and #78).
