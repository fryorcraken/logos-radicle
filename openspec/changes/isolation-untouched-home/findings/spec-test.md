# spec-test review: isolation-untouched-home

Reviewed: the delta `specs/embedded-identity/spec.md` (one requirement, two
scenarios) against `radicle/rust-ffi/tests/profile_init.rs`; issue #59 read
fresh with `gh issue view 59`; the implementation was read only at the lines
mutated.

**Coverage (part 1).** Both scenarios are pinned at the Rust FFI layer, which is
the only layer that can create an identity into a scratch home and read the
tree back.

- "Creating into home A leaves home B's profile as it was" is
  `two_homes_get_two_different_identities`: B is created first, its whole tree
  recorded (`tree_record`), `keys/radicle`, `keys/radicle.pub` and `config.json`
  asserted present as files, A created, A's `nodeId` asserted non-empty, and
  `record_changes` against B asserted empty. Added, removed and byte-changed
  entries are all in the record, and the recorder's own tests pin each branch
  against hand-written expectations.
- "Creating into home B does not bring home A into being" is the same test: A's
  path asserted absent before B is created and again after.

**Can they fail (part 2).** Read: the node-id inequality is between two
independent creations, the preconditions stop two empty records comparing
equal, and the recorder tests use expected values written by hand, not derived
from the recorder. Mutated:

1. `init_profile_inner`, just before the `Ok(json!(...))`: when the home path
   contains `profile-init-iso-a`, copy A's `keys/radicle.pub` over sibling
   B's. Command `cargo test --manifest-path radicle/rust-ffi/Cargo.toml --test
   profile_init`: `two_homes_get_two_different_identities` FAILED with
   `changed: keys/radicle.pub`; the other 21 passed. Killed.
2. Same place, before `Home::new` takes effect: when the home contains
   `profile-init-iso-b`, `create_dir_all` the `iso-a` sibling. Same command
   narrowed to `two_homes`: FAILED at the post-B absence assertion
   (`creating an identity into home B brought home A into being`). Killed.

Both mutations are restored; `git status --short` is empty. The remaining
mutations the tasks name (config overwritten, one entry added and one removed,
null implementation) were not re-run; the recorder table tests cover the added,
removed and changed arms by reading.

**What the dev decided that the spec never said (part 3).** No `NO SPEC:`
markers in `profile_init.rs` (the three in the tree are in `node_config.rs` and
`node_seeding.rs`, other changes). Unmarked behaviour: the recorder's own tests
(`the_recorder_*`) pin an instrument, not product behaviour, so no scenario is
owed. The `nodeId` inequality and "each home holds its own profile" assertions
in the same test are the existing requirement "Two homes yield two different
node ids" in `openspec/specs/embedded-identity/spec.md`, not new behaviour.

**Moved requirements (part 4).** None; the delta only ADDs.

**Spec soundness (part 5).** Self-consistent. Both scenarios are testable as
written. The spec matches the issue's "Done when": B's key files and config
byte-identical, no new entries, input-dependent. Nothing in the spec is stale
against the issue.

- [x] **none** — no findings; clean areas described above
