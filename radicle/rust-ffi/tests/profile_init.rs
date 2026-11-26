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
//! ## Never the developer's real profile
//!
//! Every home here is under this crate's own `tmp/` and removed on drop, via
//! the same `fixture::scratch_dir` the read tests use. Nothing in this file
//! reads `RAD_HOME` or `HOME`, and nothing may: a test that created an identity
//! in a real `~/.radicle` would be writing a permanent key into a directory it
//! then deletes.

mod fixture;

use fixture::{parse, scratch_dir};
use radicle_local_ffi::profileinit::{
    home_state, init_profile, init_profile_with_entropy, key_encrypted, profile_exists, HomeState,
};

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

/// **The isolation test.** Two homes must yield two *different* identities.
///
/// Asserting on the node ids rather than on the paths is the whole point: see
/// this file's header for why the path-shaped version of this test passes
/// against an implementation that gives every profile the same key.
#[test]
fn two_homes_get_two_different_identities() {
    let (dir_a, home_a) = fresh_home("profile-init-iso-a");
    let (dir_b, home_b) = fresh_home("profile-init-iso-b");

    let a = parse(&init_profile(&home_a, "alice", ""));
    let b = parse(&init_profile(&home_b, "bob", ""));

    let nid_a = a["nodeId"].as_str().unwrap_or_default().to_string();
    let nid_b = b["nodeId"].as_str().unwrap_or_default().to_string();

    assert!(!nid_a.is_empty(), "first profile reported no node id: {a}");
    assert!(!nid_b.is_empty(), "second profile reported no node id: {b}");
    assert_ne!(
        nid_a, nid_b,
        "two embedded profiles share one identity — an embedded node would be \
         indistinguishable from any other, which is the whole property this \
         mode promises. A fixed seed produces exactly this."
    );

    // And neither wrote into the other. Checked after the ids, because a shared
    // identity is the failure that matters and this would pass regardless of it.
    assert!(
        profile_exists(&home_a) && profile_exists(&home_b),
        "each home must hold its own profile"
    );
    assert!(
        !home_a.starts_with(&home_b) && !home_b.starts_with(&home_a),
        "the homes must not nest: {home_a} / {home_b}"
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

/// Every file under `<home>/keys`, by name, with its bytes.
///
/// A map rather than the one private-key file: the requirement is that **every
/// file under the `keys` directory** is byte-for-byte what it was, so a probe
/// that left `keys/radicle` alone and rewrote `keys/radicle.pub` (or dropped a
/// sibling file next to them) must be seen. Names are part of the snapshot, so
/// a file created or removed by the probe changes the map as well.
fn keys_snapshot(home: &str) -> std::collections::BTreeMap<String, Vec<u8>> {
    let keys = std::path::Path::new(home).join("keys");
    let mut out = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(&keys).expect("the keys directory must exist") {
        let entry = entry.expect("a keys entry must be readable");
        let name = entry.file_name().to_string_lossy().into_owned();
        let bytes = std::fs::read(entry.path()).expect("a keys file must be readable");
        out.insert(name, bytes);
    }
    out
}

/// Reading the key does not change it — any of it.
///
/// A probe that unsealed and rewrote the key would report the right answer and
/// destroy the property it reported. The bytes of every file under `keys` are
/// compared before and after, which is the only observation that would notice.
///
/// The snapshot is asserted to hold the private key AND the public one before
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

    let before = keys_snapshot(&home);
    assert!(
        before.contains_key("radicle") && before.contains_key("radicle.pub"),
        "precondition: the snapshot must hold both halves of the key, got {:?}",
        before.keys().collect::<Vec<_>>()
    );

    let probed = parse(&key_encrypted(&home));
    assert_eq!(probed["encrypted"], serde_json::json!(true), "{probed}");
    assert!(
        profile_exists(&home),
        "precondition: the existence probe must see the identity it reports on"
    );

    let after = keys_snapshot(&home);
    assert_eq!(
        before, after,
        "reporting whether the key is encrypted must not rewrite, remove or add \
         any file under keys/"
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

// ---------------------------------------------------------------------------
// The entropy draw.
//
// `/dev/urandom` cannot be made to short-read, fail or chunk on demand, so the
// production path alone can never show what creation does with a source that
// misbehaves. These tests reach the same code through
// `init_profile_with_entropy`, handing it a scripted reader.
//
// The mutation this section exists to kill is the one the embedded node's
// first review found surviving: `read_exact` replaced by a single `read` that
// discards the count. A 16-byte short read then leaves half the seed zeroed,
// and creation reports success with a well-formed DID — the isolation test
// still passes, because two half-zeroed seeds still differ in their random
// half.
// ---------------------------------------------------------------------------

/// What every failed draw must say, per "A failed entropy draw creates no key".
const ENTROPY_UNREADABLE: &str = "the entropy for a new key could not be read";

/// One thing a scripted entropy source does on its next read.
enum Step {
    /// Supply these bytes, as many as fit the caller's buffer; the rest wait
    /// for the next read.
    Give(Vec<u8>),
    /// Report `ErrorKind::Interrupted`, as a read cut short by a signal does.
    Interrupt,
    /// Report a hard read error.
    Fail,
    /// Report an error of kind `UnexpectedEof` with its own text — the kind
    /// `read_exact` uses for a source that ran out, raised by the source
    /// itself instead.
    FailAsEndOfFile,
}

/// A reader that does exactly what its script says, then reports end of input.
struct Scripted(std::collections::VecDeque<Step>);

impl Scripted {
    fn new(steps: Vec<Step>) -> Self {
        Self(steps.into())
    }

    /// The same bytes, one per read.
    fn one_byte_at_a_time(bytes: &[u8]) -> Self {
        Self::new(bytes.iter().map(|b| Step::Give(vec![*b])).collect())
    }
}

impl std::io::Read for Scripted {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.0.pop_front() {
            None => Ok(0),
            Some(Step::Interrupt) => Err(std::io::ErrorKind::Interrupted.into()),
            Some(Step::Fail) => Err(std::io::Error::other("the test source failed on purpose")),
            Some(Step::FailAsEndOfFile) => Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "the test source reported its own end-of-file error",
            )),
            Some(Step::Give(mut bytes)) => {
                let n = bytes.len().min(buf.len());
                buf[..n].copy_from_slice(&bytes[..n]);
                if n < bytes.len() {
                    self.0.push_front(Step::Give(bytes.split_off(n)));
                }
                Ok(n)
            }
        }
    }
}

