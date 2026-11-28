#include <logos_test.h>

#include "settings_store.h"
#include "local_store.h"
// For isError() — the module's one failure-shape predicate, which every
// validation case below asserts against.
#include "seed_client.h"

#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <sstream>
#include <string>
#include <sys/stat.h>
#include <unistd.h>

using namespace radicle;

namespace {

/// A scratch settings file, removed on destruction.
///
/// Deliberately NOT pointed at the developer's real XDG data dir. The
/// `probe_*` examples read a real machine on purpose and are correctly not
/// tests; a test that wrote into a real profile would be a different thing
/// entirely.
struct ScratchSettings {
    std::string dir;

    explicit ScratchSettings(const std::string& name)
    {
        const char* base = std::getenv("TMPDIR");
        dir = std::string(base ? base : "/tmp") + "/radicle-settings-" + name;
        ::mkdir(dir.c_str(), 0755);
    }

    std::string path() const { return dir + "/settings.json"; }

    void write(const std::string& contents) const
    {
        std::ofstream out(path(), std::ios::trunc);
        out << contents;
    }

    /// No file at `path()`. The directory is named per test and nothing cleans
    /// it, so a test that asserts what a first write creates must not inherit
    /// the file an earlier run left.
    void remove() const { std::remove(path().c_str()); }

    /// The file's raw text, so a test can compare it byte for byte.
    std::string contents() const
    {
        std::ifstream in(path());
        std::stringstream buffer;
        buffer << in.rdbuf();
        return buffer.str();
    }

    /// The file as JSON, parsed here rather than through the store: an
    /// assertion about what is on disk must not ask the code under test what
    /// it wrote. A file that does not parse comes back as a discarded value,
    /// which equals no object an assertion names.
    nlohmann::json onDisk() const
    {
        return nlohmann::json::parse(contents(), nullptr, false);
    }
};

/// An array nested `depth` deep, written as text: `[` that many times, then
/// `]` that many times. Text rather than a built `nlohmann::json`, so a test
/// states the file's depth without asking the library to construct it.
std::string nestedArray(size_t depth)
{
    return std::string(depth, '[') + std::string(depth, ']');
}

/// An object nested `depth` deep (`depth` >= 1), as text: `{"a":{"a":{}}}` is
/// depth 3. Its innermost level is `{}`.
std::string nestedObject(size_t depth)
{
    std::string text;
    for (size_t level = 1; level < depth; ++level) text += R"({"a":)";
    return text + "{}" + std::string(depth - 1, '}');
}

/// Arrays and objects alternating, `depth` levels (`depth` >= 1), outermost an
/// array and innermost an array: `[{"a":[]}]` is depth 3.
std::string nestedMixed(size_t depth)
{
    std::string open;
    std::string close;
    for (size_t level = 1; level < depth; ++level) {
        const bool array = level % 2 == 1;
        open += array ? "[" : R"({"a":)";
        close = (array ? "]" : "}") + close;
    }
    return open + "[]" + close;
}

/// The file after one `radHome` write over `{"mode":"turbo","deep":<member>}`,
/// parsed here rather than through the store. A write that is refused comes
/// back as a discarded value, which equals no object a test names.
nlohmann::json fileAfterWriteOver(const std::string& name, const std::string& member)
{
    ScratchSettings s(name);
    s.write(R"({"mode":"turbo","deep":)" + member + "}");
    SettingsStore store{s.path()};
    if (isError(store.set(SettingsStore::kKeyRadHome, "/home/u/.radicle")))
        return nlohmann::json(nlohmann::json::value_t::discarded);
    return s.onDisk();
}

/// What a write over a file holding `member` as `deep` must leave: `deep`
/// still there, or dropped, beside the mode it kept and the key it wrote.
nlohmann::json keptWith(const std::string& member)
{
    return nlohmann::json::parse(
        R"({"mode":"turbo","deep":)" + member + R"(,"radHome":"/home/u/.radicle"})");
}

nlohmann::json droppedMember()
{
    return nlohmann::json::parse(R"({"mode":"turbo","radHome":"/home/u/.radicle"})");
}

} // namespace

// ---------------------------------------------------------------------------
// Isolation. The test that matters most, and the one that has to be
// input-dependent: two stores in two locations must not see each other.
// ---------------------------------------------------------------------------

LOGOS_TEST(two_settings_stores_in_two_places_do_not_see_each_others_values)
{
    ScratchSettings a("isolation-a");
    ScratchSettings b("isolation-b");

    SettingsStore alice{a.path()};
    SettingsStore bob{b.path()};

    // DIFFERENT values on purpose. Identical ones would pass against a store
    // that ignored its path entirely and shared one global file — the
    // same-answer-for-every-input trap. The assertion below can only hold if
    // each store actually read its own file.
    alice.set(SettingsStore::kKeyRemoteSeed, "https://alice.example");
    bob.set(SettingsStore::kKeyRemoteSeed, "https://bob.example");

    LOGOS_ASSERT_EQ(alice.get(SettingsStore::kKeyRemoteSeed),
                    std::string("https://alice.example"));
    LOGOS_ASSERT_EQ(bob.get(SettingsStore::kKeyRemoteSeed),
                    std::string("https://bob.example"));
}

LOGOS_TEST(a_write_through_one_store_is_invisible_to_a_store_elsewhere)
{
    ScratchSettings a("isolation-write-a");
    ScratchSettings b("isolation-write-b");

    SettingsStore alice{a.path()};
    SettingsStore bob{b.path()};

    // A NON-default mode, so the assertion below can tell "bob read his own
    // file" from "bob read alice's". Writing the default would pass against a
    // shared file, which is the same-answer-for-every-input trap.
    alice.set(SettingsStore::kKeyMode, SettingsStore::kModeLocal);

    // bob never wrote a mode, so he must still report the default — not
    // alice's. A shared file would hand back `local` here.
    LOGOS_ASSERT_EQ(bob.get(SettingsStore::kKeyMode),
                    std::string(SettingsStore::kModeExplore));
}

// ---------------------------------------------------------------------------
// Persistence — the point of the whole store.
// ---------------------------------------------------------------------------

LOGOS_TEST(a_setting_survives_being_read_back_by_a_fresh_store)
{
    // Two separate SettingsStore objects over one path: this is what "survives
    // a restart" means at this layer. The seed chosen via setRemoteSeed used
    // to be lost on restart entirely; this is the cheapest proof it is not.
    ScratchSettings s("persist");
    {
        SettingsStore first{s.path()};
        first.set(SettingsStore::kKeyRemoteSeed, "https://persisted.example");
    }

    SettingsStore second{s.path()};
    LOGOS_ASSERT_EQ(second.get(SettingsStore::kKeyRemoteSeed),
                    std::string("https://persisted.example"));
}

