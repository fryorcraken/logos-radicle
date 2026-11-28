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
  that decides what a read reports, `normalised()`, shared by reads and by the
  write's reply.

**Non-Goals:**

- Anything a read reports. `normalised()` is the old body of `load()`, moved;
  the only edit since is that its defaults are built from the key list that
  `set()` also checks (Decision 2), which changes no value. The two contracts
  outside this file that hold only through its `explore` fallback
  (`node-paths`' "An unrecognised mode resolves nothing", `source-modes`' "An
  unknown mode on disk grants no local access") are untouched.
- Rewriting files already on disk. Every file an earlier build wrote holds all
  five keys, because the normalised view it saved did; this change adds no
  member, and it does not remove those either.
- Preserving the file's text (Decision 6).

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

`load()` is now `normalised(readStored())`. `set()` refuses an unknown key,
then an invalid value, then a store with no path; only then reads the file,
drops the members nested too deep to keep (Decision 9), assigns the one key
and saves the object.

**The unknown-key refusal asks `isKnownKey()`**, backed by `kKnownKeys`, the
one list of the five keys, from which `normalised()` also builds its
defaults. Two other places to judge it were tried or plausible:

- `stored.contains(key)`, the file's own members, refuses `radHome` on a
  fresh file or one that never held it, which is the first-write case.
- `normalised(stored).contains(key)`, which the first pass used, gives the
  right answer only because the defaults literal happens to hold the five
  keys. It built the whole normalised view to read one bit, needed a comment
  to say why it meant "known key", and forced the file read to the top of
  `set()`. Architecture and readability review both raised it.

**The file is read after every refusal**, not before. Nothing that refuses
needs the file, and the span from the read to the rename in `save()` is the
window in which another instance's write is lost (Risks). `validate()` runs
a git binary for a non-empty `gitPath`, so reading first held that window
open for the length of the probe; architecture review found that the first
pass's Risks entry claimed a narrowing the code did not have.

Considered and ruled out:

- **A second parse inside `set()`, leaving `load()` whole.** Two copies of the
  rule "a file that does not parse holds nothing", which would be free to
  drift. The split leaves one.
- **Remembering the unknown members in memory from the last read.** The store
  is stateless on purpose; two instances over one file (two Basecamps on one
  profile, or the restart the tests model) would each write back what they
  last saw and undo the other's change. Reading the file at write time is the
  only source that is never stale.

Removing the fix (starting `set()` from `load()`) turns red exactly the tests
listed under Decision 7.

### 3. The reply is `normalised(stored)`, not a second read and not the file

"A settings write replies with the settings as read, not the file as stored"
requires the reply to equal what `getSettings` reports next. Deriving it from
the object just written gives that without reading the file a second time,
and it describes this write rather than whatever another instance wrote in
between. Returning `stored` itself would hand a view a sixth key and a mode it
has no segment for, or fewer than five keys for a file that never held them.
Run as a mutation in the first pass, that turned red the two new reply tests,
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

**A file that exists but cannot be opened** takes the same branch of
`readStored()` as an absent one: the `std::ifstream` fails to open, so the
store reads an empty object. Replacing it works because `save()` never opens
the settings path for writing; it writes a temporary beside it and
`rename(2)`s that over the path, which replaces whatever entry is there
without following it or needing to read it. The test,
`a_write_over_a_file_that_exists_but_cannot_be_opened_replaces_it`, builds
that file as a symlink pointing at itself (`open(2)` fails with `ELOOP`)
rather than a mode-000 file, because root can read a mode-000 file and the
test would then pass or fail with whoever runs the suite.

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

### 6. Values, not text: `nlohmann::json`, re-serialised

"A settings write keeps each member's value, not the file's text" is why the
file is parsed and re-serialised rather than edited in place. Keeping the text
would mean a JSON writer that splices one member into the original bytes:
more code than the parse, and it is the shape that leaves a member the file
names twice still named twice, which that requirement rules out. So a write
keeps each member's JSON value, and the file's member order, whitespace and
number spelling are not kept. The requirement's two specifics follow from the
parser: a member named twice parses to its last value and is written once
(`a_member_the_file_names_twice_is_kept_once_with_its_last_value`), and a
number beyond an IEEE 754 binary64 double, which `nlohmann::json` parses as a
rounded double, is outside the contract by the requirement's own words.

The type is `nlohmann::json`, whose objects are sorted maps, not
`nlohmann::ordered_json`. The ordered type would keep member order, but it is a
different type from the `nlohmann::json` the store hands every caller, so the
write path would carry a second JSON type and a conversion; and every file
this module has written is already sorted. Not worth it for an order the
requirement leaves out.

**The text is one member per line with each value compact**, written by
`serialise()` in `settings_store.cpp`, not `dump(2)`. `dump(2)` indents every
level of a nested value by its depth, so a kept nested member's text grows
with the square of its depth; correctness review measured a 20 KB file
nested 10000 deep rewritten as 200,040,027 bytes. The cap in Decision 9 bounds
the depth but not the factor, and a file can hold many members. For the flat,
all-string object this module writes for itself, `serialise()` gives the same
text `dump(2)` did. The spec leaves the whitespace open, so the test is a
`NO SPEC:` one: `a_kept_nested_member_does_not_grow_the_file_with_its_depth`
failed against `dump(2)` before `serialise()` existed.

### 7. The tests, and the mutation they were watched failing against

The issue's "Done when" asked for a test that writes
`{"mode":"turbo","remoteSeed":"https://a.example","futureKey":"x"}`, does one
`set` of an unrelated key, and asserts the file on disk, shown to fail against
the mutation it names. That test is
`a_write_of_one_key_keeps_an_unknown_mode_and_an_unknown_key`, in
`radicle/tests/test_settings_store.cpp`. The mutation is the shape `set()` had
before this change, saving the normalised view; re-run on the final code as
`auto stored = load();` in place of `readStored()`, these went red:

- in `test_settings_store.cpp`:
  `a_write_of_one_key_keeps_an_unknown_mode_and_an_unknown_key`,
  `a_write_keeps_a_known_key_whose_value_is_not_a_string`,
  `writing_the_key_that_held_an_uninterpretable_value_replaces_only_that_value`,
  `the_reply_is_the_normalised_view_while_the_file_keeps_its_members`,
  `a_write_over_a_file_that_is_not_json_replaces_it_with_only_the_key_written`,
  `a_write_over_json_that_is_not_an_object_replaces_it_with_only_the_key_written`,
  `a_write_over_a_file_that_exists_but_cannot_be_opened_replaces_it`,
  `the_file_layout_does_not_change_what_a_write_keeps`,
  `a_member_the_file_names_twice_is_kept_once_with_its_last_value`,
  `the_first_write_creates_a_file_holding_only_that_key`,
  `a_member_exactly_128_deep_is_kept`,
  `a_member_129_deep_is_dropped_and_the_rest_kept`,
  `a_kept_nested_member_does_not_grow_the_file_with_its_depth`,
  `a_member_nested_far_deeper_still_lets_the_write_complete` (the depth
  tests fail here because the normalised save adds the four defaults, not
  because of depth);
- in `test_radicle_impl.cpp`:
  `a_seed_stored_by_set_remote_seed_keeps_the_files_other_members`,
  `a_settings_write_replies_with_what_get_settings_reports_not_the_file`.

`a_refused_write_leaves_the_file_as_it_was` stayed green under it, correctly:
the old `set()` also refused before it saved. It guards a different mutation,
a save that happens before a refusal returns, and compares the file byte for
byte so that a rewrite with equal members would still show. The spec-test
review ran that mutation (a `save(stored)` before the unknown-key refusal) and
it was the one test to go red.

The unopenable-file test also guards a mutation of its own: a `set()` that
refuses when the path exists but cannot be opened. Run on the final code
(an `lstat` that succeeds and an `ifstream` that fails, returning an error),
it was the only test to go red.

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

### 9. A member nested deeper than 128 is dropped by a write

Keeping every member means keeping every depth, and correctness and security
review both found what that reaches: nlohmann's parser does not recurse and
accepts any depth, but `dump()` recurses once per level, so a write over a
file holding a member 100000 deep (a 200 KB file) died with SIGSEGV, at
`-O0` and `-O2`, and the reviewers saw it from 20000 levels. The file is left
as it was, so every later write dies the same way, and the crash takes the
Basecamp host with it. It was not reachable before this change, because the
normalised view a write used to save holds only strings. "A settings write
drops a member nested too deep to keep" settles it: of the members other than
the one written, a write keeps every one of depth 128 or less, drops every
deeper one from the file, and completes as any accepted write does.

**Why drop, rather than refuse or reset.** Refusing the write contradicts
Decision 4's reason: the write is the user's way out of a file this build
cannot handle, and a refused write here is refused again on every later
click, which is the dead end Decision 4 exists to avoid. Resetting (treating
the file as unreadable and writing only the key) gets the user out but loses
every other member, the user's own seed and home among them, to save nothing.
Dropping loses the one member that cannot be written back safely, and nothing
this build reads came from it.

**Why 128.** The bound has to sit far above any settings value a build will
write and far below what the call stack holds. Every member this module
writes is a string, depth 0, and a structured setting a later build might add
would plausibly need a handful of levels. On the stack side the reviewers
measured `dump()` surviving 10000 levels and crashing from 20000, in a
harness on a Linux main thread; a thread Basecamp calls the module on can
have a much smaller stack (macOS gives a secondary thread 512 KB by default,
a sixteenth of Linux's usual 8 MB), which would put the crash in the low
thousands. 128 recursive frames of `dump()` is a small fraction of either.
The two tests either side of the cap pin the number: a cap of 127 turned
`a_member_exactly_128_deep_is_kept` red (and the `NO SPEC:` size test, whose
member it drops), and `a_member_129_deep_is_dropped_and_the_rest_kept` failed
against the code before the cap.

**Reads are unaffected, and the drop lives in `set()` alone.** A member deeper
than 128 is an array or an object, never a string, so `normalised()` ignores
it whether or not it is there, and a read reports the same either way.
Putting the drop in `readStored()` would cost every read a walk of the file
for nothing a read reports. A read of a deep file was already safe: the
correctness review read a 100000-deep file through `all()` without a crash,
since parsing does not recurse and neither does nlohmann's destructor.

**The depth check is a guard, and it must not recurse.** `nestsDeeperThan()`
walks with a stack of its own and stops one level past the limit, so a
member a million levels deep costs it 129 steps down that path. A recursive
check would reintroduce the crash it exists to prevent. Run as a mutation (a
recursive function that measures the full depth), the test binary died with
SIGSEGV in `a_member_nested_far_deeper_still_lets_the_write_complete`; that
test also went red as the crash against the code before the cap. A recursive
check that stopped at the limit would not crash, so that test cannot tell it
from the walk; the walk is chosen because its safety does not depend on an
early return being kept.

## Risks / Trade-offs

- [A number beyond a binary64 double is rounded] → `nlohmann::json` parses an
  integer outside the 64-bit range as a double, so
  `{"futureKey":123456789012345678901234567890}` is written back rounded.
  "A settings write keeps each member's value, not the file's text" puts such
  a number outside the contract; no key this or any foreseeable build writes
  is one.
- [A member named twice collapses to its last value] → required, by the same
  requirement, and pinned by
  `a_member_the_file_names_twice_is_kept_once_with_its_last_value`; the read
  side already behaved this way.
- [A member deeper than 128 is lost on the next write] → by requirement
  (Decision 9), and silently: nothing reports the drop. Only a file this
  module did not write can hold one.
- [Two instances writing at once] → each reads the file, sets its key, and
  renames its copy over the file, so the later rename drops the earlier
  writer's key. The old `load()`-then-`save()` had the same window. It is now
  narrower than before: the read happens after every refusal, so the window
  is read, assign, drop, save, and no longer includes `validate()`'s git
  probe (Decision 2).

## Migration Plan

None. A file this build writes is read by an older or a newer build exactly
as before: an older build's `load()` ignores the members it does not know.
Rolling back restores the old rewrite on the next write, losing nothing that
the old build did not already lose.