/// An opener that hands over `source` when the draw asks for it.
fn opening(source: Scripted) -> impl FnOnce() -> std::io::Result<Scripted> {
    move || Ok(source)
}

/// 32 bytes that are all different, so a seed assembled out of order, or with
/// any byte dropped or zeroed, is a different seed.
fn known_seed() -> Vec<u8> {
    (1u8..=32).map(|i| i.wrapping_mul(37)).collect()
}

/// The node id the `radicle` crate itself derives from `bytes`, through
/// `Profile::init` directly — not through the code under test. That is what
/// makes it an oracle: a seam that ignored its source and drew from
/// `/dev/urandom`, or used a fixed seed, cannot match it.
fn node_id_the_crate_derives_from(bytes: &[u8], name: &str) -> String {
    let dir = scratch_dir(name);
    let home = radicle::profile::Home::new(dir.join("home")).expect("oracle: Home::new");
    let seed: [u8; 32] = bytes.try_into().expect("oracle: a seed is 32 bytes");
    let profile = radicle::profile::Profile::init(
        home,
        radicle::node::Alias::new("oracle"),
        None,
        radicle::crypto::Seed::new(seed),
    )
    .expect("oracle: Profile::init");
    let did = profile.did().to_string();
    cleanup(&dir);
    did
}