LOGOS_TEST(setting_one_key_leaves_the_others_untouched)
{
    ScratchSettings s("independent");
    SettingsStore store{s.path()};

    // A NON-default mode, so the assertion below distinguishes "the first write
    // survived the second" from "the mode fell back to its default". Writing
    // `explore` here would satisfy the assertion either way.
    store.set(SettingsStore::kKeyMode, SettingsStore::kModeLocal);
    store.set(SettingsStore::kKeyRemoteSeed, "https://kept.example");

    // Distinct values again, so a store that overwrote the whole file on every
    // write would lose the first one and fail here.
    LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyMode),
                    std::string(SettingsStore::kModeLocal));
    LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyRemoteSeed),
                    std::string("https://kept.example"));
}

LOGOS_TEST(set_returns_the_whole_settings_object_not_just_the_changed_key)
{
    ScratchSettings s("returns-all");
    SettingsStore store{s.path()};

    const auto result = store.set(SettingsStore::kKeyMode, SettingsStore::kModeExplore);

    LOGOS_ASSERT_FALSE(isError(result));
    // A caller re-renders from this one reply, so every key has to be present
    // or the view shows a stale value for whatever is missing.
    LOGOS_ASSERT_TRUE(result.contains(SettingsStore::kKeyMode));
    LOGOS_ASSERT_TRUE(result.contains(SettingsStore::kKeyRadHome));
    LOGOS_ASSERT_TRUE(result.contains(SettingsStore::kKeyRadSocket));
    LOGOS_ASSERT_TRUE(result.contains(SettingsStore::kKeyGitPath));
    LOGOS_ASSERT_TRUE(result.contains(SettingsStore::kKeyRemoteSeed));
}

// ---------------------------------------------------------------------------
// Defaults and damaged files.
// ---------------------------------------------------------------------------

LOGOS_TEST(a_missing_settings_file_yields_defaults_rather_than_an_error)
{
    ScratchSettings s("missing");
    SettingsStore store{s.path()};

    // `explore`. See the default-mode test below for why this is the specific
    // value rather than merely "some known mode".
    LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyMode),
                    std::string(SettingsStore::kModeExplore));
}

LOGOS_TEST(a_corrupted_settings_file_yields_defaults_rather_than_a_dead_module)
{
    ScratchSettings s("corrupt");
    s.write("{ this is not json at all");

    SettingsStore store{s.path()};
    LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyMode),
                    std::string(SettingsStore::kModeExplore));
}

LOGOS_TEST(the_default_mode_needs_no_local_profile_to_show_anything)
{
    // The first-run guarantee, and the reason the default is Explore.
    //
    // The default used to be `local`, on the argument that it preserved what
    // the module did before modes existed. That argument is about a user who
    // ALREADY has a working profile — but the default is what a user with NO
    // settings file gets, and the overwhelmingly common case for that is a
    // first run, where there may be no Radicle home at all. In `local` with no
    // profile the module can show nothing: `localListRepos` is the only list
    // call the UI will issue, it returns the "no local profile" error, and the
    // repository list stays empty with the seed never asked.
    //
    // That is not hypothetical. Every seed-browsing end-to-end spec runs under
    // a throwaway `$HOME` with no profile, and all of them broke at once the
    // moment the `local` default landed — the app started in a mode that could
    // not answer.
    //
    // Asserted as a PROPERTY of the default rather than as the literal string
    // `explore`, because the property is what actually matters and it survives
    // a future fourth mode: whatever the default is, it must be a mode that
    // reaches a seed over HTTP rather than one that needs a home on this
    // machine. `local` and `embedded` both fail that; only `explore` passes.
    ScratchSettings s("first-run-default");
    SettingsStore store{s.path()};

    const auto mode = store.get(SettingsStore::kKeyMode);

    LOGOS_ASSERT_TRUE(SettingsStore::isKnownMode(mode));
    LOGOS_ASSERT_TRUE(SettingsStore::modeIsStartable(mode));
    LOGOS_ASSERT_TRUE(mode != std::string(SettingsStore::kModeLocal));
    LOGOS_ASSERT_TRUE(mode != std::string(SettingsStore::kModeEmbedded));
}

LOGOS_TEST(the_default_and_the_unknown_mode_fallback_agree)
{
    // Two answers to two different questions that happen to coincide, and this
    // pins the coincidence so a future change to either one is a deliberate
    // divergence rather than an accident.
    //
    // `load()` gives an uninterpretable stored mode Explore, on the grounds
    // that a file this build cannot read is no basis for claiming a node
    // identity. The default now agrees, on the different grounds that a first
    // run has no profile to read. The file was inconsistent about this for one
    // milestone, and the inconsistency was the bug: the fallback path was
    // right and the default path was not.
    ScratchSettings fresh("agreement-fresh");
    ScratchSettings junk("agreement-junk");
    junk.write(R"({"mode":"turbo"})");

    SettingsStore freshStore{fresh.path()};
    SettingsStore junkStore{junk.path()};

    LOGOS_ASSERT_EQ(freshStore.get(SettingsStore::kKeyMode),
                    junkStore.get(SettingsStore::kKeyMode));
}

LOGOS_TEST(unknown_keys_in_the_file_are_ignored_rather_than_surfaced)
{
    ScratchSettings s("unknown-in-file");
    s.write(R"({"mode":"explore","somethingElse":"x"})");

    SettingsStore store{s.path()};
    // The known key is honoured...
    LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyMode),
                    std::string(SettingsStore::kModeExplore));
    // ...and the unknown one does not appear in the reported settings.
    LOGOS_ASSERT_FALSE(store.all().contains("somethingElse"));
}

