//! Creating a Radicle identity, and proving the identities are separate.
//!
//! ## The one assertion that carries this file
//!
//! `two_homes_get_two_different_identities` is the test the whole embedded mode
//! rests on, and it is written the way it is because the obvious version cannot
//! fail. Isolation has an easy false proof: init two profiles, check the two
//! directories differ, check both hold a key. That passes against an
//! implementation with a hardcoded seed — which is not hypothetical, it is
//! exactly what `tests/fixture/mod.rs` does deliberately (`Seed::new([7u8;32])`,
//! fixed so failures reproduce). Two homes, one identity, every
//! directory-shaped assertion green.
//!
//! So the assertion is on the **node ids**, and that they *differ*. That is the
//! input-dependent form CLAUDE.md's branch-switch lesson asks for: a fixture
//! answering the same for every input cannot tell working from broken.
//!
//! Its second half has the same trap in another shape: "each home holds a
//! profile" passes against a creation that overwrote the other home's key. So
//! the other home is recorded, whole and byte for byte, before the second
//! creation, and compared after it.
//!
//! ## Never the developer's real profile
//!
//! Every home here is under this crate's own `tmp/` and removed on drop, via
//! the same `fixture::scratch_dir` the read tests use. Nothing in this file
//! reads `RAD_HOME` or `HOME`, and nothing may: a test that created an identity
//! in a real `~/.radicle` would be writing a permanent key into a directory it
//! then deletes.

mod fixture;

use fixture::{parse, scratch_dir};
use radicle_local_ffi::profileinit::{init_profile, key_encrypted, profile_exists};

/// A scratch home path that does not exist yet — `init_profile` creates it.
///
/// Deliberately a path *inside* a fresh scratch dir rather than the scratch dir
/// itself, so the "creates its own home" behaviour is exercised rather than
/// assumed.
fn fresh_home(name: &str) -> (std::path::PathBuf, String) {
    let dir = scratch_dir(name);
    let home = dir.join("home");
    let s = home.display().to_string();
    (dir, s)
}

fn cleanup(dir: &std::path::Path) {
    let _ = std::fs::remove_dir_all(dir);
}

/// **The isolation test.** Two homes must yield two *different* identities,
/// and creating one must leave the other exactly as it was.
///
/// Asserting on the node ids rather than on the paths is the first half: see
/// this file's header for why the path-shaped version of this test passes
/// against an implementation that gives every profile the same key.
///
/// The second half needs B to exist, populated, *before* A is created. A
/// record of a home that does not exist yet is empty, and an empty record can
/// only see entries appear; it cannot see a rewrite. So B is created first, its
/// whole tree recorded (keys, `config.json`, every database under it), and the
/// record compared after A. Checking only `profile_exists` and that the paths
/// do not nest — what this test used to do — passes against a creation that
/// overwrites the sibling's key pair, which was measured.
///
/// The reverse direction is the absence of A: A's path must not exist until A
/// is created, so creating B cannot have brought it into being.
#[test]
fn two_homes_get_two_different_identities() {
    let (dir_a, home_a) = fresh_home("profile-init-iso-a");
    let (dir_b, home_b) = fresh_home("profile-init-iso-b");
    let path_a = std::path::Path::new(&home_a);
    let path_b = std::path::Path::new(&home_b);

    assert!(
        !home_a.starts_with(&home_b) && !home_b.starts_with(&home_a),
        "precondition: the homes must not nest: {home_a} / {home_b}"
    );
    assert!(
        !path_a.exists(),
        "precondition: home A must not exist before anything is created"
    );

    let b = parse(&init_profile(&home_b, "bob", ""));
    let nid_b = b["nodeId"].as_str().unwrap_or_default().to_string();
    assert!(!nid_b.is_empty(), "home B reported no node id: {b}");
    assert!(
        !path_a.exists(),
        "creating an identity into home B brought home A into being: {home_a}"
    );

    let before = tree_record(path_b);
    for required in ["keys/radicle", "keys/radicle.pub", "config.json"] {
        assert!(
            holds_file(&before, required),
            "precondition: the record of home B must hold {required}, got {:?}",
            before.keys().collect::<Vec<_>>()
        );
    }

    let a = parse(&init_profile(&home_a, "alice", ""));
    let nid_a = a["nodeId"].as_str().unwrap_or_default().to_string();
    assert!(!nid_a.is_empty(), "home A reported no node id: {a}");
    assert_ne!(
        nid_a, nid_b,
        "two embedded profiles share one identity — an embedded node would be \
         indistinguishable from any other, which is the whole property this \
         mode promises. A fixed seed produces exactly this."
    );

    // Checked after the ids, because a shared identity is the failure that
    // matters most, and this would pass regardless of it.
    let changes = record_changes(&before, &tree_record(path_b));
    assert!(
        changes.is_empty(),
        "creating an identity into home A changed home B: {changes:?}"
    );
    assert!(
        profile_exists(&home_a) && profile_exists(&home_b),
        "each home must hold its own profile"
    );

    cleanup(&dir_a);
    cleanup(&dir_b);
}