/// The names of the files under `<home>/keys`. An absent directory holds none.
fn files_under_keys(home: &str) -> Vec<String> {
    match std::fs::read_dir(std::path::Path::new(home).join("keys")) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => panic!("the keys directory could not be listed: {e}"),
        Ok(entries) => entries
            .map(|e| {
                e.expect("a keys entry must be readable")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect(),
    }
}

/// **A source that ends after 16 bytes creates no key.**
///
/// The regression test for the surviving mutation: with a lenient `read` the
/// half-filled seed is accepted and this reply reports `created:true`.
#[test]
fn a_source_that_ends_after_16_bytes_creates_no_key() {
    let (dir, home) = fresh_home("entropy-short");
    let half = known_seed()[..16].to_vec();

    let out = parse(&init_profile_with_entropy(
        &home,
        "tester",
        "",
        opening(Scripted::new(vec![Step::Give(half)])),
    ));

    let message = out["error"].as_str().unwrap_or_default();
    assert!(
        message.contains(ENTROPY_UNREADABLE),
        "a short source must fail with the entropy message, got: {out}"
    );
    assert!(
        out.get("created").is_none() && out.get("nodeId").is_none(),
        "a failed draw must not report an identity: {out}"
    );
    assert_eq!(
        files_under_keys(&home),
        Vec::<String>::new(),
        "a failed draw must write no key material"
    );

    cleanup(&dir);
}

/// **A source that fails mid-read creates no key**, and the failure is not
/// mistaken for end of input or retried as an interruption.
#[test]
fn a_source_that_fails_mid_read_creates_no_key() {
    let (dir, home) = fresh_home("entropy-fail");
    let half = known_seed()[..16].to_vec();

    let out = parse(&init_profile_with_entropy(
        &home,
        "tester",
        "",
        opening(Scripted::new(vec![Step::Give(half), Step::Fail])),
    ));

    let message = out["error"].as_str().unwrap_or_default();
    assert!(
        message.contains(ENTROPY_UNREADABLE),
        "a failing source must fail with the entropy message, got: {out}"
    );
    assert!(
        message.contains("failed on purpose"),
        "the source's own error must be carried, so the cause is visible: {out}"
    );
    assert_eq!(
        files_under_keys(&home),
        Vec::<String>::new(),
        "a failed draw must write no key material"
    );

    cleanup(&dir);
}

/// **A read error's text is carried whatever its kind** — including
/// `UnexpectedEof`, the kind `read_exact` itself uses when a source runs out.
///
/// The draw replaces std's own short-read text with a sentence a user can act
/// on. Keyed on the error's kind alone, that replacement would also swallow
/// this source's own words, which "A failed entropy draw creates no key"
/// requires the reply to include.
#[test]
fn a_read_error_of_the_end_of_file_kind_keeps_its_own_text() {
    let (dir, home) = fresh_home("entropy-own-eof");
    let half = known_seed()[..16].to_vec();

    let out = parse(&init_profile_with_entropy(
        &home,
        "tester",
        "",
        opening(Scripted::new(vec![Step::Give(half), Step::FailAsEndOfFile])),
    ));

    let message = out["error"].as_str().unwrap_or_default();
    assert!(
        message.contains(ENTROPY_UNREADABLE),
        "a failing source must fail with the entropy message, got: {out}"
    );
    assert!(
        message.contains("reported its own end-of-file error"),
        "the source's own error must be carried, whatever its kind: {out}"
    );
    assert_eq!(
        files_under_keys(&home),
        Vec::<String>::new(),
        "a failed draw must write no key material"
    );

    cleanup(&dir);
}

/// A source that cannot even be opened fails the same way.
///
/// This is the message the old code got wrong: "could not open the system
/// random source" said nothing about entropy or a key.
#[test]
fn a_source_that_cannot_be_opened_creates_no_key() {
    let (dir, home) = fresh_home("entropy-unopenable");

    let out = parse(&init_profile_with_entropy(&home, "tester", "", || {
        Err::<Scripted, _>(std::io::Error::other("no such source"))
    }));

    let message = out["error"].as_str().unwrap_or_default();
    assert!(
        message.contains(ENTROPY_UNREADABLE),
        "an unopenable source must fail with the entropy message, got: {out}"
    );
    assert!(
        message.contains("no such source"),
        "the opener's own error must be carried: {out}"
    );
    assert_eq!(
        files_under_keys(&home),
        Vec::<String>::new(),
        "a failed draw must write no key material"
    );

    cleanup(&dir);
}

/// **A failed draw leaves the home completable**: it classifies as Empty, and
/// a full source then creates into it.
#[test]
fn a_failed_draw_leaves_the_home_completable() {
    let (dir, home) = fresh_home("entropy-completable");
    let seed = known_seed();

    let failed = parse(&init_profile_with_entropy(
        &home,
        "tester",
        "",
        opening(Scripted::new(vec![Step::Give(seed[..16].to_vec())])),
    ));
    assert!(failed["error"].is_string(), "precondition: {failed}");
    assert_eq!(
        home_state(&home),
        HomeState::Empty,
        "a failed draw must leave the home Empty, not Partial"
    );

    let created = parse(&init_profile_with_entropy(
        &home,
        "tester",
        "",
        opening(Scripted::new(vec![Step::Give(seed)])),
    ));
    assert_eq!(created["created"], serde_json::json!(true), "{created}");
    assert!(
        !created["nodeId"].as_str().unwrap_or_default().is_empty(),
        "the retried creation must yield an identity: {created}"
    );

    cleanup(&dir);
}

/// **A seed supplied in pieces is used whole.** One byte per read and all 32
/// in one read give the same identity — and it is the identity the crate
/// itself derives from those bytes, so neither side can agree by ignoring the
/// source.
///
/// With a single lenient `read`, the one-byte source yields a key from one real
/// byte and 31 zeros, and the ids differ.
#[test]
fn a_seed_supplied_one_byte_per_read_is_used_whole() {
    let (dir_whole, home_whole) = fresh_home("entropy-whole");
    let (dir_bytes, home_bytes) = fresh_home("entropy-bytewise");
    let seed = known_seed();

    let whole = parse(&init_profile_with_entropy(
        &home_whole,
        "tester",
        "",
        opening(Scripted::new(vec![Step::Give(seed.clone())])),
    ));
    let bytewise = parse(&init_profile_with_entropy(
        &home_bytes,
        "tester",
        "",
        opening(Scripted::one_byte_at_a_time(&seed)),
    ));

    assert_eq!(whole["created"], serde_json::json!(true), "{whole}");
    assert_eq!(bytewise["created"], serde_json::json!(true), "{bytewise}");
    assert_eq!(
        whole["nodeId"], bytewise["nodeId"],
        "the same 32 bytes must give the same identity however they arrive"
    );
    assert_eq!(
        whole["nodeId"].as_str().unwrap_or_default(),
        node_id_the_crate_derives_from(&seed, "entropy-whole-oracle"),
        "the key must be derived from exactly the bytes the source supplied"
    );

    cleanup(&dir_whole);
    cleanup(&dir_bytes);
}

/// **An interrupted read is retried, not failed.**
#[test]
fn an_interrupted_read_is_retried_not_failed() {
    let (dir, home) = fresh_home("entropy-interrupted");
    let seed = known_seed();

    let out = parse(&init_profile_with_entropy(
        &home,
        "tester",
        "",
        opening(Scripted::new(vec![
            Step::Give(seed[..16].to_vec()),
            Step::Interrupt,
            Step::Give(seed[16..].to_vec()),
        ])),
    ));

    assert_eq!(
        out["created"],
        serde_json::json!(true),
        "an interruption is not a failure: {out}"
    );
    assert_eq!(
        out["nodeId"].as_str().unwrap_or_default(),
        node_id_the_crate_derives_from(&seed, "entropy-interrupted-oracle"),
        "the bytes either side of the interruption must both reach the key"
    );

    cleanup(&dir);
}

// NO SPEC: the spec requires a failed draw to leave the home Empty, which an
// empty directory tree satisfies; it does not say whether that tree may be
// created. This pins that the draw happens before `Home::new`, so a failed
// draw creates nothing at all — design.md, Decision 3.
#[test]
fn a_failed_draw_creates_no_home_directory() {
    let (dir, home) = fresh_home("entropy-no-home");

    let out = parse(&init_profile_with_entropy(
        &home,
        "tester",
        "",
        opening(Scripted::new(vec![Step::Give(known_seed()[..16].to_vec())])),
    ));
    assert!(out["error"].is_string(), "precondition: {out}");
    assert!(
        !std::path::Path::new(&home).exists(),
        "a failed draw must leave the filesystem as it found it, but {home} \
         was created"
    );

    cleanup(&dir);
}

/// **A source longer than a seed uses its first 32 bytes**: the bytes past the
/// 32nd never reach the key.
#[test]
fn a_source_longer_than_a_seed_uses_its_first_32_bytes() {
    let (dir, home) = fresh_home("entropy-long");
    let seed = known_seed();
    let mut long = seed.clone();
    long.extend(std::iter::repeat_n(0xAAu8, 32));

    let out = parse(&init_profile_with_entropy(
        &home,
        "tester",
        "",
        opening(Scripted::new(vec![Step::Give(long)])),
    ));
    assert_eq!(out["created"], serde_json::json!(true), "{out}");
    assert_eq!(
        out["nodeId"].as_str().unwrap_or_default(),
        node_id_the_crate_derives_from(&seed, "entropy-long-oracle"),
        "the first 32 bytes, and only those, must decide the key"
    );

    cleanup(&dir);
}

/// A refused call never opens the source, so a refusal is never reported as an
/// entropy failure — whichever of the five refusals it is.
///
/// The message half is each refusal's own existing requirement: whatever the
/// source would have done, the caller is told what they got wrong. The `opened`
/// flag is design.md's Decision 2, and it is the half that does the work: an
/// implementation that opens the source before validating but holds the error
/// until the draw still reports the refusal correctly, and only the flag sees
/// it.
///
/// A table rather than the occupied home alone, because the draw sits *after*
/// five checks and a draw moved up past only some of them is a different bug:
/// moved above the alias check, say, it leaves the occupied-home case green and
/// turns the bad-alias case into an entropy error.
// NO SPEC: that the source is not opened for a refused call is design.md's
// Decision 2, not a requirement. The spec requires only that each refusal keep
// its own message; the `opened` assertion pins the stronger ordering.
#[test]
fn a_refused_home_never_opens_the_entropy_source() {
    // (case, the words its own refusal must contain)
    let cases: [(&str, &str); 5] = [
        ("empty-home", "no home given"),
        ("relative-home", "absolute"),
        ("occupied-home", "would overwrite its signing key"),
        ("half-created-home", "half-created"),
        ("bad-alias", "not a usable alias"),
    ];

    for (case, wanted) in cases {
        let (dir, home) = fresh_home(&format!("entropy-refused-{case}"));
        let mut alias = "tester";
        let target = match case {
            "empty-home" => String::new(),
            "relative-home" => "tmp/entropy-refused-must-not-appear".to_string(),
            "occupied-home" => {
                let first = parse(&init_profile(&home, "tester", ""));
                assert_eq!(first["created"], serde_json::json!(true), "{first}");
                home.clone()
            }
            "half-created-home" => {
                let first = parse(&init_profile(&home, "tester", ""));
                assert_eq!(first["created"], serde_json::json!(true), "{first}");
                std::fs::remove_file(std::path::Path::new(&home).join("config.json"))
                    .expect("fixture: could not remove config.json");
                home.clone()
            }
            "bad-alias" => {
                alias = "not a valid alias";
                home.clone()
            }
            other => unreachable!("{other}"),
        };

        let opened = std::cell::Cell::new(false);
        let out = parse(&init_profile_with_entropy(&target, alias, "", || {
            opened.set(true);
            Err::<Scripted, _>(std::io::Error::other("no such source"))
        }));

        let message = out["error"].as_str().unwrap_or_default();
        assert!(
            message.contains(wanted),
            "{case}: must be refused with its own message ({wanted:?}), got: {out}"
        );
        assert!(
            !message.contains(ENTROPY_UNREADABLE),
            "{case}: a refusal must not be reported as an entropy failure: {out}"
        );
        assert!(
            !opened.get(),
            "{case}: the entropy source was opened for a call that was always \
             going to be refused"
        );

        cleanup(&dir);
    }
}

/// **A source that falls short of 32 bytes at any length creates no key**, and
/// one that fails at any length does too.
///
/// The spec's scenarios name 16 bytes. A table, because a draw that accepted
/// "at least half a seed" (or any other threshold) is red only at the lengths
/// above it, and an empty source is the one a real `/dev/urandom` open that
/// succeeds but yields nothing would be.
#[test]
fn a_source_short_of_32_bytes_at_any_length_creates_no_key() {
    let seed = known_seed();

    // (case, how many bytes arrive, then what the source does)
    let cases: [(&str, usize, bool); 6] = [
        ("ends-after-0", 0, false),
        ("ends-after-1", 1, false),
        ("ends-after-31", 31, false),
        ("fails-after-0", 0, true),
        ("fails-after-1", 1, true),
        ("fails-after-31", 31, true),
    ];

    for (case, supplied, fails) in cases {
        let (dir, home) = fresh_home(&format!("entropy-any-{case}"));
        let mut steps = Vec::new();
        if supplied > 0 {
            steps.push(Step::Give(seed[..supplied].to_vec()));
        }
        if fails {
            steps.push(Step::Fail);
        }

        let out = parse(&init_profile_with_entropy(
            &home,
            "tester",
            "",
            opening(Scripted::new(steps)),
        ));

        assert!(
            out["error"]
                .as_str()
                .unwrap_or_default()
                .contains(ENTROPY_UNREADABLE),
            "{case}: must fail with the entropy message, got: {out}"
        );
        assert!(
            out.get("created").is_none() && out.get("nodeId").is_none(),
            "{case}: a failed draw must not report an identity: {out}"
        );
        assert_eq!(
            files_under_keys(&home),
            Vec::<String>::new(),
            "{case}: a failed draw must write no key material"
        );

        cleanup(&dir);
    }
}

/// **An interrupted read is retried wherever it falls**: before the first
/// byte, between reads, twice running, and one byte from the end.
///
/// Each yields the identity the crate itself derives from the same 32 bytes, so
/// a retry that dropped or repeated a chunk is a different key and not merely a
/// created one.
#[test]
fn an_interruption_anywhere_in_the_draw_is_retried() {
    let seed = known_seed();
    let expected = node_id_the_crate_derives_from(&seed, "entropy-interrupt-table-oracle");

    // (case, the script)
    let cases: Vec<(&str, Vec<Step>)> = vec![
        (
            "before-any-byte",
            vec![Step::Interrupt, Step::Give(seed.clone())],
        ),
        (
            "twice-in-a-row",
            vec![
                Step::Give(seed[..16].to_vec()),
                Step::Interrupt,
                Step::Interrupt,
                Step::Give(seed[16..].to_vec()),
            ],
        ),
        (
            "one-byte-from-the-end",
            vec![
                Step::Give(seed[..31].to_vec()),
                Step::Interrupt,
                Step::Give(seed[31..].to_vec()),
            ],
        ),
    ];

    for (case, steps) in cases {
        let (dir, home) = fresh_home(&format!("entropy-interrupt-{case}"));

        let out = parse(&init_profile_with_entropy(
            &home,
            "tester",
            "",
            opening(Scripted::new(steps)),
        ));

        assert_eq!(
            out["created"],
            serde_json::json!(true),
            "{case}: an interruption is not a failure: {out}"
        );
        assert_eq!(
            out["nodeId"].as_str().unwrap_or_default(),
            expected,
            "{case}: every byte on both sides of the interruption must reach the key"
        );

        cleanup(&dir);
    }
}

// ---------------------------------------------------------------------------
// "No caller can choose the entropy source."
//
// The requirement is about two signatures, and a signature is something a test
// can pin: not by calling the function, but by what the compiler and the C
// header say its parameters are. The C++ half, `createEmbeddedIdentity(alias,
// passphrase)`, is out of this layer's sight; it is held by every C++ caller
// passing two arguments, which a third would break at compile time.
// ---------------------------------------------------------------------------

/// The FFI entry point, as compiled, takes a home, an alias and a passphrase
/// and nothing else.
///
/// Assigning the function to a pointer of that exact type is the assertion: a
/// fourth parameter (a path, a file descriptor, a callback supplying bytes)
/// changes the type and this stops compiling.
#[test]
fn the_init_entry_point_takes_home_alias_and_passphrase_and_nothing_else() {
    use std::os::raw::c_char;

    let entry: unsafe extern "C" fn(*const c_char, *const c_char, *const c_char) -> *mut c_char =
        radicle_local_ffi::radicle_local_init_profile;

    // Used, so the binding is not optimised into a vacuous one. A NULL home is
    // refused as an error object, never read as a source of anything.
    let raw = unsafe { entry(std::ptr::null(), std::ptr::null(), std::ptr::null()) };
    assert!(!raw.is_null());
    unsafe { radicle_local_ffi::radicle_free_string(raw) };
}

/// The declaration C++ compiles against names the same three parameters.
///
/// `radicle_ffi.h` is checked in by hand, so it can drift from the Rust
/// signature in either direction, and the C++ side would then pass an argument
/// Rust never reads, or none Rust expects. Counted from the declaration's own
/// text, up to its `;`.
#[test]
fn the_declared_init_prototype_has_three_const_char_pointer_parameters() {
    const HEADER: &str = include_str!("../../src/radicle_ffi.h");

    let start = HEADER
        .find("char* radicle_local_init_profile(")
        .expect("the header must declare radicle_local_init_profile");
    let declaration = &HEADER[start..];
    let end = declaration
        .find(';')
        .expect("the declaration must end in a semicolon");
    let open = declaration.find('(').unwrap();
    let params: Vec<&str> = declaration[open + 1..end]
        .trim_end_matches(')')
        .split(',')
        .map(str::trim)
        .collect();

    assert_eq!(
        params.len(),
        3,
        "the declared prototype must take exactly home, alias and passphrase, \
         got {params:?}"
    );
    for param in &params {
        assert!(
            param.starts_with("const char*"),
            "every declared parameter must be a `const char*`, got {param:?} in \
             {params:?}"
        );
    }
}
