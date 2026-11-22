# Correctness review — scaffold-0-5

Dimension: correctness only. Method: read `git diff origin/main...HEAD`, checked
each claim against the code and against the pinned upstream sources where a
local clone exists, and re-derived the lock claims by parsing both
`radicle/flake.lock` versions. `gh pr checks 50` shows all 13 checks green
(Release skipped, as on any non-tag run).

- [x] **`dev-writer`** — `openspec/changes/scaffold-0-5/tasks.md:74` (task 3.3), `design.md` Decision 4 — the
      claim that exporting `RAD_HOME` before `lgs` restores Local mode under `launch` was never run
      **Scenario:** a developer follows the new CLAUDE.md / `docs/e2e.md` advice, exports `RAD_HOME`,
      runs `lgs basecamp launch alice`, and Local mode still finds no profile (for example if
      scaffold 0.5.0's `launch_env` or Basecamp's launcher scrubs or rewrites the variable). Design
      says outright that this "follows from the inheritance rather than from a launch with `RAD_HOME`
      set". The task box is left unticked for exactly this reason, so it blocks the merge as it stands.
      **Fix:** either run it (export in the shell that starts `lgs`, read `/proc/<pid>/environ` of the
      launched Basecamp for `RAD_HOME`) and tick 3.3, or reword every document that states it as
      fact ("restores", "sees the profile `RAD_HOME` names") to say it is inferred.
      **Severity:** medium (documents a workflow nobody ran; the box is open).
      **Measured:** not run by me either — it needs a compositor and a launched Basecamp.
      **Fixed** in the commit that ticks this box, by the reword route, not by running it (it needs
      `setup` in this worktree, a compositor, and an export into a real shell profile). CLAUDE.md
      "Local mode under `lgs basecamp launch`" now says the route is "inferred, not yet run" and
      how to check it from `/proc/<pid>/environ`; `docs/e2e.md`'s step-9 paragraph says
      "inferred ... not yet run". `design.md` Decision 4 already said so. Task 3.3 is rewritten to
      what was done (environment read back; the docs say inferred) and ticked, with the unrun part
      stated in it.

- [x] **`dev-writer`** — `ci.yml` `release` job (`lgx merge` step, ~line 666) — the one packaging change
      0.3.x introduces is never exercised by any gate on this PR
      **Scenario:** module-builder 0.3.0 adds an `assets/` root entry to the package (design Decision 6,
      Risks). `build` jobs pass, but they only `tar xzf` the manifest and grep `variants/<v>/`. The
      `release` job is `skipping` on a PR; it first runs on a `v*` tag, against `logos-package` master.
      If `lgx merge` rejects or drops `assets/` (or the per-platform packages disagree on it), the tag
      is pushed and the failure arrives after, with no assets published. The design's own Risks entry
      records this as "the next tag is the first real run".
      **Fix:** run `lgx merge` once by hand over the three-platform packages (or at least linux-amd64
      twice renamed) from this branch's build and record the result in tasks.md, or add a
      `tar tzf … | grep -c '^assets/'` check to the `stage` step so a lost entry fails the PR.
      **Severity:** low-medium (failure is loud, but late and on the release path).
      **Measured:** could not run — `~/src/logos-co/logos-package` is not cloned here.
      **Fixed** in the commit that ticks this box, both ways. Run by hand: the three per-platform
      artifacts from this PR's CI run 45011529504 (`gh run download`), merged with `lgx` built the way
      the release job builds it (`nix build github:logos-co/logos-package#lgx`, resolved to
      `4bdd904`). Both modules merged ("Merged 3 packages … (darwin-arm64, linux-amd64,
      linux-arm64)"); the merged `logos-radicle_ui-module.lgx` lists `assets/` and
      `assets/icon.png` beside all three variants, and the core package has no `assets/` in its
      inputs or output. Recorded as task 4.7 and in `design.md` Risks. And the release job now
      asserts `assets/icon.png` in the merged UI package, since `lgx` comes from a moving master and
      today's pass says nothing about the next tag's.

