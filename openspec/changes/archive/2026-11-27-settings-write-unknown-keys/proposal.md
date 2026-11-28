# Proposal

## Why

Every accepted settings write saves the whole file from the normalised read
rather than from what was on disk. So the first write after a newer build or
a hand edit silently loses a `mode` this build cannot interpret (it is saved
as `explore`), every key this build does not know, and any known key holding a
non-string value. Since the seed picker became a write through the store, an
everyday control triggers it. The read side already promises that an
uninterpretable mode "MUST stay as it was found", so a newer build's choice
survives a downgrade; the write side breaks that promise one click later, and
no requirement says which behaviour is intended.

This change decides it: **a write preserves what it did not change.**
Normalising stays a property of reads only.

## What Changes

- An accepted write through the settings store — by `setSetting`, and by
  `setRemoteSeed` storing a seed — changes only the member of the settings file
  that its key names. Every other member keeps its JSON value: an
  uninterpretable `mode`, a key this build does not know, a known key whose
  value is not a string. The write adds no other member, so defaults are not
  written out for keys the file never held.
- A refused write leaves the file's bytes unchanged.
- What a write keeps is each member's JSON value, not the file's text. The
  file's member order, whitespace and number spelling do not change which
  members a write keeps or their values, and a member the file names twice is
  kept once, with its last value. The text itself, and a number beyond what an
  IEEE 754 double holds, are left out of the contract rather than promised.
- A member the write did not name is kept only while its value nests arrays
  and objects 128 levels deep or less; a deeper one is dropped from the file,
  and the write completes as usual however deep the file nests. Keeping
  arbitrary members means keeping arbitrary depth, which review measured
  crashing the module on a write (a 20 KB file nested 10000 deep was also
  rewritten as a 200 MB one), and refusing the write instead would leave the
  user no way to change a setting. No read changes: a member that deep holds
  no string, so no setting reads from it.
- The refusal of a write by a store with no path now has to name
  `XDG_DATA_HOME` and `HOME`, which the requirement only asked for as "naming
  the problem" and no scenario checked.
- A write over a settings file that is absent, cannot be read, or does not
  parse as a JSON object replaces it with an object holding only the key
  written. This is a write's behaviour; a read of such a file still yields
  defaults, as before.
- The reply to an accepted `setSetting` stays the normalised view, what
  `getSettings` then reports: five keys, an uninterpretable stored `mode`
  reported as `explore`, and no key this build does not know. The file and the
  reply now differ in exactly those members.
- Wording: the new requirement says a member a write keeps is not one the
  module persists, so "The settings store holds exactly five named keys" ("no
  other key") stays as written and does not contradict a file that keeps a
  newer build's key. "A settings file that cannot be read yields defaults" no
  longer says unknown keys are "ignored", which read as a statement about
  writes too. "Settings survive a restart" keeps its sentence that writing one
  key does not disturb another's stored value: that is the five settings as
  read back, which the new requirement's rule about the file's members implies
  rather than contradicts. (Moving the sentence was considered; the CLI
  refuses a MODIFIED block that drops one of the live scenarios, so the move
  would need a remove-and-re-add of the whole requirement for no gain in
  behaviour.)

Not changed: what any read reports. `load()`'s normalisation, and with it
`node-paths`' "An unrecognised mode resolves nothing" and `source-modes`' "An
unknown mode on disk grants no local access", which both hold only because an
unknown mode reads as `explore`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `module-settings`: adds "A settings write changes only the key it names",
  "A settings write keeps each member's value, not the file's text",
  "A settings write drops a member nested too deep to keep",
  "A settings write replies with the settings as read, not the file as stored"
  and "A settings write over an unreadable file stores only the key written";
  modifies "A settings file that cannot be read yields defaults" to match, and
  to require that a store with no path names `XDG_DATA_HOME` and `HOME` when it
  refuses a write.

## Impact

- `radicle/src/settings_store.cpp` (`SettingsStore::set`, and whatever it
  reads the on-disk object with). Both write paths, `setSetting` and
  `setRemoteSeed`, go through `set`, so neither caller changes shape.
- The core module's unit tests: the settings store tests, and the module tests
  for `setRemoteSeed`'s write.
- No change to the JSON contract of any method: `getSettings` and the
  `setSetting` reply keep their shape.
- On-disk compatibility improves in one direction only: a file written by
  this build is read by an older or newer build exactly as before, and a file
  written by a newer build now keeps its members through this build's writes.
