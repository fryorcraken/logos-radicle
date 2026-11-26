## Stages

- [x] spec — `spec-writer`
- [x] design + code — `dev-writer`
- [x] tests — `tester`
- [x] review: correctness — `code-reviewer`
- [x] review: security — `code-reviewer`
- [x] review: readability — `code-reviewer`
- [x] review: architecture — `code-reviewer`
- [x] review: spec-test — `spec-test-reviewer`
- [x] review: design — `design-reviewer`
- [x] findings all ticked, `findings/` deleted — `closer`
- [x] `openspec validate --strict`, then `archive` — `closer`
- [ ] CI green, title/body checked, PR merged — `closer`

# Tasks

## 1. Make the change easy

- [x] 1.1 Extract the per-key checks from `SettingsStore::set` into a public
      static `SettingsStore::validate(key, value)`, with `set` calling it, in a
      commit that changes no behaviour; verify the core unit tests are as green
      as before it (`nix build ./radicle#checks.x86_64-linux.unit-tests`)

## 2. `setRemoteSeed` stores the seed it accepts

- [x] 2.1 Write the failing tests first in `radicle/tests/test_radicle_impl.cpp`
      for every scenario of "A seed accepted by setRemoteSeed survives a
      restart" and "A seed setRemoteSeed refuses changes nothing", giving each
      module a scratch settings path; verify the acceptance, no-path, no-scheme,
      reachability and trailing-slash tests fail against the unchanged
      `setRemoteSeed`
- [x] 2.2 Rework `RadicleImpl::setRemoteSeed`: shape via
      `SettingsStore::validate`, probe a candidate `SeedClient`, store
      `remoteSeed`, and only then replace the client in force and emit
      `remoteSeedChanged`; verify every test from 2.1 passes
- [x] 2.3 Test an acceptance emitting one `remoteSeedChanged` (also the
      control for the no-event assertions), no request for a URL with no
      scheme, a failed probe leaving the in-force seed reachable, and the
      stored URL losing its trailing slash; verify each is green after 2.2 and
      that the no-scheme, reachability and trailing-slash ones were red before
      it. These were first `NO SPEC:` tests; the spec now requires each, and
      the markers are gone
- [x] 2.4 Correct the comments that describe `setRemoteSeed` as never storing
      or as rolling back (`radicle_impl.h`, `settings_store.h`,
      `settings_store.cpp`, the test comments, `Main.qml`'s seed picker) and
      the annotation in `docs/M3-embedded-node-plan.md`; verify with
      `git grep -n "setRemoteSeed" radicle/src radicle-ui/src docs` that no
      comment still says so. The plan document's pre-Phase-1 section (its
      lines on "persists no configuration at all") is frozen planning prose
      describing the state before the store existed, and is left as written;
      the annotation below it is what was corrected
- [x] 2.5 Record in `design.md` which test each guard turns red, and verify each
      claim by applying the mutation it names and watching that test fail

## 3. What answering means, and the normalised URL (review findings)

- [x] 3.1 Write the failing tests first for the new requirements: a 2xx body
      that is not a seed index (`[]`, `null`, `"ok"`, `42`, `{}`,
      `{"apiVersion":6}`, `not json`), a failure status with a seed-index body
      and with `{"error":5}`, `https://` and `http://` refused before any
      request, a `nid` that is absent or not a string, and the reply carrying
      the probe's `apiVersion` and `nid`; verify each defect test is red
      against the code before 3.2
- [x] 3.2 `SeedClient::probe` refuses anything but a JSON object with a string
      `apiVersion`; `getJson` surfaces a seed's `error` only when it is a
      string; `setRemoteSeed` shape-checks the URL as given and as stored, and
      replies with `SeedClient::nid()`; verify the core unit tests leave only
      the two sandbox socket-cap failures
- [x] 3.3 Test the store-write refusal for a store with a path whose file
      cannot be written, not only for a store with no path; and have a store
      with no path say why it cannot write
- [x] 3.4 Re-measure each ordering guard in `design.md` by moving the gate and
      naming the tests that go red, and the assertion that does

## 4. An error member, empty messages and credentials (review round 2)

- [x] 4.1 Write the failing tests first: a 2xx body carrying an `error` member
      (five bodies) refused with a non-empty string `error` and no other
      member; a failure status with `{"error":""}` refused the same way, at
      the module and at `SeedClient`; a seed URL with credentials refused by
      `setSetting` without echoing the secret, and by `setRemoteSeed` before
      any request; and a `SeedClient` probe that fails after a good one
      leaving nothing behind. Verify each defect test is red against the code
      before 4.2
- [x] 4.2 `probe` judges a body only when it is the seed's (a private `fetch`
      says whose it is) and builds every refusal itself; an `error` member
      refuses; an empty seed `error` falls back to the status; `validate`
      refuses userinfo in a `remoteSeed` authority. Verify the core unit tests
      leave only the two sandbox socket-cap failures
- [x] 4.3 Add the slash-only rows (`/`, `///`) to the malformed-URL table, with
      no-request and `setSetting`-reason assertions on every row
- [x] 4.4 Measure each new guard by mutation and record in `design.md` which
      test it turns red; correct the reason given for asking both URL forms,
      and record the probe bar's alternatives and its cost
- [x] 4.5 Say where the next editor reads it that the two doors store the seed
      in different forms (`radicle_impl.h`, the `setSetting` branch), and what
      the transport may share (`seed_client.h`). Comments only: no test can
      see them

## Workflow follow-up

- The `tester` owns the final suite and may keep, adapt or remove the tests
  above.
- Archive after review, per `docs/OPENSPEC-ARCHIVE.md`.
