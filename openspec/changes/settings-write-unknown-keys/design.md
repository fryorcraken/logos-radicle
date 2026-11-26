# Design

## Context

`SettingsStore` keeps no state between calls: every read and every write
opens the settings file afresh, which is what lets "a restart" be modelled in
the tests as a second store over the same path. Before this change `set()`
was `load()`, assign one key, `save()` the result. `load()` is the
normalising read: it starts from the five defaults, copies a member of the
file only when its key is one of the five and its value is a string, and
sends a mode this build does not know to `explore`. So `set()` saved the
normalised view, and every member that view leaves out was lost on the first
write (see `proposal.md`, Why).

Both writing methods go through `set()`: `setSetting` directly, and
`setRemoteSeed` once its candidate seed has answered. Neither caller changes.

## Goals / Non-Goals

**Goals:**

- One place that decides what a write puts on disk, in `set()`, and one place
  that decides what a read reports, shared by reads and by the write's reply.

**Non-Goals:**

- Anything a read reports. `normalised()` is the old body of `load()`, moved
  and unchanged, so the two contracts outside this file that hold only
  through its `explore` fallback (`node-paths`' "An unrecognised mode resolves
  nothing", `source-modes`' "An unknown mode on disk grants no local access")
  are untouched.
- Rewriting files already on disk. Every file an earlier build wrote holds all
  five keys, because the normalised view it saved did; this change adds no
  member, and it does not remove those either.
- Preserving the file's text. A write re-serialises the object (see Decision
  6); only the JSON value of each member is kept.

## Decisions

### 1. A write preserves what it did not change; normalising is for reads

The issue left this open: "Whether a write should preserve what it did not
change, or whether normalising on write is the intended behaviour and the
spec should say so." The spec-writer decided for preserving, and the spec now
says so ("A settings write changes only the key it names"). The reason is the
promise the read side already made: an uninterpretable stored mode "MUST stay
as it was found", so that switching back to the newer build that wrote it
loses nothing. Normalising on write kept that promise only until the next
click. Since #101 made the seed picker's `setRemoteSeed` a write through the
store, that click is an everyday one.

Normalising on write, written into the spec, was the alternative. It would
have made the read side's promise worthless rather than kept it, and given a
downgrade a silent cost that nothing reports.

### 2. The issue's shape: read the raw object, set one key, write it back

The issue suggested "read the raw JSON object from disk, set the one key, and
write that back, leaving the normalising `load()` to reads only". That is the
shape taken. `load()` was split, in a commit of its own that changes no
behaviour, into:

- `readStored()`: the file's JSON object as it is on disk, or an empty object
  when there is no path, no file, a file that cannot be read, or one that
  does not parse as an object;
- `normalised(stored)`: pure, the five keys as a read reports them.

`load()` is now `normalised(readStored())`, and `set()` reads the file once,
judges the key against `normalised(stored)`, assigns into `stored` and saves
`stored`.

Considered and ruled out:

- **A second parse inside `set()`, leaving `load()` whole.** Two copies of the
  rule "a file that does not parse holds nothing", which would be free to
  drift. The split leaves one.
- **Remembering the unknown members in memory from the last read.** The store
  is stateless on purpose; two instances over one file (two Basecamps on one
  profile, or the restart the tests model) would each write back what they
  last saw and undo the other's change. Reading the file at write time is the
  only source that is never stale.

Removing the fix (restoring `save(normalised)` in `set()`) turns red exactly
the tests listed under Decision 7.

### 3. The reply is `normalised(stored)`, not a second read and not the file

"A settings write replies with the settings as read, not the file as stored"
requires the reply to equal what `getSettings` reports next. Deriving it from
the object just written gives that without reading the file a second time,
and it describes this write rather than whatever another instance wrote in
between. Returning `stored` itself would hand a view a sixth key and a mode it
has no segment for, or fewer than five keys for a file that never held them.
Run as a mutation, that turns red the two new reply tests,
`the_reply_is_the_normalised_view_while_the_file_keeps_its_members` and
`a_settings_write_replies_with_what_get_settings_reports_not_the_file`, and
three that were already there: `set_returns_the_whole_settings_object_not_just_the_changed_key`,
`the_store_reports_exactly_five_keys_before_and_after_each_write` and
`get_settings_reports_exactly_five_keys_before_and_after_every_write`.

### 4. A write over an unreadable file replaces it rather than refusing

"A settings write over an unreadable file stores only the key written". The
write is the user's way out of a file this build cannot read; refusing it
would leave settings that can never be changed until the user finds and
deletes a file under their XDG data directory that they cannot see and did
not knowingly write. Reads already treat such a file as holding nothing, so
the write's view and the read's view of it agree, and the user's choice is
the only thing the new file holds.

Considered and ruled out: refusing with a message that names the file, which
is the dead-end above; and moving the unreadable file aside as a backup
before replacing it. Nothing asks for the backup, it is a second file and a
second policy (when is it cleaned?), and the torn write it would guard against
is what `save()`'s temp-file-and-rename with `fsync` already prevents.

The one case that still refuses is a file that cannot be *written*: the path's
parent is not a directory, or the rename fails. That is
`a_seed_that_answers_but_whose_settings_file_cannot_be_written_is_refused`,
unchanged.