/// The identity is real, not merely reported: a fresh read of the keystore
/// must agree with what creation claimed.
///
/// Without this, `init_profile` could return a well-formed node id it invented
/// and never wrote, and every other test here would still pass.
#[test]
fn the_reported_identity_is_the_one_actually_on_disk() {
    let (dir, home) = fresh_home("profile-init-readback");

    let created = parse(&init_profile(&home, "tester", ""));
    let claimed = created["nodeId"].as_str().unwrap_or_default();

    // `node_id` is the module's own independent read of `keys/radicle.pub` —
    // the same path `getCapabilities()` reports the identity through, so this
    // also pins that the two agree.
    let read_back = parse(&radicle_local_ffi::env::node_id(&home));

    assert_eq!(
        claimed,
        read_back["nodeId"].as_str().unwrap_or_default(),
        "creation reported an identity the keystore does not hold"
    );

    cleanup(&dir);
}

/// **An existing profile is refused, never overwritten.**
///
/// This is the irreversible one. A second init over a live keystore destroys a
/// signing key that *is* the user's identity, and nothing recovers it.
///
/// ## Why this asserts on the message and not just on failure
///
/// There are **two** guards against a second init: this module's, and the
/// crate's own — `Keystore::init` returns `Error::AlreadyInitialized`
/// (`radicle-crypto-0.19.0/src/ssh/keystore.rs:121-133`). So "the second call
/// errored" is true with our guard deleted, and an earlier version of this test
/// asserted exactly that and passed against a build with no guard at all.
///
/// Asserting that the error *names the home* does not separate them either:
/// the crate's message embeds the offending file path, which contains the home.
/// That is the same-answer-for-every-input trap CLAUDE.md documents, on the one
/// operation in this module that cannot be undone.
///
/// What distinguishes them is the wording, so that is what is asserted. Ours
/// names the consequence ("would overwrite its signing key"); the crate's
/// describes a file ("keystore already initialized"). Verified by mutation:
/// removing our guard makes this test fail while the other eight stay green.
#[test]
fn a_second_init_is_refused_and_leaves_the_first_identity_intact() {
    let (dir, home) = fresh_home("profile-init-refuse");

    let first = parse(&init_profile(&home, "tester", ""));
    let original = first["nodeId"].as_str().unwrap_or_default().to_string();
    assert!(!original.is_empty(), "precondition: a profile was created");

    let second = parse(&init_profile(&home, "someone-else", ""));
    let message = second["error"].as_str().unwrap_or_default();
    assert!(
        !message.is_empty(),
        "a second init must be refused, got: {second}"
    );
    assert!(
        message.contains(&home),
        "the refusal must name the home that was in the way, got: {second}"
    );
    // The part only THIS module's guard produces. Without it the crate refuses
    // too, with a message about a keystore file — a correct refusal, but one
    // that arrives after `Home::new` has already built the directory tree, and
    // one that does not tell the user what was at stake.
    assert!(
        message.contains("would overwrite its signing key"),
        "the refusal must be this module's, which fires before any directory \
         is created and names the consequence — a message about a keystore \
         file is the crate's guard, reached only because ours was removed. \
         Got: {second}"
    );

    // The assertion that actually matters: the key is untouched. A refusal that
    // reported an error *after* rewriting the keystore would pass the check
    // above and still have destroyed the identity.
    let after = parse(&radicle_local_ffi::env::node_id(&home));
    assert_eq!(
        original,
        after["nodeId"].as_str().unwrap_or_default(),
        "the refused init changed the identity on disk"
    );

    cleanup(&dir);
}

/// A home that exists but holds no keys is a legitimate place to create one.
///
/// This is not a corner case: the embedded home is a directory this module
/// creates and may already have made — for settings, or on a run that failed
/// between `mkdir` and keygen. Treating "the directory is there" as "a profile
/// is there" would make such a setup permanently uncompletable, with an error
/// blaming a profile that does not exist.
#[test]
fn an_empty_directory_is_not_mistaken_for_an_existing_profile() {
    let (dir, home) = fresh_home("profile-init-empty-dir");
    std::fs::create_dir_all(&home).expect("could not pre-create the home");

    assert!(
        !profile_exists(&home),
        "an empty directory must not read as a profile"
    );

    let created = parse(&init_profile(&home, "tester", ""));
    assert!(
        created["created"].as_bool().unwrap_or(false),
        "creating into an existing empty directory must work, got: {created}"
    );

    cleanup(&dir);
}

