# specflow project overlay

<!--
Read in full by every specflow agent before its first shell command. It holds
this project's facts; it does not change specflow's rules. Replace each TODO.
"None." is a valid answer. /specflow:run refuses to dispatch while any line
reads TODO.

Optional: add a `## <role>` section (for example `## dev-writer` or
`## code-reviewer`) to give that role project-specific rules — typically the
project's security posture (what input is untrusted, where it is validated,
whether a crash is a denial of service) and its API conventions.
-->

Two modules in one repo: `radicle/` (core — all HTTP, JSON, filesystem and FFI,
with a Rust staticlib under `radicle/rust-ffi/`) and `radicle-ui/` (QML view,
forwards everything to core). `CLAUDE.md` above the specflow block is the long
form of everything here; read the section it names when a row points at it.

## Test layers

Pick the cheapest layer that can see the behaviour you changed. Every layer
runs on every PR.

| Layer | Command (one plain call, from the repo root) | Sees | Cannot see |
|---|---|---|---|
| QML syntax | `sh radicle-ui/tests/check-qml-syntax.sh` | a file Qt cannot parse | anything else; `qmllint` does **not** catch syntax errors |
| qmllint | `sh radicle-ui/tests/lint-qml.sh` | lint-category problems | runtime behaviour. Set `REQUIRE_QML_LINT=1` in CI only; locally it may skip when no binary is found |
| QML component tests | `qmltestrunner -input radicle-ui/tests/tst_<name>.qml` for one file; `sh radicle-ui/tests/run-qml-tests.sh` for all | one component in isolation: selection state, layout invariants, load ordering | wiring between modules, a real backend, the real clipboard, window geometry of the real app |
| Core module unit tests | `nix build ./radicle#checks.x86_64-linux.unit-tests -L --out-link result-test` | URL building, ref resolution, pagination, error shapes, local-profile detection; no network | the Rust `local*` path, the QML. CI runs the `cd radicle` form with the system from `nix eval`; the `./radicle#` form from the root is the cd-free equivalent and has not been run in this repo |
| Rust FFI tests | `cargo test --manifest-path radicle/rust-ffi/Cargo.toml` | the `local*` read path against real fixture profiles; the panic guard at the `extern "C"` boundary | the C++ side of the FFI; a changed `const` (see Mutation tool) |
| Rust format and lint | `cargo fmt --manifest-path radicle/rust-ffi/Cargo.toml --check`, then `cargo clippy --manifest-path radicle/rust-ffi/Cargo.toml --all-targets -- -D warnings` | formatting; dead code that is a never-wired safety guard | behaviour |
| End-to-end specs | `radicle-ui/tests/ui/*.yaml`, run through sitometres. **Read `docs/e2e.md` first** | the wiring: module loads, replica connects, data arrives, a click reaches a handler | window geometry (no step takes a size), the system clipboard (the run is `offscreen`), and anything under `local.yaml` without a seeded `RAD_HOME` |

- **Write the test with the code.** A UI feature ships with UI test coverage in
  the same change. A UI bug fix ships with a regression test that was watched
  failing first.
- **Logic that needs no view belongs in the core module**, testable without Qt.
- **`run-qml-tests.sh` covers around thirty files and its output truncates in a
  tool result.** A local pass is then a green you cannot read to the end, and a
  mutation was once recorded as survived because the output stopped before the
  mutated file. For an unambiguous answer about one file run
  `qmltestrunner -input radicle-ui/tests/tst_<name>.qml` and read all of it.
  Never `| tail`.
- **Make fakes return input-dependent data.** A fake that returns the same
  thing for every input cannot tell "reloaded" from "never reloaded"; deleting
  the whole `onBranchChanged` body once left every test passing.
- **Prefer `state:` over `text:` in a spec**, put `objectName` on the clickable
  element, and add every new spec to the `ui-tests.yml` matrix or it runs
  nowhere.

## Build

- **Both modules, both variants:** `lgs basecamp build --variant all`.
  Dev only: `lgs basecamp build --variant lgx`. One module:
  `lgs basecamp build --variant lgx --module radicle_ui`.
- **Output:** `.scaffold/basecamp/lgx/<NN>-<module>.lgx` (dev) and
  `.scaffold/basecamp/portable/<NN>-<module>.lgx`, as symlinks into the nix
  store; `NN` is the load-order index. Both directories are wiped on each
  build. Use `cp -L` for a real file.
- **Root from the cwd:** yes. `lgs` resolves `scaffold.toml`'s relative module
  refs (`path:./radicle#lgx`) against the project root it was run from, so from
  a worktree it builds that worktree's sources. `build` needs no
  `lgs basecamp setup`, and works in a worktree with no `.scaffold/` at all.
- **Install `lgs` at the version `LGS_VERSION` names in both workflows:**
  `cargo install logos-scaffold --version <LGS_VERSION> --locked`.