LOGOS_TEST(a_mode_in_the_file_that_this_build_does_not_know_is_not_handed_back)
{
    // `set()` validates, so this cannot come from the module — but the file is
    // on disk, and a hand edit, a torn write, or a NEWER build all produce one.
    // It used to be read back verbatim, and both consumers then treated it as
    // Local by falling through an else: the module reported the attached
    // profile's home and full read access under a mode name the UI cannot draw.
    ScratchSettings s("unknown-mode-in-file");
    s.write(R"({"mode":"turbo","remoteSeed":"https://kept.example.test"})");

    SettingsStore store{s.path()};

    // Explore, NOT the `local` default: a file this build cannot interpret is
    // no basis for claiming a node identity, and Explore is the one mode that
    // touches no local profile. Asserting the specific value rather than merely
    // "known" is what pins that down — a fallback to `local` is a known mode
    // too, and is exactly the leak this guards against.
    LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyMode),
                    std::string(SettingsStore::kModeExplore));

    // The rest of the file survives. A corrupt mode is not a reason to discard
    // settings that parsed perfectly well — and asserting it here is what stops
    // a future "just return defaults" simplification from silently dropping the
    // user's seed along with the bad mode.
    LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyRemoteSeed),
                    std::string("https://kept.example.test"));
}

LOGOS_TEST(a_known_mode_in_the_file_is_read_back_unchanged)
{
    // The other half, and the reason the test above means anything: the
    // sanitising must be input-dependent. A load() that returned Explore for
    // every stored mode would pass the assertion above and fail here.
    ScratchSettings s("known-mode-in-file");

    for (const char* mode : {SettingsStore::kModeLocal,
                             SettingsStore::kModeEmbedded,
                             SettingsStore::kModeExplore}) {
        s.write(std::string(R"({"mode":")") + mode + R"("})");
        SettingsStore store{s.path()};
        LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyMode), std::string(mode));
    }
}

// ---------------------------------------------------------------------------
// Validation on write. Each case asserts the REFUSAL and that nothing changed
// — a validator that reported an error and stored the value anyway would pass
// a test that only checked the return.
// ---------------------------------------------------------------------------

LOGOS_TEST(an_unknown_setting_key_is_refused_by_name)
{
    ScratchSettings s("unknown-key");
    SettingsStore store{s.path()};

    const auto result = store.set("noSuchSetting", "x");
    LOGOS_ASSERT_TRUE(isError(result));
    LOGOS_ASSERT_CONTAINS(result["error"].get<std::string>(), std::string("noSuchSetting"));
}

LOGOS_TEST(an_unknown_mode_is_refused_and_the_message_lists_the_valid_ones)
{
    ScratchSettings s("bad-mode");
    SettingsStore store{s.path()};

    const auto result = store.set(SettingsStore::kKeyMode, "turbo");
    LOGOS_ASSERT_TRUE(isError(result));
    // Every valid mode by NAME, from the constants. The message used to be
    // spelled out in the validator and this assertion checked one hardcoded
    // word, so a rename could leave the sentence naming modes that no longer
    // exist with nothing going red. Asserting all three ties the message to the
    // set it claims to describe.
    LOGOS_ASSERT_CONTAINS(result["error"].get<std::string>(),
                          std::string(SettingsStore::kModeExplore));
    LOGOS_ASSERT_CONTAINS(result["error"].get<std::string>(),
                          std::string(SettingsStore::kModeLocal));
    LOGOS_ASSERT_CONTAINS(result["error"].get<std::string>(),
                          std::string(SettingsStore::kModeEmbedded));

    // And the stored value is untouched — still the default, since nothing was
    // ever written. This is the half a return-only assertion would miss.
    LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyMode),
                    std::string(SettingsStore::kModeExplore));
}

LOGOS_TEST(a_refused_mode_names_all_three_whatever_was_tried)
{
    // The sentence has to describe the SET, not echo the attempt. `remote` is
    // the case worth a row of its own: it is a real word in this module's
    // vocabulary (the method prefix Explore derives), so it is the value a
    // caller is likeliest to try, and a message built around the attempted
    // value would name that and nothing else.
    //
    // A table, because one value cannot tell a message built from the valid
    // set from one that happens to contain the three words for that input.
    ScratchSettings s("bad-mode-table");
    SettingsStore store{s.path()};

    for (const char* tried : {"remote", "turbo"}) {
        const auto result = store.set(SettingsStore::kKeyMode, tried);
        LOGOS_ASSERT_TRUE(isError(result));
        const std::string message = result["error"].get<std::string>();
        LOGOS_ASSERT_CONTAINS(message, std::string(SettingsStore::kModeExplore));
        LOGOS_ASSERT_CONTAINS(message, std::string(SettingsStore::kModeLocal));
        LOGOS_ASSERT_CONTAINS(message, std::string(SettingsStore::kModeEmbedded));
    }
}

LOGOS_TEST(the_store_reports_exactly_five_keys_before_and_after_each_write)
{
    // The store's own reply, counted. The test that checks the five names with
    // `contains()` cannot see a sixth key, and a reply grown by one — a node
    // key, say, which this surface must never carry — left the whole suite
    // green. The count is what a stray key cannot pass.
    //
    // The file is REMOVED first. `ScratchSettings` names a fixed directory under
    // TMPDIR that nothing cleans (whatever its comment says of destruction), and
    // this test writes all five keys into it, so on a second local run the
    // opening read would be of a file already holding values — not of one that
    // has never been written, which is the half it claims. The check after the
    // removal makes a removal that did not take fail here.
    ScratchSettings s("five-keys");
    std::remove(s.path().c_str());
    LOGOS_ASSERT_FALSE(std::ifstream(s.path()).good());
    SettingsStore store{s.path()};

    LOGOS_ASSERT_EQ(store.all().size(), size_t(5));

    // One accepted value per key, so "after a write to each" is the whole set.
    const auto afterMode = store.set(SettingsStore::kKeyMode, SettingsStore::kModeLocal);
    LOGOS_ASSERT_EQ(afterMode.size(), size_t(5));
    const auto afterHome = store.set(SettingsStore::kKeyRadHome, "/some/home");
    LOGOS_ASSERT_EQ(afterHome.size(), size_t(5));
    const auto afterSocket = store.set(SettingsStore::kKeyRadSocket, "/run/user/1000/rad.sock");
    LOGOS_ASSERT_EQ(afterSocket.size(), size_t(5));
    const auto afterGit = store.set(SettingsStore::kKeyGitPath, "");
    LOGOS_ASSERT_EQ(afterGit.size(), size_t(5));
    const auto afterSeed = store.set(SettingsStore::kKeyRemoteSeed, "https://seed.example");
    LOGOS_ASSERT_EQ(afterSeed.size(), size_t(5));

    LOGOS_ASSERT_EQ(store.all().size(), size_t(5));
}