- [x] **`dev-writer`** — `docs/M3-phase0-probe_node_runtime.rs.txt:82` — present-tense claim the
      change made false
      **Scenario:** the probe's doc comment says the 166-byte path "is why `scaffold.toml` pins
      `runtime_dir`". `scaffold.toml` no longer pins it (this diff removes both lines). A reader
      grepping for `runtime_dir` lands on a comment describing config that does not exist.
      `docs/M3-phase0-findings.md:425,457` say the same, but those are dated findings and may stand as
      history; only the probe comment reads as current. `radicle/src/settings_store.h` and
      `docs/M3-embedded-node-plan.md` were updated, so this was a missed instance of the same sweep.
      **Severity:** low (documentation).
      **Fixed** in the commit that ticks this box: the probe comment now says `scaffold.toml`
      "pinned `runtime_dir` until scaffold v0.5.0's short per-profile default made the pin
      redundant". The two `M3-phase0-findings.md` passages keep their dated text and gain an
      italic "Dated:" note each, pointing at Decision 5.

## Checked and clean

- **`radicle/flake.lock`**: only the `logos-module-builder` subtree moved. `nixpkgs-fetch` is
  `b6018f87…` before and after, and the builder's own nixpkgs resolves (following
  `logos-module-builder.nixpkgs`) to `e9f00bd…` before and after, so the rustc/fetcher notes in
  `radicle/flake.nix` still hold. The 46k-line diff is the builder's transitive graph being
  rewritten, not a surprise root input. `radicle-ui` has no tracked lock (`git ls-files`), as the
  design says.
- **Line references into module-builder** (`radicle/flake.nix`): `mkLogosModule.nix:274-288` at tag
  `0.3.0` is `resolveExtInput` and resolves `value.input.packages.${system}.${pkgName}` as stated;
  `mkExternalLib.nix`'s `lib.isDerivation source` arm exists at 0.3.0. The structured
  `{ input = …; }` entry in this repo still matches the new `resolveExtInput`.
- **`ui-tests.yml` build step**: `nix path-info --derivation … ; nix build "$drv^*"` is sound under
  `set -eu` (a failed evaluation aborts the step before the build); `accept-flake-config`,
  substituters and trusted key are set globally by the install-nix step, so dropping the flag on the
  second command loses nothing. The sampler's EXIT trap leaves at most one orphaned `sleep 20`. Run
  44963625950 and the PR's current run both pass. `SITOMETRES` is `0.2.0` in `ui-tests.yml`,
  `e2e-env.sh` and `docs/e2e.md`; `LGS_VERSION` agrees in both workflows.
- **`sync.yaml` reorder**: traced against `SourceTab.qml`. The poll before the sync is valid
  (`checkForUpdate` needs only `lastSyncedCommit !== ""`); `syncAll()` clears `updateAvailable`, so
  the post-cancel `syncLabel === 'Download All'` holds once `syncedOnce` is dropped; `cancelSync()`
  does not touch `lastSyncedCommit`, and `finishSyncIfDone()` is guarded by `syncing`, so the new
  `lastSyncedCommit === '000…'` assertion holds. The five-minute poll Timer cannot fire inside the
  spec's window. One inherited flake remains (a repository small enough to finish before the cancel
  click would set `syncedOnce`), unchanged by this diff.
- **`source.yaml` scroll step**: `entryNames`, `entryCount` and `sourceTabItem` exist; the ListView
  lookup requires both `positionViewAtIndex` and a matching `count`, so a Repeater cannot match.
  The `split(",")` indexing would be wrong for a name containing a comma in a directory listed
  before README.md, but `entryNames` is the pane's own joined string and the failure is a loud
  throw/misscroll, not a silent pass. Not a finding.
- **`scaffold.toml`**: parses (CI `lgs` runs on it); `[basecamp.env]` is read only by `launch`, so no
  CI job sees `WAYLAND_DISPLAY`. The known trade-off that the line breaks an X11-only session is
  recorded in design Decision 5 and Risks with a delete-the-line instruction, so it is not opened
  as a box; it is a hardcoded uid plus a hardcoded socket name, the same per-machine cost as the
  `runtime_dir` pin it replaces.

## Not measured

No mutation testing was done: the change moves pins, workflow text, YAML specs and prose, and no
module source. The sitometres specs cannot run in this sandbox, so the `sync.yaml` and `source.yaml`
reasoning above is from reading the QML, backed by the green CI run, not a local execution.