/// An empty passphrase means an unencrypted key, and the reply says so.
///
/// The pair below is the point: both directions are asserted, because a
/// reply that hardcoded `encrypted` either way would pass one of them alone.
#[test]
fn an_empty_passphrase_yields_an_unencrypted_key_and_reports_it() {
    let (dir, home) = fresh_home("profile-init-plain");

    let created = parse(&init_profile(&home, "tester", ""));
    assert_eq!(
        created["encrypted"],
        serde_json::json!(false),
        "an empty passphrase must be reported as unencrypted: {created}"
    );

    // Reported and true: the module's own write path can load a signer with no
    // prompt, which is only possible against a plaintext keystore.
    let can = parse(&radicle_local_ffi::cobwrite::can_write(&home));
    assert_eq!(
        can["canWrite"],
        serde_json::json!(true),
        "an unencrypted key must be immediately signable: {can}"
    );

    cleanup(&dir);
}

/// And a passphrase actually encrypts. Asserted through `can_write`, which
/// needs the *private* half — the only observation that distinguishes an
/// encrypted keystore from a plaintext one from outside.
///
/// This is the half that proves `encrypted` is derived rather than echoed.
#[test]
fn a_passphrase_encrypts_the_key_and_reports_it() {
    let (dir, home) = fresh_home("profile-init-encrypted");

    let created = parse(&init_profile(&home, "tester", "correct horse battery"));
    assert_eq!(
        created["encrypted"],
        serde_json::json!(true),
        "a non-empty passphrase must be reported as encrypted: {created}"
    );

    // With no RAD_PASSPHRASE and no agent holding this brand-new key, a signer
    // cannot be loaded — which is exactly what "encrypted" has to mean for it
    // to be worth reporting.
    let can = parse(&radicle_local_ffi::cobwrite::can_write(&home));
    assert_eq!(
        can["canWrite"],
        serde_json::json!(false),
        "an encrypted key must not be signable without its passphrase: {can}"
    );

    cleanup(&dir);
}

/// The created profile is a real one the existing read path can open.
///
/// `init_profile` returning success while producing something `open_storage`
/// refuses would be a profile in name only, and every assertion above would
/// still pass. This is the cheapest end-to-end statement that it is not.
#[test]
fn the_created_profile_is_readable_by_the_existing_local_path() {
    let (dir, home) = fresh_home("profile-init-readable");

    let created = parse(&init_profile(&home, "tester", ""));
    assert!(created["created"].as_bool().unwrap_or(false), "{created}");

    // An empty storage lists nothing rather than failing — the same assertion
    // `local_storage.rs` makes against a fixture-built profile. Reaching it
    // through a profile this module created is the point.
    let repos = parse(&radicle_local_ffi::local::list_repos(&home, "all", 1, 10));
    assert!(
        repos["error"].is_null(),
        "the created profile must be readable, got: {repos}"
    );
    assert_eq!(
        repos["items"].as_array().map(|a| a.len()),
        Some(0),
        "a fresh profile holds no repositories: {repos}"
    );

    cleanup(&dir);
}

/// A bad alias is refused *before* anything is created.
///
/// Asserted by the absence of a home afterwards, not by the error text alone:
/// an implementation that created the home, then validated, then errored would
/// report the same message and leave a half-made profile behind for the next
/// attempt to trip over.
#[test]
fn a_bad_alias_is_refused_and_creates_nothing() {
    let (dir, home) = fresh_home("profile-init-bad-alias");

    let out = parse(&init_profile(&home, "not a valid alias", ""));
    assert!(
        out["error"].is_string(),
        "an alias with whitespace must be refused, got: {out}"
    );
    assert!(
        !std::path::Path::new(&home).exists(),
        "a refused init must not leave a home behind"
    );

    cleanup(&dir);
}

// ---------------------------------------------------------------------------
// The half-created home.
//
// `Profile::init` writes the keystore first and then runs seven more fallible
// steps (radicle-0.25.1/src/profile.rs:240-266). Any of them failing leaves key
// files on disk with no profile around them. A two-state check would call that
// home occupied for ever, and with no `force` it would be permanently
// uncompletable — the guard protecting a stub that was never an identity.
//
// The fixture builds that state directly rather than trying to make a real init
// fail partway: forcing a failure at, say, `database_mut` needs a filesystem
// the test cannot arrange portably, while the state itself is exactly "key
// files present, config.json absent" and is fully specified by that sentence.
// ---------------------------------------------------------------------------