// ---------------------------------------------------------------------------
// What a write does to the file. "A settings write changes only the key it
// names": every assertion below is about the file ON DISK, parsed by the test
// itself, because the store's reply is the normalised view and would hide
// exactly what these tests are about. Each test writes its starting file by
// hand, so nothing an earlier run left in the directory can stand in for it.
//
// The mutation these exist for is the shape `set()` had before this change:
// `auto settings = load(); settings[key] = value; save(settings);`, which saves
// the normalised view: the unknown mode is saved as `explore`, the unknown and
// non-string members are dropped or defaulted, and the defaults the file never
// held are added. The refused-write test is the exception, and guards a
// different mutation: a save that runs before a refusal returns, which the old
// shape did not do either. Which tests went red under the old shape is in
// `design.md`, Decision 7, measured there; it is not repeated here, where it
// would go stale the next time a test is added.
// ---------------------------------------------------------------------------

LOGOS_TEST(a_write_of_one_key_keeps_an_unknown_mode_and_an_unknown_key)
{
    // The file a newer build, or a hand edit, leaves: a mode this build cannot
    // interpret and a key it does not know, beside a seed it does.
    ScratchSettings s("write-keeps-unknown");
    s.write(R"({"mode":"turbo","remoteSeed":"https://a.example","futureKey":"x"})");
    SettingsStore store{s.path()};

    // A key the file does not hold, so the write is "unrelated" to every member
    // it must keep.
    LOGOS_ASSERT_FALSE(isError(store.set(SettingsStore::kKeyRadHome, "/home/u/.radicle")));

    LOGOS_ASSERT_EQ(s.onDisk(), nlohmann::json::parse(
        R"({"mode":"turbo","remoteSeed":"https://a.example","futureKey":"x",)"
        R"("radHome":"/home/u/.radicle"})"));
}

LOGOS_TEST(a_write_keeps_a_known_key_whose_value_is_not_a_string)
{
    // Two non-string values of two different JSON types. `load()` defaults
    // both to "", so a write that saved the normalised view would write
    // `"radSocket":""` and `"gitPath":""` here.
    ScratchSettings s("write-keeps-non-string");
    s.write(R"({"mode":"local","radSocket":5,"gitPath":null})");
    SettingsStore store{s.path()};

    LOGOS_ASSERT_FALSE(isError(store.set(SettingsStore::kKeyRemoteSeed, "https://b.example")));

    LOGOS_ASSERT_EQ(s.onDisk(), nlohmann::json::parse(
        R"({"mode":"local","radSocket":5,"gitPath":null,"remoteSeed":"https://b.example"})"));
}

LOGOS_TEST(writing_the_key_that_held_an_uninterpretable_value_replaces_only_that_value)
{
    // The other half of "keeps what it did not name": the member it DOES name
    // is replaced, even when it held a value this build cannot read. A write
    // that kept every existing member verbatim, its own included, would leave
    // `turbo` here.
    ScratchSettings s("write-replaces-named");
    s.write(R"({"mode":"turbo","futureKey":"x"})");
    SettingsStore store{s.path()};

    LOGOS_ASSERT_FALSE(isError(store.set(SettingsStore::kKeyMode, SettingsStore::kModeLocal)));

    LOGOS_ASSERT_EQ(s.onDisk(), nlohmann::json::parse(R"({"mode":"local","futureKey":"x"})"));
}

LOGOS_TEST(a_refused_write_leaves_the_file_as_it_was)
{
    // Compared byte for byte, not only as JSON: the file is written with
    // spacing this module would never produce, so a refused write that
    // rewrote it with the same members would still show here.
    const std::string original = "{ \"mode\" : \"turbo\",  \"futureKey\" : \"x\" }";
    ScratchSettings s("write-refused");
    s.write(original);
    SettingsStore store{s.path()};

    // Refused by validation: `remote` is a word of this module's vocabulary but
    // not a mode.
    LOGOS_ASSERT_TRUE(isError(store.set(SettingsStore::kKeyMode, "remote")));
    LOGOS_ASSERT_EQ(s.contents(), original);

    // Refused as a key the store does not know.
    LOGOS_ASSERT_TRUE(isError(store.set("noSuchSetting", "y")));
    LOGOS_ASSERT_EQ(s.contents(), original);
    LOGOS_ASSERT_EQ(s.onDisk(), nlohmann::json::parse(R"({"mode":"turbo","futureKey":"x"})"));
}

LOGOS_TEST(the_reply_is_the_normalised_view_while_the_file_keeps_its_members)
{
    ScratchSettings s("write-reply-normalised");
    s.write(R"({"mode":"turbo","remoteSeed":"https://a.example","futureKey":"x"})");
    SettingsStore store{s.path()};

    const auto reply = store.set(SettingsStore::kKeyRadHome, "/home/u/.radicle");

    LOGOS_ASSERT_FALSE(isError(reply));
    // Exactly the five keys: `futureKey` is the file's, not the reply's.
    LOGOS_ASSERT_EQ(reply.size(), size_t(5));
    LOGOS_ASSERT_FALSE(reply.contains("futureKey"));
    // `turbo` is reported as explore, as every read reports it.
    LOGOS_ASSERT_EQ(reply[SettingsStore::kKeyMode].get<std::string>(),
                    std::string(SettingsStore::kModeExplore));
    // Values that came from the file and from this write, so a reply built
    // from defaults alone, or from the file before the write, fails here.
    LOGOS_ASSERT_EQ(reply[SettingsStore::kKeyRemoteSeed].get<std::string>(),
                    std::string("https://a.example"));
    LOGOS_ASSERT_EQ(reply[SettingsStore::kKeyRadHome].get<std::string>(),
                    std::string("/home/u/.radicle"));
    // What the next read reports, which is what getSettings dumps.
    LOGOS_ASSERT_EQ(reply, store.all());

    // And the file differs from the reply in exactly those members.
    const auto disk = s.onDisk();
    LOGOS_ASSERT_EQ(disk[SettingsStore::kKeyMode].get<std::string>(), std::string("turbo"));
    LOGOS_ASSERT_EQ(disk["futureKey"].get<std::string>(), std::string("x"));
}

LOGOS_TEST(a_write_over_a_file_that_is_not_json_replaces_it_with_only_the_key_written)
{
    ScratchSettings s("write-over-not-json");
    s.write("{ this is not json at all");
    SettingsStore store{s.path()};

    const auto reply = store.set(SettingsStore::kKeyMode, SettingsStore::kModeLocal);

    // The file's state is no reason to refuse: the write is the user's way out
    // of a file this build cannot read.
    LOGOS_ASSERT_FALSE(isError(reply));
    LOGOS_ASSERT_EQ(reply[SettingsStore::kKeyMode].get<std::string>(),
                    std::string(SettingsStore::kModeLocal));
    LOGOS_ASSERT_EQ(s.onDisk(), nlohmann::json::parse(R"({"mode":"local"})"));
}

LOGOS_TEST(a_write_over_json_that_is_not_an_object_replaces_it_with_only_the_key_written)
{
    // Valid JSON naming a mode, so a write that tried to carry an array's
    // contents forward, or refused a file that parsed, is told apart.
    ScratchSettings s("write-over-array");
    s.write(R"(["mode","turbo"])");
    SettingsStore store{s.path()};

    const auto reply = store.set(SettingsStore::kKeyMode, SettingsStore::kModeLocal);

    LOGOS_ASSERT_FALSE(isError(reply));
    LOGOS_ASSERT_EQ(s.onDisk(), nlohmann::json::parse(R"({"mode":"local"})"));
}

LOGOS_TEST(a_write_over_a_file_that_exists_but_cannot_be_opened_replaces_it)
{
    // "Cannot be read", as distinct from absent and from not-JSON. A symlink
    // that points at itself is the one such file whose state does not depend on
    // who runs the suite: a mode-000 file is still readable by root, so a test
    // built on it would pass or fail with the user. `open(2)` on it fails with
    // ELOOP, so the store reads nothing; `rename(2)` replaces the link itself
    // rather than following it, so the write has somewhere to land.
    ScratchSettings s("write-over-unopenable");
    s.remove();
    LOGOS_ASSERT_EQ(::symlink(s.path().c_str(), s.path().c_str()), 0);
    // The precondition, checked: were the file openable this test would be
    // about a different branch of the reader.
    LOGOS_ASSERT_FALSE(std::ifstream(s.path()).good());
    SettingsStore store{s.path()};

    // A non-default value, so the reply cannot be mistaken for the defaults.
    const auto reply = store.set(SettingsStore::kKeyMode, SettingsStore::kModeLocal);

    // The file's state is no reason to refuse.
    LOGOS_ASSERT_FALSE(isError(reply));
    LOGOS_ASSERT_EQ(reply[SettingsStore::kKeyMode].get<std::string>(),
                    std::string(SettingsStore::kModeLocal));
    // Parsed through the same path, which only succeeds once the link is gone.
    LOGOS_ASSERT_EQ(s.onDisk(), nlohmann::json::parse(R"({"mode":"local"})"));
}

LOGOS_TEST(the_file_layout_does_not_change_what_a_write_keeps)
{
    // "The file's layout does not change what a write keeps". The starting file
    // is laid out the way the module never writes it: a newline after `{` and
    // after each `,`, a tab before each member, a space either side of each `:`,
    // and a number spelled in exponent form. Several lines, so a reader that
    // looked at the first line only, or a write that spliced text into the
    // file, would lose or mangle a member here and not in the one-line files
    // above.
    ScratchSettings s("write-layout");
    s.write("{\n\t\"futureKey\" : 1.5e3,\n\t\"mode\" : \"turbo\"}");
    SettingsStore store{s.path()};

    LOGOS_ASSERT_FALSE(isError(store.set(SettingsStore::kKeyRadHome, "/home/u/.radicle")));

    // `1.5e3` is the file's spelling; the requirement is on the value, which
    // equals 1500 however the writer spells it.
    LOGOS_ASSERT_EQ(s.onDisk(), nlohmann::json::parse(
        R"({"futureKey":1.5e3,"mode":"turbo","radHome":"/home/u/.radicle"})"));
}

LOGOS_TEST(a_member_the_file_names_twice_is_kept_once_with_its_last_value)
{
    // `futureKey` is not the key written, and its two values differ, so a write
    // that kept the first, or both, is told apart from one that kept the last.
    ScratchSettings s("write-duplicate");
    s.write(R"({"futureKey":"a","mode":"turbo","futureKey":"b"})");
    SettingsStore store{s.path()};

    LOGOS_ASSERT_FALSE(isError(store.set(SettingsStore::kKeyRadHome, "/home/u/.radicle")));

    LOGOS_ASSERT_EQ(s.onDisk(), nlohmann::json::parse(
        R"({"mode":"turbo","futureKey":"b","radHome":"/home/u/.radicle"})"));

    // The text, not the parse: a parse that takes the last value cannot see a
    // file that still names the member twice, which is what a write that edits
    // the file's text in place would leave.
    const std::string text = s.contents();
    size_t count = 0;
    for (size_t at = text.find("\"futureKey\""); at != std::string::npos;
         at = text.find("\"futureKey\"", at + 1))
        ++count;
    LOGOS_ASSERT_EQ(count, size_t(1));
}

LOGOS_TEST(the_first_write_creates_a_file_holding_only_that_key)
{
    // No defaults written out for keys the file never held: an empty
    // `radHome` on disk is indistinguishable from one the user chose to
    // clear, and a later build's default would then never reach this user.
    ScratchSettings s("write-first");
    s.remove();
    LOGOS_ASSERT_FALSE(std::ifstream(s.path()).good());
    SettingsStore store{s.path()};

    LOGOS_ASSERT_FALSE(isError(store.set(SettingsStore::kKeyRemoteSeed, "https://a.example")));

    LOGOS_ASSERT_EQ(s.onDisk(), nlohmann::json::parse(R"({"remoteSeed":"https://a.example"})"));
    LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyMode), std::string(SettingsStore::kModeExplore));
}

// ---------------------------------------------------------------------------
// "A settings write drops a member nested too deep to keep". The cap is 128:
// the two tests either side of it are what pin the number, since a cap of 127
// fails the first and a cap of 129 the second. The 100000-deep one is the
// crash: a write that serialised such a member recursively overflowed the
// stack and took the host process with it.
// ---------------------------------------------------------------------------

LOGOS_TEST(a_member_exactly_128_deep_is_kept)
{
    const std::string deep = nestedArray(128);
    ScratchSettings s("write-deep-128");
    s.write(R"({"mode":"turbo","deep":)" + deep + "}");
    SettingsStore store{s.path()};

    LOGOS_ASSERT_FALSE(isError(store.set(SettingsStore::kKeyRadHome, "/home/u/.radicle")));

    LOGOS_ASSERT_EQ(s.onDisk(), nlohmann::json::parse(
        R"({"mode":"turbo","deep":)" + deep + R"(,"radHome":"/home/u/.radicle"})"));
}

LOGOS_TEST(a_member_129_deep_is_dropped_and_the_rest_kept)
{
    // `turbo` beside it, so a write that answered a too-deep member by
    // treating the whole file as unreadable, and kept only the key written,
    // is told apart from one that drops that member alone.
    ScratchSettings s("write-deep-129");
    s.write(R"({"mode":"turbo","deep":)" + nestedArray(129) + "}");
    SettingsStore store{s.path()};

    LOGOS_ASSERT_FALSE(isError(store.set(SettingsStore::kKeyRadHome, "/home/u/.radicle")));

    LOGOS_ASSERT_EQ(s.onDisk(), nlohmann::json::parse(
        R"({"mode":"turbo","radHome":"/home/u/.radicle"})"));
}

// NO SPEC: the spec leaves the file's whitespace open. This pins that a kept
// nested member is written compactly, so the file a write leaves grows with
// what it holds and not with how deep that nests. Indented, every level is a
// line of its own indented by its depth, so the text grows with the square of
// the depth.
LOGOS_TEST(a_kept_nested_member_does_not_grow_the_file_with_its_depth)
{
    const std::string original = R"({"mode":"turbo","deep":)" + nestedArray(128) + "}";
    ScratchSettings s("write-deep-size");
    s.write(original);
    SettingsStore store{s.path()};

    LOGOS_ASSERT_FALSE(isError(store.set(SettingsStore::kKeyRadHome, "/home/u/.radicle")));

    // The member is still there, so the size below is not small for want of it.
    LOGOS_ASSERT_TRUE(s.onDisk().contains("deep"));

    // The file gains one member, `"radHome": "/home/u/.radicle"`, and a line
    // break and two spaces of indent for each member it now lays out on its
    // own line: well under 100 bytes. A bound of twice the original, which is
    // what this was, let a write that adds a line break per level through.
    // 100 is hardcoded from the one member added, not taken from what the
    // writer produced.
    LOGOS_ASSERT_TRUE(s.contents().size() <= original.size() + 100);
}

// "Of the members other than the one written, ... every one of depth 128 or
// less" and "the number of arrays and objects on its deepest path, counting the
// value itself when it is one". The 128 and 129 tests above use arrays only, so
// a depth that counted arrays and not objects, or followed the first child and
// not the deepest, would pass them; each shape below is one such mutation.

LOGOS_TEST(a_member_of_nested_objects_counts_its_objects_toward_its_depth)
{
    LOGOS_ASSERT_EQ(fileAfterWriteOver("write-deep-objects-128", nestedObject(128)),
                    keptWith(nestedObject(128)));
    LOGOS_ASSERT_EQ(fileAfterWriteOver("write-deep-objects-129", nestedObject(129)),
                    droppedMember());
}

LOGOS_TEST(a_member_alternating_arrays_and_objects_counts_both_toward_its_depth)
{
    LOGOS_ASSERT_EQ(fileAfterWriteOver("write-deep-mixed-128", nestedMixed(128)),
                    keptWith(nestedMixed(128)));
    LOGOS_ASSERT_EQ(fileAfterWriteOver("write-deep-mixed-129", nestedMixed(129)),
                    droppedMember());
}

LOGOS_TEST(a_member_is_as_deep_as_its_deepest_path_wherever_that_path_sits)
{
    // Depth 129 by the second child, then by the first: a walk that followed
    // only one end of an array would keep one of them.
    const std::string deepSecond = "[[]," + nestedArray(128) + "]";
    const std::string deepFirst = "[" + nestedArray(128) + ",[]]";
    LOGOS_ASSERT_EQ(fileAfterWriteOver("write-deep-second", deepSecond), droppedMember());
    LOGOS_ASSERT_EQ(fileAfterWriteOver("write-deep-first", deepFirst), droppedMember());
}

LOGOS_TEST(a_member_with_two_branches_is_as_deep_as_one_of_them_not_both)
{
    // Two branches of 127 under one array: depth 128, kept. A depth that added
    // the branches, or counted every level it visited, would reach 255.
    const std::string branches = "[" + nestedArray(127) + "," + nestedArray(127) + "]";
    LOGOS_ASSERT_EQ(fileAfterWriteOver("write-deep-branches", branches), keptWith(branches));
}

LOGOS_TEST(a_member_nested_far_deeper_still_lets_the_write_complete)
{
    // Last of the depth tests on purpose: before the cap, this one ended the
    // test binary with a stack overflow, and the ones above still reported.
    ScratchSettings s("write-deep-100000");
    s.write(R"({"mode":"turbo","deep":)" + nestedArray(100000) + "}");
    SettingsStore store{s.path()};

    const auto reply = store.set(SettingsStore::kKeyRadHome, "/home/u/.radicle");

    LOGOS_ASSERT_FALSE(isError(reply));
    LOGOS_ASSERT_EQ(reply[SettingsStore::kKeyRadHome].get<std::string>(),
                    std::string("/home/u/.radicle"));
    LOGOS_ASSERT_EQ(s.onDisk(), nlohmann::json::parse(
        R"({"mode":"turbo","radHome":"/home/u/.radicle"})"));
}

LOGOS_TEST(all_three_modes_are_accepted)
{
    ScratchSettings s("modes");
    SettingsStore store{s.path()};

    for (const char* mode : {SettingsStore::kModeLocal,
                             SettingsStore::kModeEmbedded,
                             SettingsStore::kModeExplore}) {
        const auto result = store.set(SettingsStore::kKeyMode, mode);
        LOGOS_ASSERT_FALSE(isError(result));
        LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyMode), std::string(mode));
    }
}

LOGOS_TEST(every_known_mode_is_startable_and_the_two_views_of_that_agree)
{
    // Embedded joined the startable set when it gained a home to point at.
    //
    // Asserted through BOTH accessors on purpose. They are two views of one
    // fact and the boolean is derived from the set — so a `modeIsStartable`
    // that stopped consulting `startableModes` and hardcoded its own answer
    // would still satisfy either assertion alone. Checking that the two agree
    // for every known mode is what pins the derivation rather than the values.
    for (const char* mode : {SettingsStore::kModeExplore,
                             SettingsStore::kModeLocal,
                             SettingsStore::kModeEmbedded}) {
        LOGOS_ASSERT_TRUE(SettingsStore::isKnownMode(mode));
        LOGOS_ASSERT_TRUE(SettingsStore::modeIsStartable(mode));

        bool inSet = false;
        for (const auto& m : SettingsStore::startableModes())
            if (m == mode) inSet = true;
        LOGOS_ASSERT_TRUE(inSet);
    }

    // And the derivation is input-dependent, which is the half the loop above
    // cannot show: a `modeIsStartable` returning true unconditionally would
    // pass every assertion so far. An unknown mode must be unstartable.
    LOGOS_ASSERT_FALSE(SettingsStore::modeIsStartable("turbo"));
    LOGOS_ASSERT_EQ(SettingsStore::startableModes().size(), size_t(3));
}

// ---------------------------------------------------------------------------
// The embedded home.
//
// Same input-dependence discipline as the settings path above it: every case
// pins a DIFFERENT expected answer, so a resolver that ignored its arguments
// cannot pass.
// ---------------------------------------------------------------------------

LOGOS_TEST(the_embedded_home_follows_xdg_data_home_when_set)
{
    LOGOS_ASSERT_EQ(embeddedHomeFor("/xdg/data", "/home/u"),
                    std::string("/xdg/data/radicle-module/embedded-home"));
}

LOGOS_TEST(the_embedded_home_falls_back_to_the_xdg_default_under_the_user_home)
{
    LOGOS_ASSERT_EQ(embeddedHomeFor("", "/home/u"),
                    std::string("/home/u/.local/share/radicle-module/embedded-home"));
}

LOGOS_TEST(no_environment_at_all_yields_no_embedded_home)
{
    // Same reasoning as the settings path: an empty answer makes the caller
    // report "nowhere to put an identity" rather than creating a permanent
    // signing key at a path relative to nothing.
    LOGOS_ASSERT_TRUE(embeddedHomeFor("", "").empty());
}

LOGOS_TEST(two_basecamp_profiles_get_two_embedded_homes)
{
    // The isolation that makes the embedded mode safe for two Basecamp
    // profiles, and it is the SAME isolation the settings file already has,
    // because both are derived from XDG_DATA_HOME by one rule.
    const std::string alice = embeddedHomeFor("/profiles/alice/xdg-data", "/home/u");
    const std::string bob   = embeddedHomeFor("/profiles/bob/xdg-data", "/home/u");

    LOGOS_ASSERT_TRUE(alice != bob);
    LOGOS_ASSERT_CONTAINS(alice, std::string("alice"));
    LOGOS_ASSERT_CONTAINS(bob, std::string("bob"));
}

LOGOS_TEST(the_embedded_home_is_never_the_users_own_radicle_home)
{
    // **The assertion the whole mode rests on.** Embedded promises an identity
    // separate from any node the user already runs; a home that could resolve
    // to `~/.radicle` would hand it their real signing key and put two nodes on
    // one git storage.
    //
    // Pinned as a property rather than as a literal path, so it survives a
    // future change to where the home lives: whatever the answer is, it must
    // not be the conventional Radicle home, and it must not be reachable from a
    // user home when an XDG data dir was given.
    LOGOS_ASSERT_TRUE(embeddedHomeFor("/xdg/data", "/home/u")
                      != std::string("/home/u/.radicle"));
    LOGOS_ASSERT_TRUE(embeddedHomeFor("", "/home/u")
                      != std::string("/home/u/.radicle"));

    // And with an XDG data dir in force, the user home does not appear in the
    // answer at all — the only input it reads is the data dir. A resolver that
    // consulted `$HOME` as well would fail this while passing every assertion
    // above.
    LOGOS_ASSERT_TRUE(embeddedHomeFor("/xdg/data", "/home/u").find("/home/u")
                      == std::string::npos);
}

LOGOS_TEST(the_embedded_home_and_the_settings_file_are_siblings_not_the_same_path)
{
    // Two facts at once, and both matter.
    //
    // Siblings: they share a parent, which is what makes the per-profile
    // separation of one the per-profile separation of the other rather than
    // two rules that happen to agree today.
    //
    // Not the same path: an embedded home that landed ON the settings file
    // would have `Home::new` trying to make a directory where a file is — and
    // a `Profile::init` that ever succeeded there would put a signing key in
    // the settings directory. Asserting they differ costs one line and rules
    // out a whole class of copy-paste error in the shared derivation.
    const std::string settings = settingsPathFor("/xdg/data", "/home/u");
    const std::string home     = embeddedHomeFor("/xdg/data", "/home/u");

    LOGOS_ASSERT_TRUE(settings != home);
    LOGOS_ASSERT_CONTAINS(settings, std::string("/radicle-module/"));
    LOGOS_ASSERT_CONTAINS(home, std::string("/radicle-module/"));
}

LOGOS_TEST(the_embedded_home_is_a_directory_a_socket_would_not_fit_under)
{
    // Not a limitation being accepted — the reason the design works, pinned so
    // a future change cannot quietly reintroduce "the socket follows the home".
    //
    // Under Basecamp's real per-profile layout this directory is far past the
    // 108-byte sun_path cap (Phase 0 measured 166 bytes), which is fine for a
    // directory and fatal for a socket. `resolveSocket` therefore never derives
    // the socket from the home when a runtime dir exists, and this asserts the
    // two facts together: the home is long, and the socket resolved alongside
    // it is short and does not contain it.
    const std::string longXdg =
        "/home/user/.local/share/logos/basecamp/profiles/alice/xdg-data-dir-with-a-long-name";
    const std::string home = embeddedHomeFor(longXdg, "/home/user");

    const NodePaths paths =
        resolvePaths(home, "", "", "", "", "/run/user/1000", "alice");

    LOGOS_ASSERT_EQ(paths.home, home);
    LOGOS_ASSERT_TRUE(paths.socket.find(home) == std::string::npos);
    LOGOS_ASSERT_TRUE(paths.socket.size() + 1 <= kSunPathMax);
    // No problem reported: a home this long is not a problem, only a socket
    // would be. If this ever fails, the socket has started following the home.
    LOGOS_ASSERT_TRUE(paths.problem.empty());
}

LOGOS_TEST(a_git_path_that_does_not_exist_is_refused_and_named)
{
    // The negative case that makes the git setting meaningful. A test that only
    // set a valid git would pass against a store that ignored the value
    // entirely.
    ScratchSettings s("bad-git");
    SettingsStore store{s.path()};

    const auto result = store.set(SettingsStore::kKeyGitPath, "/definitely/not/here/git");
    LOGOS_ASSERT_TRUE(isError(result));
    LOGOS_ASSERT_CONTAINS(result["error"].get<std::string>(),
                          std::string("/definitely/not/here/git"));
    LOGOS_ASSERT_TRUE(store.get(SettingsStore::kKeyGitPath).empty());
}

LOGOS_TEST(an_empty_git_path_is_accepted_because_it_means_find_it_on_path)
{
    ScratchSettings s("empty-git");
    SettingsStore store{s.path()};

    const auto result = store.set(SettingsStore::kKeyGitPath, "");
    LOGOS_ASSERT_FALSE(isError(result));
}

LOGOS_TEST(an_overlong_socket_path_is_refused_at_set_time)
{
    ScratchSettings s("long-socket");
    SettingsStore store{s.path()};

    const std::string tooLong = "/" + std::string(120, 'x') + ".sock";
    const auto result = store.set(SettingsStore::kKeyRadSocket, tooLong);

    LOGOS_ASSERT_TRUE(isError(result));
    LOGOS_ASSERT_CONTAINS(result["error"].get<std::string>(), std::string("107"));
    LOGOS_ASSERT_TRUE(store.get(SettingsStore::kKeyRadSocket).empty());
}

LOGOS_TEST(a_socket_path_within_the_cap_is_accepted)
{
    ScratchSettings s("ok-socket");
    SettingsStore store{s.path()};

    const auto result = store.set(SettingsStore::kKeyRadSocket, "/run/user/1000/rad.sock");
    LOGOS_ASSERT_FALSE(isError(result));
    LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyRadSocket),
                    std::string("/run/user/1000/rad.sock"));
}

