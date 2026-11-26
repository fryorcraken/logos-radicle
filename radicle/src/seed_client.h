#pragma once

#include <cstdint>
#include <functional>
#include <nlohmann/json.hpp>
#include <string>

#include "http_client.h"

namespace radicle {

/**
 * Client for a Radicle seed node's read-only JSON API (radicle-httpd
 * `/api/v1`). This is the "remote" half of the module: it needs no local
 * `rad` install and no local node, and it can only ever read — the seed API
 * rejects writes with HTTP 405.
 *
 * Every method returns a JSON object. On failure that object is
 * `{"error":"..."}`, so callers have exactly one failure shape to check.
 *
 * **Keep this a value type.** `RadicleImpl::setRemoteSeed` copies the client
 * in force, probes the copy and only then moves it in, so that a refused seed
 * leaves the original untouched. That holds only while the transport is the
 * one thing a copy shares with its source. The transport may hold state of its
 * own — a test's fake records every request in one log, which is how a test
 * names the candidate's request — but it must not make the candidate's probe
 * change what the client in force reports; production's default calls
 * `httpGet`, which keeps nothing. Any other shared member, such as a response
 * cache behind a pointer or an in-flight request handle, would let the
 * candidate's probe reach the client in force, or would stop the copy compiling
 * far from the edit that broke it.
 */
class SeedClient {
public:
    /// Fetches a URL and returns its body. Injectable so tests can drive the
    /// client without a network; production uses the Qt HTTP client.
    using Transport = std::function<HttpResponse(const std::string& url)>;

    /// `seedUrl` is an origin such as "https://seed.radicle.xyz" (no /api/v1).
    explicit SeedClient(std::string seedUrl = "https://seed.radicle.xyz");

    /// Replace the transport. Passing an empty function restores the default.
    void setTransport(Transport transport);

    void setSeedUrl(const std::string& seedUrl);
    const std::string& seedUrl() const { return m_seedUrl; }

    /// Last observed API version from `/api/v1` ("" until probed).
    const std::string& apiVersion() const { return m_apiVersion; }
    /// Last observed node id from `/api/v1`; "" until probed, and "" when the
    /// index carried none or carried one that is not a string.
    const std::string& nid() const { return m_nid; }
    bool reachable() const { return m_reachable; }

    /// Fetch `/api/v1` and cache apiVersion + nid. -> the index object.
    ///
    /// The seed answers only with a 2xx reply whose body is a JSON object
    /// carrying a string `apiVersion` and no `error` member. Anything else — a
    /// transport error, any other status, a body that is not JSON, JSON that is
    /// not an object, an object with no string `apiVersion`, an object with an
    /// `error` member whatever else it holds — leaves `reachable()` false and
    /// `apiVersion()` / `nid()` empty, and returns this client's own
    /// `{"error":"<non-empty message>"}`, never the seed's body.
    nlohmann::json probe();

    nlohmann::json listRepos(const std::string& query, int64_t page, int64_t perPage);
    nlohmann::json getRepo(const std::string& rid);
    /**
     * Branches (refs/heads/*) for a repo, each with the name (prefix
     * stripped) and the commit it points at, plus which one is the repo's
     * default. No dedicated seed endpoint exists for this: getRepo() already
     * returns the full refs map (see resolveSha(), which reads the same
     * data), so this reuses that single request rather than adding another
     * round trip.
     * -> {"items":[{"name":"main","head":"<sha>"}],"default":"main"}
     */
    nlohmann::json listBranches(const std::string& rid);
    nlohmann::json getTree(const std::string& rid, const std::string& sha,
                           const std::string& path);
    nlohmann::json getBlob(const std::string& rid, const std::string& sha,
                           const std::string& path);
    nlohmann::json getReadme(const std::string& rid, const std::string& sha);
    nlohmann::json listCommits(const std::string& rid, const std::string& sha,
                               int64_t page, int64_t perPage);
    nlohmann::json getCommit(const std::string& rid, const std::string& sha);
    nlohmann::json listIssues(const std::string& rid, const std::string& status,
                              int64_t page, int64_t perPage);
    nlohmann::json getIssue(const std::string& rid, const std::string& id);
    nlohmann::json listPatches(const std::string& rid, const std::string& status,
                               int64_t page, int64_t perPage);
    nlohmann::json getPatch(const std::string& rid, const std::string& id);

    /**
     * Resolve a branch name / short SHA / empty string to a full 40-char SHA.
     *
     * This exists because the seed API refuses anything shorter: asking for
     * `tree/master` returns HTTP 400 ("invalid length (have 6, want 40)").
     * Callers pass whatever the user clicked and this sorts it out, consulting
     * the repo's head and refs. An already-40-char input is returned as-is.
     */
    std::string resolveSha(const std::string& rid, const std::string& ref);

private:
    /// What one GET under /api/v1 produced. `seedAnswered` says whose `body`
    /// is: the seed's own parsed 2xx reply when true, this client's
    /// `{"error":"<non-empty message>"}` when false. The two are otherwise the
    /// same type, and a seed's body can carry an `error` key of its own, so
    /// `isError(body)` alone cannot tell them apart.
    struct Fetched {
        bool seedAnswered = false;
        nlohmann::json body;
    };