/// Build the state a crashed init leaves: key material, no `config.json`.
///
/// Made by creating a real profile and then deleting `config.json`, rather than
/// by writing plausible bytes into `keys/` — so the key files are genuinely the
/// ones `Keystore::init` produces, and the crate's own guard behaves exactly as
/// it would in the real failure.
fn half_created_home(name: &str) -> (std::path::PathBuf, String) {
    let (dir, home) = fresh_home(name);
    let created = parse(&init_profile(&home, "tester", ""));
    assert!(
        created["created"].as_bool().unwrap_or(false),
        "fixture precondition: a profile was created, got: {created}"
    );
    std::fs::remove_file(std::path::Path::new(&home).join("config.json"))
        .expect("fixture: could not remove config.json");
    (dir, home)
}

/// **A half-created home is not reported as an occupied one.**
///
/// The distinguishing assertion is on the message, for the same reason the
/// duplicate-init test asserts on wording: a collapsed two-state version still
/// errors here, so "it failed" cannot tell the two apart. Verified by mutation
/// — making `home_state` return `Complete` whenever keys exist makes this fail.
#[test]
fn a_half_created_home_is_reported_as_recoverable_not_as_an_existing_identity() {
    let (dir, home) = half_created_home("profile-init-partial");

    let out = parse(&init_profile(&home, "tester", ""));
    let message = out["error"].as_str().unwrap_or_default();

    assert!(!message.is_empty(), "a partial home must be refused: {out}");
    assert!(
        message.contains("half-created"),
        "a half-created home must be named as such, not reported as an existing \
         identity — the key here was never usable and nothing ever signed with \
         it. Got: {out}"
    );
    assert!(
        !message.contains("would overwrite its signing key"),
        "a half-created home must NOT claim a signing key is at stake: that is \
         both false and unactionable, and it is what a two-state check produces. \
         Got: {out}"
    );
    // The message has to be actionable, which means naming the path to remove.
    assert!(
        message.contains("keys"),
        "the message must name what to remove, got: {out}"
    );

    cleanup(&dir);
}

/// And the state is genuinely reachable through the public classifier, not just
/// an artefact of how the error is worded.
#[test]
fn a_half_created_home_does_not_read_as_an_existing_profile() {
    let (dir, home) = half_created_home("profile-init-partial-classify");

    assert!(
        !profile_exists(&home),
        "a home with key material but no completed init is not a profile — \
         reporting it as one is what makes it permanently uncompletable"
    );

    cleanup(&dir);
}

/// The recovery actually works: remove what the message names, and creation
/// succeeds.
///
/// This is the assertion that makes the error's claim ("this is recoverable")
/// true rather than merely reassuring. Without it the message could promise a
/// recovery that does not exist.
#[test]
fn removing_the_key_material_the_message_names_makes_the_home_usable_again() {
    let (dir, home) = half_created_home("profile-init-partial-recover");

    std::fs::remove_dir_all(std::path::Path::new(&home).join("keys"))
        .expect("could not remove the keys directory the message names");

    let created = parse(&init_profile(&home, "tester", ""));
    assert!(
        created["created"].as_bool().unwrap_or(false),
        "after removing the stale key material the home must be usable, got: \
         {created}"
    );
    assert!(
        !created["nodeId"].as_str().unwrap_or_default().is_empty(),
        "and must yield a real identity: {created}"
    );

    cleanup(&dir);
}

/// `storage/` cannot be the completeness marker, and this pins why.
///
/// `Home::new` creates all four subdirectories — `storage`, `keys`, `node`,
/// `cobs` — before any key is written (`profile.rs:595-599`). So `storage/` is
/// present in the half-created state too, and a version of `home_state` keyed
/// on it would classify every broken home as complete: the exact bug, restored.
///
/// Asserting the directory's presence directly means this fails loudly if a
/// future crate version reorders creation, rather than the marker choice
/// quietly becoming arbitrary.
#[test]
fn a_half_created_home_still_has_storage_which_is_why_config_is_the_marker() {
    let (dir, home) = half_created_home("profile-init-partial-markers");

    assert!(
        std::path::Path::new(&home).join("storage").exists(),
        "precondition: Home::new creates storage/ before keygen, so it cannot \
         distinguish a finished profile from a half-created one"
    );
    assert!(
        !std::path::Path::new(&home).join("config.json").exists(),
        "precondition: config.json is the marker and is absent here"
    );

    cleanup(&dir);
}