LOGOS_TEST(a_seed_url_without_a_scheme_is_refused)
{
    ScratchSettings s("bad-seed");
    SettingsStore store{s.path()};

    const auto result = store.set(SettingsStore::kKeyRemoteSeed, "seed.radicle.xyz");
    LOGOS_ASSERT_TRUE(isError(result));
    LOGOS_ASSERT_TRUE(store.get(SettingsStore::kKeyRemoteSeed).empty());
}

LOGOS_TEST(both_http_and_https_seed_urls_are_accepted)
{
    ScratchSettings s("seed-schemes");
    SettingsStore store{s.path()};

    LOGOS_ASSERT_FALSE(isError(store.set(SettingsStore::kKeyRemoteSeed,
                                         "https://seed.example")));
    // http is allowed deliberately: a local or LAN seed over plain http is a
    // real configuration, and refusing it would be this module inventing a
    // policy the seed API does not have.
    LOGOS_ASSERT_FALSE(isError(store.set(SettingsStore::kKeyRemoteSeed,
                                         "http://localhost:8080")));
}

// ---------------------------------------------------------------------------
// Where the file goes.
// ---------------------------------------------------------------------------

LOGOS_TEST(the_settings_path_follows_xdg_data_home_when_set)
{
    LOGOS_ASSERT_EQ(settingsPathFor("/xdg/data", "/home/u"),
                    std::string("/xdg/data/radicle-module/settings.json"));
}

