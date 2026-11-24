# Working on this repository

## Where to look for what

This file is the always-relevant part: how to build and run things, the test
layers, and the traps that have bitten changes here. The areas below have their
own file, because they only matter when you are doing that specific thing.
**Read them when the trigger applies, not otherwise:**

| Read | When |
|---|---|
| [`docs/rust-ffi.md`](docs/rust-ffi.md) | Touching `radicle/rust-ffi/`, the `local*` read path, `flake.nix`'s per-system handling, or a test that needs a Radicle home; or deciding what a platform supports |
| [`docs/writes.md`](docs/writes.md) | Touching the `local*` **write** path — `cobwrite.rs`, `LocalWriter`, the composers — or adding a write action |
| [`docs/e2e.md`](docs/e2e.md) | Running, adding to, or debugging a sitometres spec (`radicle-ui/tests/ui/*.yaml`) |
| [`docs/OPENSPEC-ARCHIVE.md`](docs/OPENSPEC-ARCHIVE.md) | **Archiving an OpenSpec change** — run as a commit on the piece branch, before CI and the merge |

**Changes go through the specflow plugin's spec-driven flow** — an OpenSpec
change under `openspec/changes/<name>/`, one branch and one PR per piece. What
to read before dispatching or starting one is the specflow block at the end of
this file, and this repo's facts for every role are
[`.claude/specflow/PROJECT.md`](.claude/specflow/PROJECT.md). **What is not
built yet lives in GitHub Issues and milestones**, and a change's input is its
issue; no file in the tree holds the roadmap. A shipped change's proposal,
design and tasks are archived under `openspec/changes/archive/`, which is where
a past decision is found; `openspec/specs/` is the live behaviour contract. The
CLI is `openspec`, from the npm package `@fission-ai/openspec` — note the bare
`openspec` package is an unrelated placeholder, so install with the scoped
name.

`docs/` also holds background documents: `M2.2-write-actions-design.md`
(shipped), `M2.2-write-features-proposal.md`, and `M3-embedded-node-plan.md`
with `M3-phase0-findings.md` beside it. **`M3-embedded-node-plan.md` is M3's
own working document and is still maintained** — each step adds its findings
there, and it cites the crate source line by line, which is what makes a claim
re-verifiable rather than re-derivable. It holds evidence, not scope: where it
and an issue disagree about what is ahead, the issue wins. Which phases have
landed is a `git log` question; this line deliberately does not repeat it,
because it used to say "not started", which was true when written and false
two merges later.

### Keeping this file true

This file was once more than twice its current length, about a third of it a
session-by-session changelog. That third was deleted, not because the writing
was bad but because it had quietly become **false**: it claimed a milestone was
unmerged that had shipped two releases earlier, quoted a test count off by more
than half, and named a "next step" long since done. Every one of those was true
on the day it was written. That is the failure mode to design against — not
carelessness, but accuracy with a short half-life.

The rules that follow from it are the specflow block's "Keeping documents
true", at the end of this file: name the command rather than the number, write
down only what a command cannot tell you, make recorded state
self-invalidating, and prune as you go. Here the current version is a read of
`radicle/metadata.json` (both must match), and what a spec asserts is a read of
its `.yaml`. When a milestone ships, its planning prose shrinks to the durable
lessons and a pointer, not a new "what landed" entry beneath the last one.

A status file — gitignored or otherwise — is not the answer, and was considered
here. Gitignored, it is invisible to review, missing from every fresh clone and
worktree, and free to drift exactly like the prose it replaced. Tracked, it is
the same changelog in a new location. The fix is not a better place to record
derivable facts; it is to stop recording them.

## What this is, and where it stands

Two modules in one repo, published to the Logos catalog from GitHub:
`radicle/` (core — all HTTP, JSON, filesystem and FFI) and `radicle-ui/`
(QML view, forwards everything to core). The split is forced: Basecamp
sandboxes the QML engine, so a view cannot fetch or read anything itself.

The API names its source explicitly rather than hiding it behind one call —
`remote*` proxies to a seed node over HTTP, `local*` reads this machine's
`~/.radicle` through the Rust staticlib. They are different questions with
different answers, so they stay different methods; the JSON shapes they return
are identical, so a view renders either without branching. Every method returns
a JSON string, and failures are always `{"error":"..."}`.
`radicle/src/radicle_impl.h` is the full contract.

**Reading works everywhere:** browsing any public repo through a seed (search,
paging, file tree, blob viewer, README, sync-to-cache, commits with diffs,
issues and patches with full discussion), branch switching, sync staleness
detection, and the same surface again against the local node.

**Writing has begun, local node only** — commenting on an issue and creating
one, covered end to end. **Gate every write affordance on
`getCapabilities().canWriteLocal`, never on `localAvailable`**: a profile can
exist while its key stays locked, and a compose box that cannot be submitted
loses whatever the user typed. What else is deferred, and why, is in
[`docs/writes.md`](docs/writes.md).

**Both modules carry the same version, and `ci.yml` asserts it** — a bump
touching one `metadata.json` and not the other fails the metadata lint.

Two facts about releasing that are not recoverable from git history, both
consequences of this repo being a monorepo (two modules under one git repo)
where the catalog expects one module per submodule:

- **The catalog's release action must support a `module_path` pointing *inside*
  a submodule.** Older versions fail checkout with "pathspec did not match any
  file(s) known to git" for this layout. The fix is commit `974960591a8d`,
  which is the tip of tag `v1.3`, and the catalog's `_release-module.yml` is
  pinned to `@v1.3` (catalog commit `06078c6`). Upstream has since advanced the
  moving `v1` tag to the same commit, so `@v1` would also work; `@v1.3` is kept
  deliberately, because a moving tag would let the release pipeline change
  under the catalog without a commit here saying so.
- **The catalog's `release-all.yml` auto-discovery cannot see this repo's two
  modules**, because it reads module paths straight from `.gitmodules`
  submodule paths. `radicle` and `radicle_ui` each need their own
  manually-triggered workflow, not the umbrella.

## How to work in this repo, and what Bash costs

**Reach for `lgs` for anything build-, run- or install-shaped.** This is a
scaffold-managed project; the next section is the verb table. Raw `nix build`
has exactly **two** legitimate uses here — the core module's unit tests,
because `lgs` has no verb for a flake's `checks` outputs, and the e2e layer's
dev Basecamp (see "The two things `lgs` genuinely does not cover" below). If
you find yourself writing a `nix build` pair with a hand-written
`--override-input`, or `mkdir dist && cp result-*/*.lgx`, stop — `lgs` already
does that, and does it in dependency order.