/// A relative home is refused rather than resolved against the process CWD.
///
/// Symmetric with `env.rs`'s relative-git-path refusal, and for a stronger
/// reason: this writes a permanent signing key, and a Basecamp-launched
/// module's working directory is not something the user chose or can see.
///
/// The assertion is that **nothing was created anywhere** — not merely that an
/// error came back. A guard that errored after `Home::new` would leave a
/// profile at whatever the CWD happened to be, which is precisely the failure,
/// and "it returned an error" would not notice.
#[test]
fn a_relative_home_is_refused_rather_than_resolved_against_the_working_directory() {
    let cwd = std::env::current_dir().expect("no working directory");
    let relative = "tmp/profile-init-relative-must-not-appear";

    let out = parse(&init_profile(relative, "tester", ""));
    assert!(
        out["error"].is_string(),
        "a relative home must be refused, got: {out}"
    );
    assert!(
        out["error"].as_str().unwrap().contains("absolute"),
        "the refusal must say what was wrong with it, got: {out}"
    );
    assert!(
        !cwd.join(relative).exists(),
        "a refused init resolved the relative path anyway and created {}",
        cwd.join(relative).display()
    );
}

// ---------------------------------------------------------------------------
// Reporting whether an EXISTING key is encrypted.
//
// `init_profile`'s reply already carries `encrypted`, and it is computed from
// the passphrase argument — which makes it useless to any later session, since
// no later session has that reply. `key_encrypted` observes the key on disk
// instead, and these tests are written to tell the two apart: every one of
// them reads a home some OTHER call created, with the creating reply thrown
// away.
// ---------------------------------------------------------------------------

/// Both directions, observed from disk rather than echoed from an argument.
///
/// The two homes are created in the same test and their replies discarded, so
/// an implementation that remembered what it was told has nothing to remember
/// from. A probe hardcoding either answer fails one leg.
#[test]
fn the_reported_encryption_follows_the_key_that_was_written() {
    let (dir_plain, home_plain) = fresh_home("key-encrypted-plain");
    let (dir_sealed, home_sealed) = fresh_home("key-encrypted-sealed");

    // The creating replies are deliberately dropped: what is under test is what
    // a session that never saw them can still learn.
    let _ = init_profile(&home_plain, "tester", "");
    let _ = init_profile(&home_sealed, "tester", "correct horse battery");

    let plain = parse(&key_encrypted(&home_plain));
    assert_eq!(
        plain["encrypted"],
        serde_json::json!(false),
        "a key written with an empty passphrase must read back as \
         unencrypted: {plain}"
    );
    assert_eq!(
        plain["problem"],
        serde_json::json!(""),
        "and with no problem to report: {plain}"
    );

    let sealed = parse(&key_encrypted(&home_sealed));
    assert_eq!(
        sealed["encrypted"],
        serde_json::json!(true),
        "a key written with a passphrase must read back as encrypted: {sealed}"
    );
    assert_eq!(
        sealed["problem"],
        serde_json::json!(""),
        "and with no problem to report: {sealed}"
    );

    cleanup(&dir_plain);
    cleanup(&dir_sealed);
}

/// The probe needs no passphrase, no `RAD_PASSPHRASE` and no ssh-agent.
///
/// This is the property that makes the field askable at all. `can_write` is the
/// control: it needs the *private* half, so it fails against this same home in
/// this same environment — which is what proves the probe is not quietly
/// unlocking anything.
#[test]
fn reporting_encryption_needs_no_passphrase_and_no_agent() {
    let (dir, home) = fresh_home("key-encrypted-no-secret");
    let _ = init_profile(&home, "tester", "correct horse battery");

    let probed = parse(&key_encrypted(&home));
    assert_eq!(
        probed["encrypted"],
        serde_json::json!(true),
        "an encrypted key must be reported as such without unlocking it: \
         {probed}"
    );
    assert!(
        probed["error"].is_null(),
        "the question was answered, so it must not come back as an error \
         object: {probed}"
    );

    // The control. If this passed, the environment would be holding the key
    // open and the assertion above would prove nothing about secrets.
    let can = parse(&radicle_local_ffi::cobwrite::can_write(&home));
    assert_eq!(
        can["canWrite"],
        serde_json::json!(false),
        "precondition: nothing in this environment can unlock the key, so \
         the probe above cannot have been unlocking it: {can}"
    );

    cleanup(&dir);
}

// ---------------------------------------------------------------------------
// Recording a directory tree, for "nothing under here changed".
//
// Two tests need that statement: a report leaves the sealed key as it was, and
// creating into one home leaves another as it was. Both are answered by the
// same record, so there is one recorder rather than one per depth.
// ---------------------------------------------------------------------------