LOGOS_TEST(the_settings_path_falls_back_to_the_xdg_default_under_the_user_home)
{
    LOGOS_ASSERT_EQ(settingsPathFor("", "/home/u"),
                    std::string("/home/u/.local/share/radicle-module/settings.json"));
}

LOGOS_TEST(two_basecamp_profiles_get_two_settings_files)
{
    // This is what keeps alice and bob apart, and it is the reason the path is
    // derived from XDG_DATA_HOME rather than from anything this module chooses
    // for itself: Basecamp already hands each profile its own.
    const std::string alice = settingsPathFor("/profiles/alice/xdg-data", "/home/u");
    const std::string bob   = settingsPathFor("/profiles/bob/xdg-data", "/home/u");

    LOGOS_ASSERT_TRUE(alice != bob);
    LOGOS_ASSERT_CONTAINS(alice, std::string("alice"));
    LOGOS_ASSERT_CONTAINS(bob, std::string("bob"));
}

LOGOS_TEST(no_environment_at_all_yields_no_settings_path)
{
    // Better than a path relative to nothing: an empty path makes the store
    // fall back to defaults in memory rather than writing somewhere arbitrary.
    LOGOS_ASSERT_TRUE(settingsPathFor("", "").empty());
}

LOGOS_TEST(a_store_with_no_path_still_reports_defaults_rather_than_failing)
{
    SettingsStore store{""};
    LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyMode),
                    std::string(SettingsStore::kModeExplore));

    // ...and refuses to write, naming the problem rather than silently
    // pretending the value was stored.
    //
    // A mode OTHER than the default, deliberately: writing the default would
    // leave the store reporting the same value whether the write was refused,
    // silently dropped, or genuinely applied, so the follow-up assertion could
    // not tell them apart.
    const auto result = store.set(SettingsStore::kKeyMode, SettingsStore::kModeLocal);
    LOGOS_ASSERT_TRUE(isError(result));

    // And nothing changed, which is what makes the refusal above mean
    // something rather than merely being reported.
    LOGOS_ASSERT_EQ(store.get(SettingsStore::kKeyMode),
                    std::string(SettingsStore::kModeExplore));
}

