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
- A refusal emits no `remoteSeedChanged`, so a view never hears of a seed
  that is not in force.
- `setSetting("remoteSeed", …)` keeps its shape-only, no-network check; its
  requirement is reworded to name `setSetting` rather than "a settings write",
  because `setRemoteSeed` is now also a settings write and does probe.
- No UI change. The picker already calls `setRemoteSeed`, already reports a
  refusal and snaps back to the seed in force.

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
  restart, and that a seed it refuses changes neither the seed in force nor
  the persisted one; rewords "A seed URL is checked for its shape only" to
  scope the no-network rule to `setSetting`.

## Impact

- `radicle/src/radicle_impl.cpp` (`setRemoteSeed`) and its contract comment in
  `radicle/src/radicle_impl.h`, which today says only that it validates.
- Comments in `radicle/src/settings_store.cpp`, `settings_store.h` and
  `radicle_impl.cpp` that describe `setRemoteSeed` as never persisting, or as
  the only place a seed is proved, need re-reading against the new behaviour.
- `radicle/tests/test_radicle_impl.cpp`: the existing `setRemoteSeed` tests
  build the module with a settings store that has no path, which now refuses
  every seed; they need a scratch settings path.
- Out of scope: `setSetting("remoteSeed", …)`'s own untested adoption, and
  `Main.qml`'s lack of a component test that would pin the picker's call, are
  separate open issues (#65 and #78).