/// One entry under a recorded directory.
///
/// A directory is an entry in its own right, so an empty directory created or
/// removed under the root changes the record. A symlink is recorded by its
/// target and never followed, so the record cannot read outside the root it
/// was asked about.
#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Dir,
    File(Vec<u8>),
    Link(std::path::PathBuf),
}

type TreeRecord = std::collections::BTreeMap<std::path::PathBuf, Entry>;

/// Every entry under `root`, recursively, by its path relative to `root`, with
/// the bytes of every file.
///
/// Names are part of the record, so an entry added or removed changes the map
/// as well as a file whose bytes changed. An entry that cannot be read panics
/// rather than being skipped: a record with a hole in it would compare equal
/// across exactly the change it was meant to see.
fn tree_record(root: &std::path::Path) -> TreeRecord {
    let mut out = TreeRecord::new();
    record_under(root, std::path::Path::new(""), &mut out);
    out
}

fn record_under(root: &std::path::Path, rel: &std::path::Path, out: &mut TreeRecord) {
    let dir = root.join(rel);
    let entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("could not list {} for the record: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("unreadable entry in {}: {e}", dir.display()));
        let path = rel.join(entry.file_name());
        // `DirEntry::file_type` does not follow symlinks, which is what keeps
        // the record inside `root`.
        let kind = entry
            .file_type()
            .unwrap_or_else(|e| panic!("no file type for {}: {e}", path.display()));
        if kind.is_symlink() {
            let target = std::fs::read_link(entry.path())
                .unwrap_or_else(|e| panic!("unreadable link {}: {e}", path.display()));
            out.insert(path, Entry::Link(target));
        } else if kind.is_dir() {
            out.insert(path.clone(), Entry::Dir);
            record_under(root, &path, out);
        } else {
            let bytes = std::fs::read(entry.path())
                .unwrap_or_else(|e| panic!("unreadable file {}: {e}", path.display()));
            out.insert(path, Entry::File(bytes));
        }
    }
}

/// What differs between two records, one line per path, empty when they agree.
///
/// Compared this way rather than with `assert_eq!` on the maps, whose failure
/// would print every byte of every SQLite file in a home and bury the one path
/// that moved.
fn record_changes(before: &TreeRecord, after: &TreeRecord) -> Vec<String> {
    let mut changes = Vec::new();
    for (path, was) in before {
        match after.get(path) {
            None => changes.push(format!("removed: {}", path.display())),
            Some(now) if now != was => changes.push(format!("changed: {}", path.display())),
            Some(_) => {}
        }
    }
    for path in after.keys().filter(|p| !before.contains_key(*p)) {
        changes.push(format!("added: {}", path.display()));
    }
    changes
}

/// Whether `record` holds `rel` as a file, for the preconditions that stop two
/// empty records comparing equal.
fn holds_file(record: &TreeRecord, rel: &str) -> bool {
    matches!(record.get(std::path::Path::new(rel)), Some(Entry::File(_)))
}

/// A small tree with every kind of entry the recorder distinguishes, built by
/// hand so its expected record is written down here and not derived from the
/// recorder.
///
/// ```text
/// a/b/deep.txt   "one"
/// empty/         (an empty directory)
/// top.bin        0x00 0x01
/// ```
fn recorder_tree(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let dir = scratch_dir(name);
    let root = dir.join("root");
    std::fs::create_dir_all(root.join("a").join("b")).expect("fixture: a/b");
    std::fs::create_dir(root.join("empty")).expect("fixture: empty");
    std::fs::write(root.join("a").join("b").join("deep.txt"), b"one").expect("fixture: deep.txt");
    std::fs::write(root.join("top.bin"), [0u8, 1u8]).expect("fixture: top.bin");
    (dir, root)
}

fn sorted(mut lines: Vec<String>) -> Vec<String> {
    lines.sort();
    lines
}

/// The recorder lists every entry, at every depth, directories included.
///
/// The two isolation tests are only as sharp as this instrument: a recorder
/// that stopped at the first level, or left directories out, would compare
/// equal across a write into `cobs/` or the removal of an empty `storage/`, and
/// those tests would stay green. The expected record is spelled out by hand.
#[test]
fn the_recorder_lists_every_entry_at_every_depth_with_directories() {
    let (dir, root) = recorder_tree("tree-record-shape");

    let record = tree_record(&root);

    let paths: Vec<String> = record.keys().map(|p| p.display().to_string()).collect();
    assert_eq!(
        paths,
        ["a", "a/b", "a/b/deep.txt", "empty", "top.bin"],
        "the record must list directories as entries and descend into them"
    );
    assert_eq!(
        record[std::path::Path::new("a/b/deep.txt")],
        Entry::File(b"one".to_vec())
    );
    assert_eq!(
        record[std::path::Path::new("top.bin")],
        Entry::File(vec![0, 1])
    );
    assert_eq!(record[std::path::Path::new("empty")], Entry::Dir);
    assert!(holds_file(&record, "a/b/deep.txt"));
    assert!(
        !holds_file(&record, "a/b"),
        "a directory is not a file, even one with files under it"
    );
    assert!(!holds_file(&record, "missing"));

    cleanup(&dir);
}