// "A store with no path reports defaults and refuses to write": the refusal
// names both variables `settingsPathFor` reads, rather than "could not write
// settings to " followed by an empty path, a sentence that ends on nothing a
// user can act on. `setRemoteSeed` passes this message on.
LOGOS_TEST(a_store_with_no_path_says_why_it_cannot_write)
{
    const auto result = SettingsStore{""}.set(SettingsStore::kKeyMode, SettingsStore::kModeLocal);
    LOGOS_ASSERT_TRUE(isError(result));
    auto message = result["error"].get<std::string>();
    LOGOS_ASSERT_CONTAINS(message, "XDG_DATA_HOME");

    // `HOME` is a substring of `XDG_DATA_HOME`, so naming only the latter would
    // satisfy a plain search for the former. Remove every occurrence of the
    // longer name first; what is left must still name `HOME` on its own.
    const std::string longer = "XDG_DATA_HOME";
    for (auto at = message.find(longer); at != std::string::npos; at = message.find(longer))
        message.erase(at, longer.size());
    LOGOS_ASSERT_CONTAINS(message, "HOME");
}

// NO SPEC: the spec requires the refusal to name the two variables and says
// nothing about how the sentence ends. This pins that it does not end on a
// space, the shape of "could not write settings to " with nothing after it.
LOGOS_TEST(the_refusal_of_a_store_with_no_path_does_not_end_on_a_space)
{
    const auto result = SettingsStore{""}.set(SettingsStore::kKeyMode, SettingsStore::kModeLocal);
    LOGOS_ASSERT_TRUE(isError(result));
    LOGOS_ASSERT_TRUE(result["error"].get<std::string>().back() != ' ');
}