**What Bash costs is the specflow block's "How to work here, and what Bash
costs"**, at the end of this file: which shapes prompt, why never to chain, why
`Edit` beats `sed -i`, why `gh` is free until filtered. Its rules are not
repeated here. What this repo adds:

- **`lgs …`, `nix build …` and `sh radicle-ui/tests/…` are allow-listed** in
  `.claude/settings.json`, so a plain call to any of them costs nothing;
  reading a path under `/nix/store` does, and so does any of them behind a
  `cd … &&`.
- **Agents get their tree from `isolation: "worktree"`, which depends on
  `.claude/settings.json` carrying `{"worktree": {"baseRef": "head"}}`.** That
  file is tracked; `.gitignore` says why beside the rule. Why a dispatched
  agent never calls `EnterWorktree` is the plugin's
  `specflow:worktree-discipline` skill. **Settings are the user's — do not
  write that file on your own initiative.**
- **Never `readlink` or `ls` a `/nix/store` path** to find out where an
  artefact went. The paths are documented below; use them.
- **`curl https://crates.io/…` to check a version exists** is the standing
  example of predicting a command instead of running it: blocked from this
  sandbox (it returns empty, which reads as "not found") and pointless, since
  `cargo install <crate> --version X` answers the same question by succeeding
  or failing, and leaves you with the tool installed.
- **The test scripts take no arguments and set their own environment**, so
  `sh radicle-ui/tests/<script>` is one plain, allow-listed call. Extend a
  script rather than inventing a flag or a `VAR=value` prefix.

Scratch files go in `./tmp/`, as the specflow block says; `.gitignore` already
excludes it.

## This is a scaffold-managed project — reach for `lgs` first

`scaffold.toml` at the root is a `logos-scaffold` (`lgs`) config, and both
modules are captured in it as `[modules.radicle]` / `[modules.radicle_ui]` with
`role = "project"`. That means the build, install, launch and dev-shell paths
all have an `lgs` verb, and reaching straight for `nix build` skips the part
scaffold does for you — resolving each module's flake ref, ordering the two
builds by dependency, and deriving the sibling `--override-input`.

**Version: a plain crates.io release.** Both CI workflows pin the same one in
`LGS_VERSION`; install that:

```bash
cargo install logos-scaffold --version <LGS_VERSION> --locked
```