- **Raw `nix build` is for exactly two things**: the core unit tests (row
  above) and the e2e layer's dev Basecamp (`docs/e2e.md` step 1). Anything
  else build-shaped has an `lgs` verb; never hand-write an `--override-input`.
- **Keep `radicle.url = "path:../radicle";` on one line in
  `radicle-ui/flake.nix`.** Scaffold derives the sibling override from it with a
  line-based parser; the multi-line form is not an error and silently builds the
  UI against the locked pin instead of the working tree.
- **`LGS_VERSION`, the `scaffold.toml` Basecamp and `lgpm` pins, and the
  `logos-module-builder` tag in both `flake.nix` files move as one set.**
  `.github/scripts/check-stack-pins.py` checks that the copies agree.
- **Both `metadata.json` files carry the same version**; the Lint job asserts it.

## Mutation tool

- **Rust (`radicle/rust-ffi/`):** `cargo mutants`, scoped to the changed file
  with `--file <path>`. It mutates functions, not `const` values, so a changed
  constant is invisible to it. It is not wired into CI and is not installed by
  default; `cargo install cargo-mutants` if absent. No time budget has been
  measured here — scope it per file, and run it in the foreground within a limit.
- **QML and the C++ core: none.** The mutation is manual: delete the handler
  body, return a constant, change the condition, and see whether anything goes
  red. Use `qmltestrunner -input radicle-ui/tests/tst_<name>.qml` on the one
  file, so the result is readable to the end. **Restore after every mutation**,
  and do it only in your own worktree.
- **A mutation to a QML file will not fail the e2e layer** unless the e2e spec
  observes it; `branches.yaml` asserts `treeCount > 0`, which is as true of the
  old branch's entries left on screen as of the new one's.

## CI gates

Two workflows, both on `pull_request` to `main`. `ci.yml` also runs on pushes
to `main` and `v*` tags, **not on a push to a feature branch**: a branch gets no
run until its PR is open. `ui-tests.yml` runs on pushes to `main`, on PRs, and
on demand.

| Check (required on `main`) | What it runs | A green does not prove |
|---|---|---|
| `Lint` | metadata validity and equal versions; the Logos stack pins agree; no QML component shadows a Qt built-in; layout imports present | behaviour of any kind |
| `QML lint and component tests` | `check-qml-syntax.sh`, `lint-qml.sh` (`REQUIRE_QML_LINT=1`), `run-qml-tests.sh` (`REQUIRE_QML_TESTS=1`), then asserts one `Totals:` line per `tst_*.qml` | the wiring between modules. A `tst_*.qml` that the runner skips fails the count, but a test whose fake returns the same for every input passes whatever the code does |
| `Rust backend tests` | `cargo fmt --check`, `clippy -D warnings`, `cargo test`, then a floor on the passed count (a floor, not an exact count) | the C++ side; a changed `const`. The Nix module build compiles the crate with `doCheck = false`, so this job is the only place those tests run |
| `UI spec validation` | every `tests/ui/*.yaml` parses against the sitometres schema | that a spec passes, or that it is in the `ui-tests.yml` matrix |
| `Build LGX (linux-amd64)`, `(linux-arm64)`, `(darwin-arm64)` | the core unit tests (linux-amd64 only) and `lgs basecamp build --variant all`, then that the packaged `ui_qml` manifest keeps its `view` | that the module loads in Basecamp. The dev variant is built as a compile gate and never published |
| `sitometres browse` / `branches` / `source` / `sync` / `write` / `local spec` | one real-Basecamp run per spec, real QtRO transport, real seed calls for the first four | window geometry and the clipboard (see Test layers). `source.yaml` needs the seed's first repository to have a root `README.md`, `sync.yaml` needs one whose sync outlasts its early steps, so a spec can go red for a seed-side reason |

- **The e2e job is slow on a cold Nix store cache**, which is what a fork PR and
  the first run after a `BASECAMP_REV` change get.
- **The release job** runs on a `v*` tag, builds from the `build` matrix, and
  fails when a platform is missing rather than shipping a partial.
- **Signed commits are required on `main`, admins enforced, 0 approving reviews
  required.** The merge waits on green CI and on the owner.
- **Squash merge** is how pieces land; check merged-ness with `gh pr list`, not
  `git branch --merged`.

## Never commit

- `./tmp/` and `.claude/worktrees/` (specflow already excludes them).
- Build output: `result`, `result-*` out-links (including `result-basecamp` and
  `result-test`), `.scaffold/`, `dist/`, `generated_code/`, `target/`, and
  `basecamp/`, `ui-results/`.
- Test run output: `radicle-ui/tests/out.txt` and `radicle-ui/tests/results.txt`
  are ignored. **`radicle-ui/tests/q6.log` is tracked** (it came in with the
  commit "Actually run the QML component tests, locally and in CI") and looks
  like run output: do not add to it or regenerate it into a commit.
- `.cursor/` and `AGENTS.md` (generated by `lgs init`), and any
  `settings.local.json`.