### 5. A write adds nothing: no defaults written out

The file holds only what someone chose. Writing the defaults beside the key
would freeze this build's defaults into the file, where a later build reads
them as the user's choices: an empty `radHome` on disk is indistinguishable
from one the user cleared. The default mode has already moved once (from
`local` to `explore`, see `normalised()`); under a write-the-defaults rule
every file written before that move would have pinned the old default. The
spec states it as "It MUST add no other member", and
`the_first_write_creates_a_file_holding_only_that_key` is the test that a
normalised save fails by adding four members.

### 6. `nlohmann::json`, not `ordered_json`; the text is re-serialised

The file is parsed into `nlohmann::json`, whose objects are sorted maps, and
written back with `dump(2)`. A hand-edited file's member order and spacing are
therefore not kept, only each member's JSON value, which is what the spec
asks ("MUST parse as a JSON object equal to"). `nlohmann::ordered_json` would
keep the order, but it is a different type from the `nlohmann::json` the
store hands every caller, so the write path would carry a second JSON type
and a conversion; and every file this module has written is already sorted.
Not worth it for an order nobody reads.

### 7. The tests, and the mutation they were watched failing against

The issue's "Done when" asked for a test that writes
`{"mode":"turbo","remoteSeed":"https://a.example","futureKey":"x"}`, does one
`set` of an unrelated key, and asserts the file on disk, shown to fail against
the mutation it names. That test is
`a_write_of_one_key_keeps_an_unknown_mode_and_an_unknown_key`, in
`radicle/tests/test_settings_store.cpp`. The mutation is the code before this
change, `set()` saving the normalised view; the tests were written first and
run against it, and these went red:

- in `test_settings_store.cpp`:
  `a_write_of_one_key_keeps_an_unknown_mode_and_an_unknown_key`,
  `a_write_keeps_a_known_key_whose_value_is_not_a_string`,
  `writing_the_key_that_held_an_uninterpretable_value_replaces_only_that_value`,
  `the_reply_is_the_normalised_view_while_the_file_keeps_its_members`,
  `a_write_over_a_file_that_is_not_json_replaces_it_with_only_the_key_written`,
  `a_write_over_json_that_is_not_an_object_replaces_it_with_only_the_key_written`,
  `the_first_write_creates_a_file_holding_only_that_key`;
- in `test_radicle_impl.cpp`:
  `a_seed_stored_by_set_remote_seed_keeps_the_files_other_members`,
  `a_settings_write_replies_with_what_get_settings_reports_not_the_file`.

`a_refused_write_leaves_the_file_as_it_was` stayed green under it, correctly:
the old `set()` also refused before it saved. It guards a different mutation,
a save that happens before a refusal returns, and compares the file byte for
byte so that a rewrite with equal members would still show. Run as a mutation
(a `save(stored)` straight after `readStored()`), together with Decision 3's
reply mutation in one build, it went red, and it was the only failure that
the reply mutation does not explain.

Every assertion about the file parses the file in the test itself, never
through the store: the reply is the normalised view and would hide exactly
what these tests are about.

### 8. Not the same as #63, and what this supersedes

The issue drew the line: the live scenario "The unreadable-mode fallback is not
a file rewrite" covers a **read**, loading a file with an unknown mode must not
write it back, and #63 asks for a test that reads the file back after such a
load. This change is the **write** path, a deliberate `set` of one key, which
also rewrote the unknown mode and keys it did not touch, and a test for #63
stays green while that happens. The tests here are all of a write; none of
them closes #63.

This change supersedes the Risks entry "A seed pick rewrites the whole
settings file from `load()`" in
`openspec/changes/archive/2026-11-26-persist-remote-seed/design.md`. That
change accepted the rewrite because "preserving what `set` did not change is
a behaviour of every write through the store, not of `setRemoteSeed`, and no
requirement asks for it". The requirement now exists, it is met in `set()` for
every write as that entry said it would have to be, and
`a_seed_stored_by_set_remote_seed_keeps_the_files_other_members` pins it for
the seed picker's path.

## Risks / Trade-offs

- [A number the file holds beyond what a double represents is not kept
  exactly] → `nlohmann::json` parses an integer outside the 64-bit range as a
  double, so `{"futureKey":123456789012345678901234567890}` is written back
  rounded. The spec says a member keeps "the JSON value it had", and is silent
  on numbers JSON itself does not promise to round-trip (RFC 8259 §6). No key
  this or any foreseeable build writes is such a number; reported to the
  spec-writer rather than guarded here.
- [Duplicate member names collapse] → a hand-edited file naming one key twice
  parses to the last value, and a write saves only that. Same parser, same
  silence in the spec; the read side already behaved this way.
- [Two instances writing at once] → each reads the file, sets its key, and
  renames its copy over the file, so the later rename drops the earlier
  writer's key. Unchanged by this change (the old `load()`-then-`save()` had
  the same window) and narrower than before, since a write now reads the file
  immediately before saving it.

## Migration Plan

None. A file this build writes is read by an older or a newer build exactly
as before: an older build's `load()` ignores the members it does not know.
Rolling back restores the old rewrite on the next write, losing nothing that
the old build did not already lose.