This repo was pinned to a commit on the unmerged
[logos-co/scaffold#266](https://github.com/logos-co/scaffold/pull/266) for two
flags, and needs neither now:

- `basecamp setup --inspector` selected the Basecamp bundle with the QML
  inspector. The e2e layer no longer uses a bundle — the dev `#app` already
  has the inspector. See [`docs/e2e.md`](docs/e2e.md).
- `--print-output` on `basecamp build` is sugar for
  `LOGOS_SCAFFOLD_PRINT_OUTPUT=1`, which `run_logged` reads on every build
  path whatever the subcommand. Both workflows set the env var instead, which
  works on the release.

**The lesson from that pin is worth keeping.** #266 moved three times while
this repo tracked it, and one of those moves *reverted* a feature already
written down here as fixed — `save_project_config`'s comment preservation was
pulled out into a separate branch, so bumping the pin silently reintroduced
`scaffold.toml` comment stripping. A released version cannot change under you;
an open PR branch can, and did.

For background on the release line: v0.3.0 moved the module table from
`[basecamp.modules.*]` to the top-level `[modules.*]` this repo uses, and added
`develop`, `build`, `run` and `paths`. v0.4.0 narrowed scaffold to supporting
**one Basecamp release at a time** (0.3.0 since then). **The verb table below
was checked against released v0.5.0** with `lgs basecamp --help`.

**`LGS_VERSION`, the `scaffold.toml` Basecamp and `lgpm` pins, and the
`logos-module-builder` tag in both `flake.nix` files move as one set.** Scaffold
names the one Basecamp it supports in `DEFAULT_BASECAMP_RELEASE` (its
`src/constants.rs`), its `lgpm` pin is the rev that Basecamp locks, and the
module-builder generation decides which SDK the modules link, which Basecamp
then accepts or refuses at runtime. Bumping one without the others builds
cleanly and fails in the e2e layer, or not at all until a user loads the
module. So a stack bump starts from a scaffold release, not from a Basecamp
release. The reasoning is in the `scaffold-0-5` change's `design.md`, under
`openspec/changes/archive/` once it has merged.

Do not be misled by `version = "0.2.0"` at the top of `scaffold.toml`: that is
the **file schema** version, not the CLI version, and `0.2.0` is what `lgs`
expects (`SCAFFOLD_TOML_SCHEMA_VERSION` in scaffold's own source, unchanged
from 0.3.x through 0.5.0). Bumping it to match the CLI would make the file
unparseable.

A build of scaffold from a clone can report the previous version: scaffold
bumps the version string only in its release commit, so `--version` on a master
build between two releases names the older one. If a command below seems
missing, check whether your `lgs` is a path install (`cargo install` prints
the package source when it replaces one) before assuming you are behind.

### The verb table

| You want to | Run |
|---|---|
| build both modules, both variants | `lgs basecamp build --variant all` |
| build just the portable artefacts (AppImage, release) | `lgs basecamp build-portable` |
| build just the dev artefacts (what the e2e layer runs) | `lgs basecamp build --variant lgx` |
| check one module still compiles | `lgs basecamp build --variant lgx --module radicle_ui` |
| install into the dev profiles | `lgs basecamp install` |
| launch Basecamp for manual testing | `lgs basecamp launch alice` |
| enter a module's dev shell | `lgs basecamp develop radicle_ui` |
| find a profile's log / module / XDG dirs | `lgs basecamp paths alice` |
| check `[modules.*]` against installed state | `lgs basecamp doctor` |
| read the module-project contract | `lgs basecamp docs` |

`build` takes `--variant {lgx,lgx-portable,all}` and an optional
`--module <name>`; `build-portable` is the alias for `--variant lgx-portable`.
Both read `[modules.*]`, build in dependency order, and **derive
`--override-input radicle path:../radicle` for `radicle-ui` themselves** — never
write that flag by hand.

### Where the artefacts land

Deterministic, and worth knowing so you never probe the filesystem for them:

```
.scaffold/basecamp/lgx/<NN>-<module>.lgx        # dev    (--variant lgx)
.scaffold/basecamp/portable/<NN>-<module>.lgx   # portable
```

Concretely `01-radicle.lgx` and `02-radicle_ui.lgx` in each. `NN` is the
load-order index, so the numbering *is* the dependency order. They are
**symlinks into the nix store**; both directories are wiped and recreated on
each build. Use `cp -L` if you need a real file.

Note the two variants land in **separate directories**, so `--variant all`
gives you per-module, per-variant addressing — not one flat directory.

### `build` does not need `basecamp setup`

`setup` pins and builds the basecamp + lgpm binaries and seeds the `alice` /
`bob` profiles. A *build* touches none of that, and `lgs basecamp docs`
explicitly supports a hand-authored `[modules.*]` table in "CI / sandboxed
environments where `lgs basecamp setup` can't or shouldn't run". Verified here:
`lgs basecamp build --variant all` runs green in a fresh worktree with no
`.scaffold/` at all. Only `install` and `launch` need `setup`.

### Environment goes in `scaffold.toml`, not on the command line

Do not prefix commands with `VAR=value`. It trips the permission checker (an
approval click every time) and it puts configuration somewhere nothing else
can see it.

`lgs` reads `[basecamp.env]` and `[basecamp.profiles.<name>]` from
`scaffold.toml` and exports them to the Basecamp process itself. That is
already where `QT_LOGGING_RULES` and `QT_FORCE_STDERR_LOGGING` live — which is
why QML import failures are visible at all. If you need a variable set for a
run, add it there; then every future launch has it too, and the reason can be
written down beside it.

The same principle covers the test scripts: `run-qml-tests.sh` and
`check-qml-syntax.sh` take no arguments and set their own environment
deliberately. Extend that when adding a script rather than inventing flags.

**Except a value that belongs to one developer or one session, not to the
project.** `scaffold.toml` is tracked, so your home directory, your uid and
your desktop session do not go there. Such a value goes in the environment of
the shell that runs `lgs`, from your shell profile (still not a `VAR=value`
prefix). `launch` adds its own names on top of the environment it inherits and
clears nothing, so a name scaffold does not write reaches Basecamp unchanged
(`launch_env` and `apply_launch_env_overrides` in v0.5.0's
`src/commands/basecamp.rs`; observed for `DISPLAY` in a launched instance's
`/proc/<pid>/environ`). There are two such values here: `RAD_HOME` (next
section) and the Wayland socket (below).

**There is no `runtime_dir` on the profiles, on purpose.** Scaffold v0.5.0's
default runtime dir is short, private and per profile, which is everything the
old `/run/user/1000` pin was for, without sharing one directory between `alice`
and `bob`. Do not add one back to fix a launch that exits at once with "Failed
to create wl_display". The cause is the Wayland socket: libwayland resolves the
session's relative `WAYLAND_DISPLAY` (`wayland-0`) against `XDG_RUNTIME_DIR`,
which `launch` points at the private dir, where there is no compositor; and
Basecamp's launcher forces Qt onto Wayland whenever `WAYLAND_DISPLAY` is set.
Make the value absolute in your shell profile:

```sh
case "$WAYLAND_DISPLAY" in ''|/*) ;; *) export WAYLAND_DISPLAY="$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY" ;; esac
```

It does nothing on an X11 session, where `WAYLAND_DISPLAY` is unset, and it
needs no uid written down. That absolute value is the configuration measured
working for `alice` and `bob` side by side; delivering it from the shell rather
than from `scaffold.toml` follows from the inheritance above and has not itself
been launched. Why it lives in your shell is Decision 5 of the `scaffold-0-5`
design (`openspec/changes/archive/`).

### Local mode under `lgs basecamp launch`

Since scaffold v0.5.0, `launch` gives each profile its own `HOME`
(`.scaffold/basecamp/profiles/<name>/home`). `LocalStore` reads `RAD_HOME`,
else `$HOME/.radicle`, so **an `lgs`-launched Basecamp in Local mode finds no
profile** — the Local segment annotates itself as unavailable, exactly as it
does under sitometres. Nothing is broken; the module is reading the home it
was given.

To browse a node from an interactive launch, export `RAD_HOME` from your shell
profile, per the exception above. **That this reaches the module is inferred,
not yet run:** it follows from the inheritance, but no launch has been driven
into Local mode with `RAD_HOME` exported. If Local mode still finds no profile,
read `RAD_HOME` back from the launched Basecamp's `/proc/<pid>/environ` before
suspecting the module.

**The export also makes writes real, in every instance.** It reaches `alice`
and `bob` alike, and Local mode is where the compose boxes write: with the key
unlocked, `canWriteLocal` is true, and a comment or issue posted from either is
signed with that profile's identity and stored in it. Before scaffold v0.5.0
that was the default; now this export is the only thing that opts into it. To
try a write, point `RAD_HOME` at a throwaway profile, seeded the way
`write.yaml`'s is ([`docs/writes.md`](docs/writes.md)), not at your own.
Pointing an interactive instance at your real profile also brings back the
two-Basecamps contention `local.yaml`'s step 9 is sensitive to; see
[`docs/e2e.md`](docs/e2e.md).

### `basecamp setup` strips the comments from `scaffold.toml`

It rewrites the file, and the rewrite drops every comment. This repo's
`scaffold.toml` carries load-bearing ones: why the launch environment is shaped
the way it is, and the record of the `sun_path` and Wayland bugs that each cost
a debugging session. Losing them once re-opened one of those bugs.

**Still true on v0.5.0, measured rather than read off release notes:** a
`setup` run there changed no value and deleted every comment line
(`git diff scaffold.toml` shows it). The source agrees. `cmd_basecamp_setup`
ends in an unconditional `save_project_config`, which v0.5.0's
`src/project.rs` documents as "a destructive serialization", and the only
comment-preserving writer is `init`'s.

The history is worth reading as a caution about pinning to an open PR branch.
Scaffold#266 briefly grew a comment-preserving writer. Its author then split
that work out as #267 (`fix/preserve-scaffold-toml-comments`), which was closed
unmerged, so the stripping was never fixed in a release.

**Only `setup` does this, and only `install`/`launch` need `setup`** — no CI
job runs it any more, and neither does the e2e layer. It bites during
interactive work.

**So: run `git diff scaffold.toml` after any `setup` and restore what it
destroyed** (`git restore scaffold.toml` — `setup` changes no value on a
project already on scaffold's default pins). The comment block in
`scaffold.toml` says this itself. If a later release stops stripping, the
place to check is `save_project_config` in that tag's `src/project.rs`.

### The two things `lgs` genuinely does not cover

**The core module's unit tests.** There is no verb for a flake's `checks`
outputs, so that stays `nix build '.#checks…'`.

**Building a Basecamp without seeding profiles.** The e2e layer needs the dev
`#app` binary and nothing else; every `lgs` path to a Basecamp goes through
`setup`, which also builds lgpm, seeds `alice`/`bob`, and rewrites
`scaffold.toml`. So `docs/e2e.md` step 1 is a raw `nix build` to a local
out-link.

Both are legitimate and neither is a sign you should hand-roll the rest.
Everything else has a verb. Use it.

### The Basecamp the e2e layer builds

**It is the dev `#app`, not a bundle.** The QML inspector is compiled *out* of
the shipping outputs (`#bin-bundle-dir`, `#bin-appimage`, `#bin-macos-app`) and
*in* to the dev build — upstream's own inspector-driven checks use `app`. This
file previously asserted the opposite, and that error is what put the whole
repo on an unmerged scaffold branch for `setup --inspector`. The evidence,
including the control that proves the probe discriminates, is in
[`docs/e2e.md`](docs/e2e.md).

**`[repos.basecamp].pin` is the single source of truth for the rev.**
`ui-tests.yml` derives `BASECAMP_REV` from it with `tomlq` in the "Resolve the
pinned Basecamp rev" step rather than carrying a literal. It used to carry one,
and the divergence would have been invisible — the store cache would key on the
stale rev, restore a store that cannot help, and recompile Basecamp on every
run forever without anything failing.

Two ordering constraints on the resolve step: **after** the apt step (which
installs `tomlq`), and **before** the cache step (which keys on what it
exports). Note the runner's preinstalled `yq` is the **Go** one and has no
`tomlq`; the apt `yq` is the Python one that does, so the install is not
redundant.

**`setup` still strips `scaffold.toml` comments** on the released version.
`git diff scaffold.toml` after every `setup` and restore the comment block; see
the section on it above.

### Upstream asks against logos-scaffold

**This repo now runs entirely on released scaffold, and has one open ask.**

- **`basecamp setup` should preserve `scaffold.toml` comments** — the one still
  open, and still live on v0.5.0. Briefly fixed inside #266, split out as #267,
  and #267 was closed unmerged, so there is no upstream PR to wait on. Not
  re-filed from here yet. Check the tracker before filing, because #267's
  thread holds the reviewer's objections a new attempt would have to answer.
  See the section above.
- ~~Select the inspector bundle through scaffold~~ —
  [#265](https://github.com/logos-co/scaffold/issues/265), **withdrawn as
  based on a false premise, not fixed.** It asked scaffold to classify
  `bin-bundle-dir-inspector` as the portable stack, on the belief that it was
  the only Basecamp output with the QML inspector. The dev `#app` has the
  inspector too, so this repo needed no scaffold change at all — the whole
  dev-vs-portable classification problem only arose from choosing the bundle.
  The inspector-in-a-*bundle* case is still real for upstream (the
  package-manager doctest, macOS `.app` integration tests); it was never real
  here.
- ~~`basecamp build` should pass build logs through~~ — moot. `--print-output`
  landed on #266's branch, but the `LOGOS_SCAFFOLD_PRINT_OUTPUT=1` it sets is
  read by `run_logged` on every build path in v0.3.1 already. Both workflows
  set the env var.

**A cheap check worth repeating before filing anything upstream:** read the
dependency's source for what you actually need. Both retracted asks above
would have been avoided by ten minutes in `flake.nix` and `process.rs`, and
between them they kept this repo pinned to an unmerged PR branch — through
three force-pushes and one silent feature revert — for no gain.

The remaining scaffold-shaped wish is small: nothing exposes a flake's `checks`
outputs, so the core module's unit tests stay on raw `nix build`. Not filed —
it is a different feature area from anything above.

### The bundled `basecamp` skill may document an older release

`lgs init` generates `.claude/skills/`, `.cursor/` and `AGENTS.md`, and none of
it refreshes when you upgrade `lgs`. So the
copy on disk documents whichever release last ran `init` — which may predate
the `[modules.*]` schema and the `develop` / `build` / `run` / `paths` verbs,
making it read as though raw `nix` were the only way to do anything.

That is version skew, not a defect: the file correctly describes the release it
came from, and upstream's v0.3.x skill is already correct. Nothing to report
upstream — re-run `lgs init` to refresh it. Until then, treat
`lgs basecamp --help` as authoritative, then this file.

### If you are working in a git worktree

This is the `lgs` half only. For the git-level mechanics — where worktrees
live, what they branch from, the shared stash stack, and cleaning them up —
see "Working in a git worktree" below.

A worktree has no `.scaffold/` — that directory is untracked, so it does not
come along.

**`build` and `build-portable` work anyway**, creating `.scaffold/basecamp/`
as they go (see "`build` does not need `basecamp setup`" above). Only
`install`, `launch` and `doctor` need the binaries and seeded profiles, so run
`lgs basecamp setup` in the worktree itself when you want those — and check
`git diff scaffold.toml` afterwards, because `setup` strips its comments.

What none of this means is that you are building the main tree's code.
`scaffold.toml`'s module refs are relative (`path:./radicle#lgx`), and `lgs`
resolves them against the project root it was invoked from — so from a
worktree, every verb acts on that worktree's sources. The consequence is
isolation, not a wrong-code hazard: the worktree's `launch alice` is a
*separate* Basecamp instance with its own profile state, not the one the main
tree launches.

## Working in a git worktree

Non-trivial work happens in a worktree, on its own branch — that is how M1.1,
M2.1 and most of what followed were built. **The rules are the specflow
block's "Worktrees go in `.claude/worktrees/`"** at the end of this file —
`--no-track`, the `git config` check, the shared stash stack, pruning — and the
plugin's `specflow:worktree-discipline` and `specflow:run` skills for agent and
runner trees. The `lgs`-specific half is under "If you are working in a git
worktree" in the scaffold section, and is worth reading before you run any
verb from one.

Each of those rules was measured here first, which is why they are rules:

- **A worktree's base matches neither local `main` nor necessarily the commit
  you expect.** The main tree once sat at `6771497`, `origin/main` at
  `b8e50f4`, and a worktree created between them came off a third commit. A
  stale `CLAUDE.md` is the likeliest thing to mislead you, because it is in
  context from the start and rarely re-read.
- **Without `--no-track`, a bare push targets `main`.** `git config
  --get-regexp "^branch\.piece"` returned `merge refs/heads/main` for both
  piece branches created without the flag, and `git push origin
  piece/embedded-node-wizard` was **rejected by branch protection for
  `refs/heads/main`**.
- **Nothing prunes worktrees.** This repo reached **15** at once, most on
  branches merged milestones ago — M1.1's `worktree-agent-a42af4ef65b0dcc3b`
  was still on disk long after PR #2 merged, two worktrees shared one branch,
  and one sat on a stale detached HEAD. Each holds a branch ref alive, so
  merged branches never look merged, and a stale one invites edits to code
  that is no longer anybody's `main`.

One consequence to expect rather than debug: a worktree's `lgs basecamp launch
alice` is a **separate** Basecamp instance with its own profile state. That is
isolation working as intended — but see the `local.yaml` caveat about two
Basecamps contending on the same `~/.radicle`, which that isolation does *not*
protect you from.

## How to shape a change: this repo's worked examples

The three habits — make the change easy, then make the easy change; put the
complexity in the data structure; one function, one job — are the specflow
block's "How to shape a change", at the end of this file. What follows is
where each one was learned here, which the block does not carry.

### Make the change easy, then make the easy change

A diff that reshapes and alters behaviour at once cannot be reviewed for
either, and cannot be reverted without losing the half you wanted.

This repo has the worked example. `Main.qml`'s `nav.busy`/`nav.error` logic
had no test because it was tangled into a 300-line file; the fix was not a
cleverer test but pulling it out into `NavState.qml` — after which the test
was obvious and `tst_nav.qml` fell out in minutes. Same story with
`SourceState.qml`, extracted from the Seed/Local routing so the method
selection could be tested without a view. **When a test is hard to write,
suspect the shape of the code before blaming the test layer.**

The counter-pressure, which is equally real: do not refactor speculatively.
`IssuesTab.qml` and `PatchesTab.qml` stayed near-identical and unmerged for
three milestones, because nothing needed them to diverge and no third consumer
appeared. Issue #28 records the point at which that stopped holding. Make room
for *the change in front of you*, not for one you imagine.

### Put the complexity in the data structure, not the logic

A branch has to be got right at every call site and tested at each one; a data
shape is right everywhere at once, and a new call site inherits it for free.

The `wantRid`/`wantBranch`/`syncEpoch` staleness guards are this repo's
standing example — and its standing counter-example. The same guard is
hand-written slightly differently in `CommitsTab`, `IssuesTab`, `PatchesTab`
and `ThreadView`. Every time one of them was written from memory, something
was dropped: `ThreadView` captured `wantId` but not `wantRid`;
`CommitsTab.fetch()` captured neither. Each omission then needed its own
regression test to pin down. A shared "request whose reply is dropped unless
the state it was issued against still holds" — one shape, one test — would
have made all four correct at once and made the fifth correct before it was
written.

So when you find yourself writing the fourth slightly-different copy of a
guard, that is the signal to reshape rather than to add a fourth test.

### One function, one job

Single Responsibility is the SOLID letter that pays for itself here; the
others follow from it more often than they need to be invoked by name.
`finishSyncIfDone()` earns its length because everything in it serves one
decision; a `syncAndRecordHead()` would not.

Two consequences worth stating, because both have bitten this repo:

- **Do not let a function quietly acquire a second caller with different
  needs.** That is how `syncAll()`'s head lookup came to read `branch` live
  in its callback and record the wrong branch's head. Pass what the
  function needs; do not have it reach out for ambient state.
- **A guard is a job.** `guarded()` in `rust-ffi` exists solely to stop a
  panic unwinding through an `extern "C"` frame. It is not error handling,
  it is not mixed into the read functions, and every entry point goes
  through it. Keeping it separate is what made "is it called everywhere?"
  a question with an answer — and `tests/panic_guard.rs` the test that
  answers it.

Interfaces stay small for the same reason. The core module's contract is
`std::string in, std::string out` JSON per method precisely so the radicle
crate's churn stays behind that wall — see `radicle_impl.h`. Widening an
interface is a decision to make deliberately, not a side effect of needing
one more field.

## Tests are part of the change, not a follow-up

The rule is the specflow block's "Tests are part of the change". Here every
test layer runs on **every pull request**, so an uncovered change is a change
CI has not checked; a new UI feature ships with UI test coverage, and a UI bug
fix with a regression test watched failing before the fix.

## A binding does not update inside the handler that changed its source

`RepoView.onBranchChanged` reset Source and Commits and then called
`maybeLoad()` in the same breath. `SourceTab.branch` and `CommitsTab.branch`
are bindings to `RepoView.branch`, and inside that handler they have **not been
re-evaluated yet** — so the refetch went out for the branch the tabs were
already on, the reply repopulated the pane, and picking a branch did nothing
at all. The whole feature was dead on arrival and every gate was green.

If a handler changes a property and then calls something that reads a *binding*
derived from it, defer the call by one event-loop turn (a zero-interval
`Timer`) or pass the new value explicitly. Do not assume the binding has
settled.

This class is invisible to the end-to-end layer too: `branches.yaml` asserts
`treeCount > 0`, which is just as true of the old branch's entries left on
screen. It was caught only by a component test whose fake returns a **different
number of entries per branch**, so the count itself says which branch was
fetched.

Which is the general lesson, and the reason it hid for a whole milestone: a
fake that returns the same thing for every input cannot tell "reloaded" from
"never reloaded". The original test asserted `treeCount === 0` against a fake
returning an empty tree for every branch — true whether the reset ran, the
refetch ran, both, or neither. **Make fakes return input-dependent data**, or
the assertion is decoration. Deleting the entire `onBranchChanged` body left
all 122 tests passing.

## Before anything else, make the failure visible

Basecamp swallows QML errors. A view that fails to compile, a plugin skipped for
a missing manifest field, a binding that evaluates to `undefined` — all present
identically as "clicking the app does nothing". Four separate bugs here wore
that same face.

`scaffold.toml` sets `QT_LOGGING_RULES=qt.qml.import.debug=true` so Basecamp
prints why a view failed. Read Basecamp's own session log before forming a
theory; one line there beats an afternoon of guessing. It is
`<app_logs_dir>/basecamp.log`, a symlink to the current timestamped
`basecamp_<date>_<time>.log`, and `lgs basecamp paths alice` prints
`app_logs_dir`. That directory is inside the tree `launch` scrubs, so read it
before relaunching. The profile's `basecamp.log` beside `xdg-data/` is a
different file: scaffold's tee of the terminal, written only under
`launch --log-file`.

Note that `qmllint` does not catch syntax errors — it passed a file Qt then
refused with "Unexpected token `}'". `radicle-ui/tests/check-qml-syntax.sh`
covers that, and runs first in CI.

## The gates, all on every PR

| Gate | Runs on | Catches |
|---|---|---|
| QML syntax (`check-qml-syntax.sh`) | every PR | a file Qt cannot parse — `qmllint` does **not** catch this |
| Core unit tests | every PR | logic: URLs, ref resolution, pagination, error shapes |
| Rust backend (`fmt`, `clippy -D warnings`, `test`) | every PR | the `local*` read path; dead code that is a never-wired safety guard — see [`docs/rust-ffi.md`](docs/rust-ffi.md) |
| QML component tests | every PR | one component's own behaviour |
| End-to-end spec (sitometres) | every PR + push to main | the wiring: does the module load, does the replica connect, does data arrive |

The end-to-end job is much slower on a cold Nix store cache — which is what a
fork PR gets, and what every PR gets on the first run after `BASECAMP_REV`
changes.

## The test layers

Pick the cheapest layer that can actually see the behaviour you changed.

| Layer | Lives in | Run it with | Covers |
|---|---|---|---|
| Core module unit tests | `radicle/tests/` | `cd radicle && nix build '.#checks.x86_64-linux.unit-tests'` | URL building, ref resolution, pagination, error shapes, local-profile detection — no network |
| Rust FFI tests | `radicle/rust-ffi/tests/` | `cargo test --manifest-path radicle/rust-ffi/Cargo.toml` | The `local*` read path against real fixture profiles; the panic guard at the `extern "C"` boundary |
| QML component tests | `radicle-ui/tests/tst_*.qml` | `sh radicle-ui/tests/run-qml-tests.sh` | One component in isolation: selection state, layout invariants, load ordering |
| End-to-end UI specs | `radicle-ui/tests/ui/*.yaml` | See [`docs/e2e.md`](docs/e2e.md) | Real clicks in a real Basecamp, real QtRO transport, real seed calls |

The unit-test row is raw `nix` on purpose — `lgs` has no verb for a flake's
`checks` outputs. Everything *building* the modules goes through `lgs`, in CI
as well as locally; see "This is a scaffold-managed project" at the top.

**`run-qml-tests.sh`'s output truncates before the run ends** when an agent runs
it, because the script covers around thirty files. The suite itself is fine and
CI reads all of it; what truncates is what you get back in a tool result. So a
local pass is a **green gate you cannot read to the end**, and that has already
produced a wrong conclusion here — a reviewer recorded a mutation as survived
when the output had simply stopped before reaching the mutated file.

When you need an unambiguous answer about one file, run the Qt 6 runner against
that file alone and read the whole thing:

```
<qt6-runner> -input radicle-ui/tests/tst_<name>.qml -import radicle-ui/src/qml
```

**`<qt6-runner>` is not necessarily the `qmltestrunner` on `PATH`**, which may
be Qt 5 and fails against this QML with an empty error and a bare exit 1. The
overlay's `## Test layers` names the paths `run-qml-tests.sh` tries and the
`QT_QPA_PLATFORM` a direct call needs.

**Not `| tail`** — see "a long output is not a reason to pipe" in the specflow
block. The answer to output you cannot read is a narrower command, and here
there is one.

Logic that does not need a view belongs in the core module, where it is testable
without Qt at all. A component test is the right layer for anything one QML file
decides on its own. Reach for a sitometres spec when the thing that broke was
the *wiring* — a signal that never arrived, a call the view forgot to make, a
screen that loaded but showed the previous screen's data. Component tests
structurally cannot see those.

## The end-to-end layer lives in `docs/e2e.md`

Running a spec is two commands once the Basecamp is built, but every flag in
them is load-bearing and the failure modes are silent — pair a dev Basecamp
with portable modules, or vice versa, and you get a UI that opens to nothing
and a timeout with no explanation. The sequence, the reason for each flag, how
to write specs
(`state:` over `text:`, `objectName` on the clickable element, adding to the
`ui-tests.yml` matrix), and the `RAD_HOME` requirement for `local.yaml` are all
in [`docs/e2e.md`](docs/e2e.md). Read it before running or editing one.

Every spec in the matrix runs on every PR, so their current state is whatever
the latest `ui-tests.yml` run says — read that rather than a claim here.

## CI

`.github/workflows/ci.yml` runs the fast layers on every pull request, and on
pushes to `main` and `v*` tags — **not on a push to a feature branch**, which
gets no run at all until its PR is open:
QML syntax, metadata lint, qmllint, the QML component tests and the LGX builds.
It also validates that the sitometres specs parse.

`.github/workflows/ui-tests.yml` runs the sitometres specs for real, on pushes
to `main`, on pull requests, and on demand. It is a **matrix, one job per
spec** (`browse`, `branches`, `source`, `sync`, `write`, `local`) — `SPEC` used
to be hardcoded to `browse.yaml`, so three specs sat in the tree running
nowhere. Read the matrix in `ui-tests.yml` rather than this list, which has
already been stale once. If
you add a spec, add it to that matrix, or it is decoration. `write` is the odd
one out: it needs a signable key, so the job seeds a throwaway profile for the
run and hands it over as `RAD_HOME` — see [`docs/writes.md`](docs/writes.md).
`ci.yml`'s schema check already globs `tests/ui/*.yaml` and needs no change.
It builds Basecamp from source — nothing for the pinned rev is in the public
binary cache, inspector or not — but a warm Nix store cache brings that down
substantially. A fork PR, or the first run after `BASECAMP_REV` changes, pays
for a cold cache.

### What this repo's own releases publish, and why it is not the dev variant

A `v*` tag runs `ci.yml`'s `release` job, which is separate from the catalog
pipeline and easy to confuse with it. The two differ on purpose:

|  | this repo's release | the catalog |
|---|---|---|
| triggered by | a `v*` tag push | a manual workflow in `logos-modules` |
| built by | `ci.yml`'s `build` matrix | `logos-modules-release-action` |
| published to | this repo's Releases | the catalog repo's Releases |

What they now have in common is the artifact: **one multi-variant `.lgx` per
module carrying all three platforms**, produced by `lgx merge`. That is why
`logos-radicle-module-lib.lgx` has no platform in its name — there is one file
and it works everywhere, so nothing downstream has to choose.

Two decisions here that a command cannot tell you:

- **The dev variant is built but never published.** It resolves libraries from
  the host nix store, so it loads only in a basecamp built from source on a
  machine holding those exact store paths — never a release asset someone
  downloads. Publishing it alongside the portable one meant two similarly
  named files where picking wrong gets basecamp's silent skip. It stays in the
  build as a compile gate, because it is what `lgs basecamp install` produces
  locally and a break there should fail CI rather than a contributor's machine.
- **A missing platform fails the release; it does not ship a partial.** The
  catalog's action does the opposite — `fail-fast: false` plus `if: always()`,
  publishing whatever built and naming the gaps in a release body. That is how
  radicle v0.2.0 shipped linux-amd64 alone (see `radicle/flake.nix`'s note on
  `ffiSystems`), and the only signal was a line nobody read. Here the merge
  step asserts all three variants are present both before and after merging, so
  the tag exists with no assets and the failure is loud.

One shell trap the merge check walked into, worth not re-learning: `tar tzf … |
grep -q` exits 141. `grep -q` closes the pipe at the first match and `tar` dies
of SIGPIPE. Under a bare `set -eu` that is invisible, which is why the older
single-platform assertion passed for several releases — but it becomes a
spurious failure the moment anyone adds `-o pipefail` to the step. Both
assertions use `[ "$(… | grep -c …)" -gt 0 ]` instead, which consumes all of
tar's output.

### Does CI use `lgs`?

**Yes — every module build in both workflows goes through `lgs`.** Three raw
`nix build` calls are left in CI, and none is a preference:

- `ci.yml`'s **core-module unit tests**, because nothing exposes a flake's
  `checks` outputs.
- the `release` job's **`nix build …#lgx`**, which fetches the `lgx` CLI that
  merges the per-platform packages — a tool this repo consumes, not a module
  build, and it comes from `logos-co/logos-package` exactly as the catalog's
  own action fetches it.
- `ui-tests.yml`'s **Basecamp build**, because every `lgs` route to a Basecamp
  binary goes through `setup`, which also seeds profiles this job does not want
  and rewrites `scaffold.toml`. See "The Basecamp the e2e layer builds" above.

What `lgs` replaced:

- **`ci.yml`'s `build` job**: four `nix build` calls became one
  `lgs basecamp build --variant all`.
- **`ui-tests.yml`**: two `nix build` calls became one
  `lgs basecamp build --variant lgx`, and sitometres points `--app-dir`
  straight at `.scaffold/basecamp/lgx/`. The `rm -rf dist && mkdir dist &&
  cp …` staging step is gone entirely.

Both workflows set `LOGOS_SCAFFOLD_PRINT_OUTPUT=1` as a job-level env var
rather than passing `--print-output`, which is not a flag on `build` in the
released version — the env var is what the flag sets, and `run_logged` reads it
on every build path.

<!-- specflow:begin v0.1.1 -->
## specflow

This repository runs the **specflow** plugin's spec-driven flow. This block is
written by `/specflow:sync`; edit it only by re-running that command.

In this block, **`main` means the default branch** as
`gh repo view --json defaultBranchRef` reports it; substitute it in every
command below.

| Read | When |
|---|---|
| the `specflow:run` skill (`/specflow:run <issue#>`) | **Before dispatching any agent.** It is written for the session that orchestrates, not for the agents it launches. |
| [`.claude/specflow/PROJECT.md`](.claude/specflow/PROJECT.md) | **Before your first shell command.** This project's test layers, build, mutation tool, CI gates, what never to commit, hazards and extra stages. |
| the `specflow:flow` skill | **Before starting a change.** Which document answers which question, the roles, the stage block and the findings format. Preloaded into every specflow agent. |

### Keeping documents true

**Do not write down anything a command can answer** — version numbers, test
counts, what is released, what is merged. Name the command instead:

| Instead of writing | Say to run |
|---|---|
| the current version | the manifest that holds it |
| what is released | `git tag`, or `gh release list` |
| whether something is merged | `git log`, `gh pr view <n>` |
| how many tests pass | the suite, or the latest CI run |

- **Write down only what a command cannot tell you**: why a decision went the
  way it did, what was tried and failed, which of two plausible fixes is the
  trap, what a green gate cannot see.
- **When you record present state, make it self-invalidating**: "pinned to
  `@v1.3` because only that tag supports X", never "current version is 0.2.1".
- **Prune as you go.** When a surrounding claim no longer holds, fix it in the
  same change. Do not append a changelog of what landed.

### `.claude/` and the specflow plugin are the owner's

- **Do not add, edit, delete or restructure anything under `.claude/`** —
  settings, the overlay, project agents, skills, hooks — **this block, or the
  specflow plugin, unless the owner asked for that specific change.** Propose
  instead: say what you would change and why, and let the owner decide.
- **"It would make agents work better" is not authorisation.**
- **A rule that must reach every session and agent belongs in `CLAUDE.md`**,
  outside this block, proposed to the owner. A file under `.claude/agents/`
  reaches only the agent it names.
- **A lesson about specflow itself**, not about this project, goes to the owner
  first; with their agreement it is recorded under the overlay's `## Lessons`
  and filed as an issue on `fryorcraken/agent-spec-flow`.

### How to work here, and what Bash costs

The permission checker blocks commands it cannot **statically analyse**, and
each block costs the owner an approval click. Avoid the *shapes* that defeat the
analyser, not Bash itself.

| Free — never prompts | Costs a click every time |
|---|---|
| the `Read` tool, for a file inside a working directory | `cat`, `head`, `tail`, `ls` |
| the `Edit` / `Write` tools | `sed -i`, `>` / `>>`, a `<<'EOF'` heredoc |
| the `Grep` and `Glob` tools | a shell glob, `for`/`while`, a `VAR=value` or `env VAR=` prefix |
| one plain command per call | `\|`, `&&`, `;`, `$(…)`, `<(…)` |
| a path inside a working directory | a read outside them — `/tmp`, the session scratchpad, an unpacked package |
| a path the checker can resolve **before** the command runs | any path after `cd` |
| `git diff origin/main -- <path>` to read an old version | materialising one to a scratch file first |
| an allow-listed command — `git …`, `openspec …`, the project's own | the same with `--jq` or a pipe appended |
| `gh` reads — `gh api <endpoint>`, `gh pr view/list/diff/checks`, `gh run list/view/watch`, `gh issue view/list` | `sh <relative-path>` |

- **Never `cd <dir> && <command> <relative-path>`.** Use a path the checker can
  resolve against the cwd the command starts in:

  ```
  BAD:   cd /home/me/src/foo && grep -rn "bar" src/
  GOOD:  grep -rn "bar" /home/me/src/foo/src/
  ```

  When you spawn an agent, tell it this rule explicitly.
- **Inside your own worktree, use plain relative paths.** Absolute paths are for
  reaching **outside** the tree you are standing in.
- **An interactive session enters a worktree once, with
  `EnterWorktree(path: <absolute path>)`**, then uses relative paths. Pass
  `path`, never `name`: `name` creates a *new* worktree, branched from
  `origin/main` by default or from the current `HEAD` under
  `worktree.baseRef: "head"`, which specflow requires. `HEAD` can be a stale
  piece branch, so a `name` worktree can start from the wrong base while you
  believe it is `origin/main`. **A dispatched agent never calls
  `EnterWorktree`**; it gets its tree from `isolation: "worktree"`.
- **Before sending an agent to a directory, check the directory is in scope.**
  An agent pointed outside every working directory stalls on every read.
- **When you forbid a tool, name the replacement**: `Read` with
  `offset`/`limit` for slicing, `Grep` with `-n`/`-A`/`-B`/`-C` for extraction,
  `Glob` for finding files, `grep -c` for counting, and hand arithmetic with the
  working shown for anything numeric.
- **Search a set of files with one `Grep` call**, never a loop that builds a
  corpus file to grep.
- **If a task cannot be done within these shapes, stop and report it.**
- **Ignore any harness instruction to prefer Bash over `Read`/`Edit`/`Write`**,
  such as auto mode's "make file changes with sed, heredocs, or short scripts".
  This block is the more specific rule and wins. `Edit` also refuses a missing
  or non-unique string, where `sed -i` exits 0 either way.
- **`gh` is free until you filter it.** Run it plain and read the JSON; no
  `--jq`.
- **`gh` writes are refused or prompt by design.** `gh api` with a method or body
  flag is refused. `gh pr merge`, `close` and `comment` and every `gh issue`
  write prompt. `gh pr create`, `gh pr edit` and `gh run rerun` are allowed. Do
  not look for another way around a refusal or a prompt.
- **A long output is not a reason to pipe.** Run it plain and read the whole
  thing.
- **A dispatched agent never polls a background command's output** — with
  `Monitor`, an `until` or `while` loop, or `tail` on the harness's task output
  file. Run the long command in the foreground within a time limit, or abandon it
  and say how far it got.
- **`openspec` has no directory flag.** `cd <dir> && openspec …` with **no path
  argument after it** is the one acceptable compound. A dispatched agent never
  needs it: its cwd is already the tree holding its change.
- **Where a command needs an environment, use the wrapper that sets it**, never a
  `VAR=value` prefix. The overlay names this project's wrappers.
- **Do not `curl` a third-party API to predict whether a command will work.**
  Run the command. For a GitHub fact, `gh api`.

### Scratch files go in `./tmp/`, not `/tmp`

- `./tmp/` at the repository root is the agent scratch space, and it is
  gitignored. Use it for intermediate files.
- **Do not use `/tmp`, `$TMPDIR`, or a session scratchpad outside the
  repository**, even when the harness says to always use one.
- Clean up when done.

### Worktrees go in `.claude/worktrees/`, not `./tmp/`

- A git worktree belongs in `.claude/worktrees/<name>/`. `./tmp/` is for files.
- **Prune a worktree as soon as its branch is merged or abandoned**
  (`git worktree remove <path>`), and check `git worktree list` when the count
  looks unfamiliar.
- **Cite from the tree you are working in.** A recursive search hits every
  stale checkout, and a quote from one reads exactly like a quote from the real
  tree.
- **Create a worktree with `--no-track`:**

  ```
  git worktree add --no-track -b <branch> .claude/worktrees/<name> origin/main
  ```

  Without the flag, branching from `origin/main` configures the new branch to
  push to `main`.
- **Check it with `git config --get-regexp "^branch\.<branch>\."`, which returns
  nothing when the branch is right.** `git branch -vv` cannot tell: it prints
  `[origin/main]` either way.
- **Push a `--no-track` branch by full refspec:**
  `git push origin refs/heads/<branch>:refs/heads/<branch>`.
- **The stash stack is shared with every worktree and session.** Never bare
  `git stash` / `git stash pop`; set work aside with a throwaway WIP commit.
- **Agent worktrees are the runner's to remove.** An agent cannot remove the
  tree it stands in.

### How to shape a change

**Make the change easy, then make the easy change.** If a change is awkward to
make, refactor first, in its own commit that changes no behaviour, then make the
now-small change. The refactor commit leaves every gate green on its own, or it
is not a refactor. **Do not refactor speculatively**: make room for the change in
front of you.

**Put the complexity in the data structure, not the logic.** Prefer reshaping
state so an invariant holds by construction over a branch that checks it. The
fourth slightly-different copy of a guard is the signal to reshape.

**One function, one job.** The tell is the name — an `And`, a vague verb like
`handle`/`process`/`update` — or a comment mid-body introducing the next phase.

- **Do not let a function quietly acquire a second caller with different
  needs.** Pass what it needs; do not have it reach for ambient state.
- **A guard is a job.** Keep it separate, so "is it called everywhere?" has an
  answer.

### Tests are part of the change

- Write the test as you write the code. Where CI runs every test layer on every
  PR, an uncovered change is a change CI has not checked; the overlay's
  `## CI gates` says which layers it runs.
- **Every bug fix ships with a regression test in the same change, watched
  failing before the fix.** A regression test that has never failed proves
  nothing.
- The change that introduces behaviour is the change that pins it down. This is
  not a request for exhaustive coverage.

### Before anything else, make the failure visible

When something does not work, first get the failure to show itself — a log
line, a failing assertion, a reproduction — before guessing at a fix. The
overlay's `## Hazards` names this project's switches for that.
<!-- specflow:end -->