- **Do not hand-edit `scaffold.toml` incidentally.** `lgs basecamp setup`
  rewrites it and strips every comment; run `git diff scaffold.toml` after any
  `setup` and `git restore scaffold.toml`. Nothing in the flow should need
  `setup`; only `install` and `launch` do.
- **Never `git add -A`**; a worktree collects the above.

## Hazards

Where this project fails without an error. First get the failure visible, then
guess.

- **Basecamp swallows QML errors.** A view that fails to compile, a plugin
  skipped for a missing manifest field, and a binding that is `undefined` all
  present as "clicking the app does nothing". Read Basecamp's own log first:
  `<app_logs_dir>/basecamp.log`, where `lgs basecamp paths alice` prints
  `app_logs_dir`; `scaffold.toml` already sets `QT_LOGGING_RULES` so import
  failures are printed. Under sitometres the log is in the throwaway user-dir,
  which `--user-dir <dir>` keeps after the run.
- **`qmllint` passes files Qt refuses to parse.** `check-qml-syntax.sh` is the
  gate for that, and it runs first in CI.
- **A binding does not update inside the handler that changed its source.**
  A handler that changes a property and then reads a binding derived from it
  sees the old value. Defer by one event-loop turn (a zero-interval `Timer`) or
  pass the new value explicitly. `RepoView.onBranchChanged` shipped dead this
  way with every gate green.
- **The `wantRid` / `wantBranch` / `syncEpoch` staleness guard is hand-written
  in `CommitsTab`, `IssuesTab`, `PatchesTab` and `ThreadView`**, and every copy
  written from memory dropped something. A fourth variant is the signal to
  reshape, not to add a test.
- **A `readonly property` alias to a child that does not exist is `undefined`
  with no complaint.**
- **Gate every write affordance on `getCapabilities().canWriteLocal`, never on
  `localAvailable`.** A profile can exist while its key is locked, and a compose
  box that cannot be submitted loses what the user typed. Writes are local node
  only; read `docs/writes.md` before touching `cobwrite.rs`, `LocalWriter` or a
  composer.
- **`guarded()` in `rust-ffi` exists solely to stop a panic unwinding through an
  `extern "C"` frame**, and every entry point goes through it;
  `tests/panic_guard.rs` is what answers "is it called everywhere". Read
  `docs/rust-ffi.md` before touching `radicle/rust-ffi/`, the `local*` read
  path, or `flake.nix`'s per-system handling.
- **Dev and portable halves must not be mixed.** A dev Basecamp with portable
  modules (or the reverse) opens to nothing and times out on the first step; the
  only sign is one `was installed for variant '<x>' which is not supported`
  line in Basecamp's log. The e2e layer pairs the dev `#app` with
  `lgs basecamp build --variant lgx`.
- **The sitometres version moves with the Basecamp pin.** 0.1.2 cannot drive
  Basecamp 0.3.0. Pin the version, never a range or `latest`. When the pin
  moves, re-run a spec on the current sitometres before assuming the app broke.
- **Never `VAR=value` or `env VAR=` prefix a command.** Environment belongs in
  `scaffold.toml`'s `[basecamp.env]` / `[basecamp.profiles.*]`, or in the shell
  profile for a per-developer value. The test scripts take no arguments and set
  their own environment; extend a script rather than inventing a flag.
- **`RAD_HOME` decides whether Local mode finds a profile**, and whether a write
  is real. `lgs`-launched Basecamps get their own `HOME`, so Local mode finds
  none unless `RAD_HOME` is exported by the shell. `write.yaml` and `local.yaml`
  run against a seeded throwaway profile; never point an interactive or test run
  at a real profile you care about. Two Basecamps on the same `~/.radicle`
  contend, and a lone `local.yaml` step-9 failure is not a code defect until no
  other instance is running.
- **The app starts in Explore.** Do not assert `localAvailable`, `canWrite` or
  `nodeIdentity` before a mode click in a spec, and prefer `wait_for` over
  `expect` after one.
- **A click on a row below the fold reports "clicked" and lands on nothing.**
  The failure appears a step later as a pane that never opened.
- **A spec not in the `ui-tests.yml` matrix runs nowhere.**
- **A `tomlq`-derived pin:** `[repos.basecamp].pin` in `scaffold.toml` is the
  only place the Basecamp rev is written; `ui-tests.yml` derives it. Do not add a
  literal.
- **Read files from your own worktree, not from a copy in context.** A worktree
  branches from `origin/main`, or from the runner's HEAD under
  `worktree.baseRef: "head"`; `docs/PLAN.md` should be read from `origin/main`.
- **`./tmp/` is the scratch space**, not `/tmp`; clean up after yourself.
- **This repo's own role agents live in `.claude/agents/`** and share names with
  the plugin's. Address the plugin's qualified, `specflow:<role>`.

## Extra stages

None.

## Lessons

None.