    /// GET `path` under /api/v1 and parse, saying whose the result is.
    Fetched fetch(const std::string& path);

    /// `fetch(path).body`: the seed's reply, or {"error":...} on failure. For
    /// every caller that passes a seed's reply on rather than judging it.
    nlohmann::json getJson(const std::string& path);

    Transport m_transport;
    std::string m_seedUrl;
    std::string m_apiVersion;
    std::string m_nid;
    bool m_reachable = false;
};

/// Build `{"error": msg}`.
nlohmann::json makeError(const std::string& msg);

/// True when `j` is an error object produced by makeError().
bool isError(const nlohmann::json& j);

/// Percent-encode one path segment (leaves unreserved chars alone).
std::string urlEncode(const std::string& s);

/**
 * Wrap a bare JSON array from the seed into the module's paginated envelope
 * `{"items":[...],"page":N,"hasMore":bool}`.
 *
 * The seed returns plain arrays with no total count and no next-page link, so
 * `hasMore` is inferred: a full page probably has more behind it. That can
 * yield one empty final page, which is the standard trade-off for this shape
 * and is cheaper than an extra probing request per page.
 *
 * Passes `{"error":...}` objects through untouched.
 */
nlohmann::json paginate(const nlohmann::json& arr, int64_t page, int64_t perPage);

/**
 * Derive `{"items":[{"name","head"}],"default":"<branch>"}` from a repo
 * document's `refs.refs` map.
 *
 * **Seed-side only.** This was once shared with the local path, on the
 * reasoning that both sources produce a repo document in the same shape so one
 * filter should serve both. That turned out to be wrong about the local side:
 * `refs.refs` carries the *canonical* `refs/heads/*`, which in Radicle storage
 * is a single delegate-consensus ref, while every peer's branches — the local
 * node's included — live under `refs/namespaces/<nid>/`. Sharing this filter
 * made the local picker show exactly one branch per repository.
 *
 * `localListBranches` now calls a dedicated backend entry point
 * (`local::list_branches`) that reads those namespaces and returns a richer
 * shape (per-branch `remote` and `isLocal`, peer branches qualified by node
 * id). The duplication this comment used to warn against is real but narrow —
 * "strip `refs/heads/`, skip tags" — and is the price of the two sources
 * genuinely differing.
 *
 * Passes `{"error":...}` objects through untouched.
 */
nlohmann::json branchesFrom(const nlohmann::json& repo);

/**
 * Parse a raw JSON reply (as returned by `LocalReader::getRepo`) and derive
 * branches from it via `branchesFrom()`.
 *
 * Factored out of `RadicleImpl::localListBranches` so the "backend returned
 * something that isn't even JSON" branch — reported as `{"error":"malformed
 * reply from local storage"}` — is directly unit-testable with a hand-crafted
 * malformed string. The real Rust backend always returns valid JSON (it is
 * built with `serde_json`), so this path cannot be reached by driving the real
 * FFI in a test; it exists as a defensive check against a future backend
 * change or a corrupted read, and a free function is the only way to exercise
 * it without one.
 */
nlohmann::json branchesFromRawJson(const std::string& raw);

/**
 * Collapse a raw `LocalWriter::canWrite()` reply (`{"canWrite":bool,
 * "reason":"..."}` or `{"canWrite":true,"nodeId":"..."}`) into the two fields
 * `getCapabilities()` reports: whether a write is possible, and the reason a
 * view should show when it is not.
 *
 * -> {"canWrite":bool,"writeUnavailableReason":"..."}
 *
 * `writeUnavailableReason` MUST be "" exactly when canWrite is true — a view
 * that saw a non-empty reason next to `canWriteLocal:true` would show a
 * contradiction on screen (an explanation for a write that is, in fact,
 * available). A malformed/unparseable raw reply is reported as
 * `{"canWrite":false,"writeUnavailableReason":"<a stated reason>"}` rather
 * than propagated as an `{"error":...}` — `getCapabilities()` itself never
 * fails; "the backend could not explain itself" is one more reason a write
 * isn't currently possible, not a failure to answer the capabilities query.
 *
 * Factored out of `RadicleImpl::getCapabilities()` so this collapse — which
 * needs a controllable `canWrite()` reply to test both branches, and
 * `LocalWriter` has no fake seam of its own — is directly unit-testable with
 * a hand-crafted probe string instead of only being reachable through a real
 * signing key.
 */
nlohmann::json writeCapabilityFrom(const std::string& rawProbe);

} // namespace radicle