/// Each kind of change is reported, by path, under its own word.
///
/// A table, because the failure each row guards against is a different branch
/// of `record_changes`: dropping the `changed` arm, the `removed` arm or the
/// `added` loop each turns exactly one row red.
#[test]
fn the_recorder_names_each_kind_of_change_by_path() {
    type Edit = fn(&std::path::Path);
    let cases: [(&str, Edit, &[&str]); 7] = [
        ("nothing is touched", |_| {}, &[]),
        (
            "a deep file's bytes change",
            |r| std::fs::write(r.join("a/b/deep.txt"), b"two").unwrap(),
            &["changed: a/b/deep.txt"],
        ),
        (
            "a file is removed",
            |r| std::fs::remove_file(r.join("top.bin")).unwrap(),
            &["removed: top.bin"],
        ),
        (
            "an empty directory is removed",
            |r| std::fs::remove_dir(r.join("empty")).unwrap(),
            &["removed: empty"],
        ),
        (
            "an empty directory is added",
            |r| std::fs::create_dir(r.join("fresh")).unwrap(),
            &["added: fresh"],
        ),
        (
            "a file is added deep down",
            |r| std::fs::write(r.join("a/b/extra"), b"").unwrap(),
            &["added: a/b/extra"],
        ),
        (
            "one of each at once",
            |r| {
                std::fs::write(r.join("a/b/deep.txt"), b"two").unwrap();
                std::fs::remove_dir(r.join("empty")).unwrap();
                std::fs::write(r.join("new"), b"x").unwrap();
            },
            &["changed: a/b/deep.txt", "removed: empty", "added: new"],
        ),
    ];

    for (i, (what, edit, expected)) in cases.iter().enumerate() {
        let (dir, root) = recorder_tree(&format!("tree-record-changes-{i}"));
        let before = tree_record(&root);
        edit(&root);
        let got = sorted(record_changes(&before, &tree_record(&root)));
        let want = sorted(expected.iter().map(|s| s.to_string()).collect());
        assert_eq!(got, want, "case {i}: {what}");
        cleanup(&dir);
    }
}

/// A symlink is recorded by its target and never followed.
///
/// What it must not do is read through the link: a link out of the root would
/// otherwise pull the outside tree into the record, so the record of one home
/// could change when another does.
#[cfg(unix)]
#[test]
fn the_recorder_does_not_follow_a_symlink_out_of_its_root() {
    let (dir, root) = recorder_tree("tree-record-link");
    let outside = dir.join("outside");
    std::fs::create_dir(&outside).expect("fixture: outside");
    std::fs::write(outside.join("secret"), b"not under root").expect("fixture: secret");
    std::os::unix::fs::symlink(&outside, root.join("link")).expect("fixture: link");

    let before = tree_record(&root);
    assert_eq!(
        before[std::path::Path::new("link")],
        Entry::Link(outside.clone()),
        "a link is recorded as its target"
    );
    assert!(
        !before.contains_key(std::path::Path::new("link/secret")),
        "the record read through a symlink: {:?}",
        before.keys().collect::<Vec<_>>()
    );

    // Changing what the link points at is a change; changing what is behind it
    // is not one this root can see.
    std::fs::write(outside.join("secret"), b"rewritten").unwrap();
    assert!(record_changes(&before, &tree_record(&root)).is_empty());

    std::fs::remove_file(root.join("link")).unwrap();
    std::os::unix::fs::symlink(root.join("empty"), root.join("link")).unwrap();
    assert_eq!(
        record_changes(&before, &tree_record(&root)),
        ["changed: link"],
        "retargeting a link must show as a change"
    );

    cleanup(&dir);
}

