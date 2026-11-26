## Stages

- [x] spec — `spec-writer`
- [x] design + code — `dev-writer`
- [x] tests — `tester`
- [ ] review: correctness — `code-reviewer`
- [ ] review: security — `code-reviewer`
- [x] review: readability — `code-reviewer`
- [ ] review: architecture — `code-reviewer`
- [ ] review: spec-test — `spec-test-reviewer`
- [x] review: design — `design-reviewer`
- [ ] findings all ticked, `findings/` deleted — `closer`
- [ ] `openspec validate --strict`, then `archive` — `closer`
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
- [x] 2.3 Pin the unspecified choices with `NO SPEC:` tests — an acceptance
      emits one `remoteSeedChanged` (also the control for the no-event
      assertions), no request is made for a URL with no scheme, a failed probe
      leaves the in-force seed reachable, and the stored URL has its trailing
      slash stripped; verify each is green after 2.2 and that the no-scheme,
      reachability and trailing-slash ones were red before it
- [x] 2.4 Correct the comments that describe `setRemoteSeed` as never storing
      or as rolling back (`radicle_impl.h`, `settings_store.h`,
      `settings_store.cpp`, the test comment on adoption) and the annotation in
      `docs/M3-embedded-node-plan.md`; verify with
      `git grep -n "setRemoteSeed" radicle/src docs` that no comment in
      `radicle/src` still says so. The plan document's pre-Phase-1 section
      (its lines on "persists no configuration at all") is frozen planning
      prose describing the state before the store existed, and is left as
      written; the annotation below it is what was corrected
- [x] 2.5 Record in `design.md` which test each guard turns red, and verify each
      claim by applying the mutation it names and watching that test fail

## Workflow follow-up

- The `tester` owns the final suite and may keep, adapt or remove the tests
  above.
- Archive after review, per `docs/OPENSPEC-ARCHIVE.md`.