/// Reading the key does not change it — any of it.
///
/// A probe that unsealed and rewrote the key would report the right answer and
/// destroy the property it reported. The bytes of every file under `keys` are
/// compared before and after, which is the only observation that would notice.
///
/// The record is asserted to hold the private key AND the public one before
/// anything is probed, so a comparison of two empty maps cannot pass for a
/// home whose keys directory was never populated.
///
/// Covers the whole read side of `getEmbeddedIdentity()` that touches this
/// directory — the encryption probe and the existence probe — so the scenario
/// "a report leaves the sealed key as it was" is asked of both.
#[test]
fn reporting_encryption_leaves_the_key_untouched() {
    let (dir, home) = fresh_home("key-encrypted-immutable");
    let _ = init_profile(&home, "tester", "correct horse battery");

    let keys = std::path::Path::new(&home).join("keys");
    let before = tree_record(&keys);
    assert!(
        holds_file(&before, "radicle") && holds_file(&before, "radicle.pub"),
        "precondition: the record must hold both halves of the key, got {:?}",
        before.keys().collect::<Vec<_>>()
    );

    let probed = parse(&key_encrypted(&home));
    assert_eq!(probed["encrypted"], serde_json::json!(true), "{probed}");
    assert!(
        profile_exists(&home),
        "precondition: the existence probe must see the identity it reports on"
    );

    let changes = record_changes(&before, &tree_record(&keys));
    assert!(
        changes.is_empty(),
        "reporting whether the key is encrypted must not rewrite, remove or add \
         any file under keys/, got: {changes:?}"
    );

    cleanup(&dir);
}

/// An unreadable key is NOT an unencrypted one, and the two must not be
/// reported alike.
///
/// This is the requirement that a `Result` collapsed with `unwrap_or(false)`
/// would silently break: the answer would be `encrypted:false` with nothing
/// saying the question was never answered, and a caller would start a node with
/// an empty passphrase against a key it cannot read.
///
/// The key file is replaced with a directory, which `Keystore::is_encrypted`
/// cannot parse and cannot be mistaken for a plaintext key.
#[test]
fn an_unreadable_key_reports_a_problem_rather_than_unencrypted() {
    let (dir, home) = fresh_home("key-encrypted-unreadable");
    let _ = init_profile(&home, "tester", "");

    let key = std::path::Path::new(&home).join("keys").join("radicle");
    std::fs::remove_file(&key).expect("the private key must exist to replace");
    std::fs::create_dir(&key).expect("could not put a directory in its place");

    let probed = parse(&key_encrypted(&home));
    assert_eq!(
        probed["encrypted"],
        serde_json::json!(false),
        "an unreadable key must not claim to be encrypted: {probed}"
    );
    assert!(
        !probed["problem"].as_str().unwrap_or_default().is_empty(),
        "and must say the question could not be answered, or it is \
         indistinguishable from a plaintext key: {probed}"
    );

    cleanup(&dir);
}

/// A home with no key at all reports a problem too, for the same reason.
///
/// `getEmbeddedIdentity()` only asks this where `exists` is true, so this is
/// the defence in depth rather than the ordinary path — but "no key" answering
/// `false` with an empty `problem` would be a plaintext key as far as any
/// caller could tell.
#[test]
fn a_home_with_no_key_reports_a_problem_rather_than_unencrypted() {
    let (dir, home) = fresh_home("key-encrypted-absent");
    std::fs::create_dir_all(&home).expect("could not create the empty home");

    let probed = parse(&key_encrypted(&home));
    assert_eq!(probed["encrypted"], serde_json::json!(false), "{probed}");
    assert!(
        !probed["problem"].as_str().unwrap_or_default().is_empty(),
        "a home with no key must say so rather than read as plaintext: \
         {probed}"
    );

    cleanup(&dir);
}

/// The FFI boundary answers with JSON rather than unwinding.
///
/// `panic_guard.rs` covers the read and write entry points on the same
/// principle; this adds the one that *creates* state, where a panic would land
/// mid-keygen with a half-written keystore on disk.
#[test]
fn the_init_entry_point_is_guarded() {
    use std::ffi::{CStr, CString};

    // A NUL home reaches `read_str`'s empty-string path; a home under a path
    // that cannot be created exercises the crate's own error path. Neither may
    // cross the boundary as a panic.
    let bad = CString::new("/proc/nonexistent-cannot-create/home").unwrap();
    let alias = CString::new("tester").unwrap();
    let pass = CString::new("").unwrap();

    for home in [std::ptr::null(), bad.as_ptr()] {
        let raw = unsafe {
            radicle_local_ffi::radicle_local_init_profile(home, alias.as_ptr(), pass.as_ptr())
        };
        assert!(!raw.is_null());
        let text = unsafe { CStr::from_ptr(raw) }
            .to_string_lossy()
            .into_owned();
        unsafe { radicle_local_ffi::radicle_free_string(raw) };

        let v: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("not JSON across the boundary: {e}\n{text}"));
        assert!(
            v["error"].is_string(),
            "an unusable home must come back as an error object, got: {v}"
        );
    }
}
