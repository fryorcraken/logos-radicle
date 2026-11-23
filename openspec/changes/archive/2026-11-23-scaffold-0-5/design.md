# Design

## Context

See `proposal.md` for why the stack moves and for the set it moves to. This
file records how, and why each pin and setting is the value it is.

Three facts about the stack shape every decision below, and none of them is
visible from this repo's own files:

- **Scaffold supports exactly one Basecamp release at a time.** v0.5.0's is
  0.3.0 (`DEFAULT_BASECAMP_RELEASE` in its `src/constants.rs`). Its launcher
  resolution, the environment `launch` exports, the bundled-module list and
  the retired-pin migration all describe that release and no other — scaffold's
  own ADR "Scaffold Supports One Basecamp Release".
- **The Basecamp pin, the `lgpm` pin and the module-builder generation are one
  set.** `lgpm` writes what Basecamp's embedded package-manager library then
  scans, so scaffold's `lgpm` pin is the rev the Basecamp release locks (ADR
  "Basecamp Pin Bumps Move as a Set"). The module-builder generation decides the
  SDK and protocol a module links, and Basecamp refuses calls from a module
  built against a different one.
- **This repo's code reaches the stack only through four seams**: the flake
  input `logos-module-builder` (both modules), `scaffold.toml`'s
  `[repos.basecamp]`/`[repos.lgpm]` pins (setup, install, launch, and the e2e
  layer's Basecamp rev), `LGS_VERSION` in the two workflows, and the
  sitometres version that drives that Basecamp (`SITOMETRES`, Decision 6).
  Nothing in `radicle/src/`, `radicle/rust-ffi/` or `radicle-ui/src/` names a
  Basecamp or SDK version, and none of it changed.

## Goals / Non-Goals

**Goals:**

- Every gate CLAUDE.md lists is green on the new set, with no module source,
  metadata, QML test or sitometres spec changed unless the new stack forces it.
- Every document that states a fact about the old stack either says the new
  fact or stops stating it.
- Concurrent `lgs basecamp launch alice` / `bob` work, which the shared
  `runtime_dir` workaround silently prevented (Decision 5).

**Non-Goals:**

- See proposal.md "Not in this change". In particular nothing here moves the
  settings file or the embedded home into Basecamp 0.3.0's `module_data`.

## Decisions

### 1. Basecamp 0.3.0, not 0.3.2

Basecamp 0.3.1 and 0.3.2 are tagged, and module-builder runs to 0.3.3. This
change still stops at Basecamp 0.3.0, because that is the release scaffold
v0.5.0 supports, and the other pins only mean something relative to it.

Moving to 0.3.2 would mean deriving the `lgpm` pin by hand from 0.3.2's
`flake.lock`, choosing the module-builder generation whose SDK set matches
it, and then running `setup`, `install` and `launch` with a scaffold whose
launcher, exported environment and bundled-module list describe 0.3.0. The
ADR is explicit that a non-default pin is built but gets no compatibility
code: `basecamp doctor` names the release scaffold actually supports, and
every difference is the project's problem. That is the position this change
exists to leave. While this repo was on 0.2.3 under scaffold 0.3.1 it was
inside scaffold's support; v0.5.0 lists that exact pair among its retired
sets, which is what started this change.

The rejected alternative was to take the newest Basecamp and pin `lgpm` and
module-builder to match it by hand. It would have bought two point releases
of Basecamp fixes. It would also have put this repo back on a combination
scaffold's tests never run, the same way it ran on an unmerged scaffold PR
branch for three force-pushes. The cost of staying on 0.3.0 is a wait, and a
newer Basecamp is not a fix for anything this repo needs today.

So the next stack bump starts from a scaffold release that names a newer
Basecamp in `DEFAULT_BASECAMP_RELEASE`, not from a Basecamp release.
`docs/PLAN.md`'s "The Logos stack" keeps that rule as the constraint still
ahead; this entry is its reasoning.

### 2. The pins are scaffold's literal defaults, written by hand

`[repos.basecamp].pin` is `bbe5da0e…` (logos-basecamp tag `0.3.0`, which
`git rev-parse 0.3.0^{commit}` confirms in a clone) and `[repos.lgpm].pin` is
`d3af2972…`. They are copied from `DEFAULT_BASECAMP_PIN` and
`DEFAULT_LGPM_PIN` in scaffold v0.5.0's `src/constants.rs`, and not from a
Basecamp `flake.lock`.

Scaffold's `setup` would have made the same edit itself: the old pair is
exactly a `RETIRED_BASECAMP_PIN_SETS` entry, labelled "0.2.3". It was not
used to make the edit because `setup` rewrites `scaffold.toml` through
`save_project_config`, which v0.5.0 still documents as destructive (see
Decision 7). Writing the pins by hand keeps the edit a two-line diff a reviewer
can check against the constants. `setup` was then run on the hand-edited file
as a check: it printed no "moved off a retired pin" line, which it would have
printed for any value that was still a retired default.

Copying the literal default, rather than a Basecamp tag name or a newer
commit on the 0.3.0 line, is what keeps the next migration automatic. Scaffold
matches retired pins by exact rev. A project on scaffold's default today is
moved by a future `setup` when that default retires. A project on any other
value is left alone, and nothing says it was left.

### 3. logos-module-builder 0.3.0, not 0.3.3

Both `flake.nix` files pin tag `0.3.0`. Scaffold v0.5.0's
`BASECAMP_DEPENDENCIES` comment states the constraint: a module must be
"built against the same `logos-protocol` / `logos-cpp-sdk` basecamp 0.3.0
links, or basecamp refuses its calls at runtime", and module-builder `0.3.0`'s
SDK set is exactly Basecamp 0.3.0's, while `0.3.1` is "already past basecamp".

What breaks without this pin: the modules still build, which is why it is a
guard worth writing down. A module-builder past the Basecamp would compile
and package cleanly, and then fail at runtime when Basecamp refused the
module's calls. Only the sitometres specs would see that. In CI that is
`ui-tests.yml`'s whole matrix, every spec failing at its first
backend-dependent step. No other gate links a module against a real Basecamp.

Relocking moved only the `logos-module-builder` subtree of
`radicle/flake.lock` (`nix flake update logos-module-builder --flake
./radicle`). `nixpkgs-fetch` was deliberately not touched. The builder's own
nixpkgs did not move either: it is still `e9f00bd` (2025-09-28) before and
after. So `radicle/flake.nix`'s notes on the builder's rustc 1.89 and its
User-Agent-less fetcher still hold, and the Rust staticlib's toolchain story is
unchanged.

`radicle-ui/flake.lock` is not relocked, because it has never been tracked
(`git log --all -- radicle-ui/flake.lock` is empty). Its only non-builder
input is `radicle.url = "path:../radicle"`, which `lgs` overrides with the
working tree on every build anyway. A committed lock would pin a `narHash` of a
directory that changes with every commit. The proposal's Impact list names
the file, and the reason it is untracked holds.

### 4. Per-profile HOME: what Local mode under `launch` sees

Scaffold v0.5.0's `launch_env` sets `HOME` to
`.scaffold/basecamp/profiles/<name>/home` (its `PROFILE_HOME_DIR`, added for
the storage module, which ignores XDG). `LocalStore` resolves `RAD_HOME`,
else `$HOME/.radicle`. So an `lgs basecamp launch alice` in Local mode now
finds no profile, where under scaffold 0.3.1 it read the developer's own
`~/.radicle`.

Nothing in this repo was changed to undo that. The module already does the
right thing for the environment it is given, and that environment is the same
one sitometres has always given it (a throwaway `HOME`, `RAD_HOME` passed in).
What changed is the developer workflow, and the route back is environmental:
`launch` builds its child with `std::process::Command` and no `env_clear`, so
anything set in the shell that runs `lgs` reaches Basecamp unless scaffold
overwrites it. Observed in a launched instance's `/proc/<pid>/environ`:
`HOME` was the profile's `home/`, while `DISPLAY=:0`, which scaffold never
writes, had come through from the shell. `RAD_HOME` is not one of the names
scaffold writes either, so exporting it in the developer's shell restores
Local mode under `launch`. That last step follows from the inheritance
rather than from a launch with `RAD_HOME` set, which was not run: setting it
needs either a `VAR=value` prefix or a change to the developer's real shell
profile.

Rejected: `RAD_HOME` under `[basecamp.profiles.<name>.env]` in
`scaffold.toml`. The file is tracked, and the value is one developer's home
directory. That is the same mistake as the hardcoded `runtime_dir` Decision 5
removes, and the reason Decision 5 keeps `WAYLAND_DISPLAY` out of the file
too. Also rejected: a gitignored per-profile `env_file`, because `launch`
fails outright when a configured `env_file` is missing, so every fresh clone
and worktree would have to create one before its first launch.

What this costs is documented in CLAUDE.md and `docs/e2e.md`: the step-9
contention caveat between an interactive instance and `local.yaml` now
applies only when the interactive instance was pointed at the same profile.
The export is also the only thing that now opts an `lgs`-launched instance
into real writes. It reaches every profile, and with the key unlocked
`canWriteLocal` is true, so a comment posted from `alice` or `bob` is signed
with that home's identity. Under scaffold 0.3.1 that was the default; per-profile
`HOME` made the safe state the default. So CLAUDE.md says so beside the
recipe, and points at a throwaway profile seeded the way `write.yaml`'s is.

No requirement in `module-settings` or `node-paths` moves.
Both derive from `XDG_DATA_HOME`, which `launch` still sets per profile, and
`local.yaml` passed 47/47 on the new stack, including the steps that read
`nodeHome` for Local and for Embedded.

### 5. `runtime_dir` goes; the developer's shell names the Wayland socket

Added to scope at the user's request. Both profiles used to carry
`runtime_dir = "/run/user/1000"`, and the pin did two jobs:

- **It kept the module socket path short.** QtRO's `logos_token_<module>`
  sockets live in the runtime dir. Scaffold's old in-profile default
  overflowed the 108-byte `sun_path` cap, and every module segfaulted.
- **It let Qt find the Wayland compositor.** libwayland resolves a relative
  `WAYLAND_DISPLAY` (`wayland-0`) against `XDG_RUNTIME_DIR`.

Scaffold v0.5.0 makes the first job redundant. `resolve_profile_runtime_dir`
defaults to `/tmp/lgs-<project-hash>-<profile>`, which `ensure_private_runtime_dir`
creates 0700. On this machine that is `/tmp/lgs-7d1fbd26-alice`, about
forty bytes before the socket name. The pin also overrode scaffold's
per-profile separation: it put alice and bob in ONE directory, which is the
arrangement behind the cross-profile `logos_token_<module>` collision
(scaffold#89) that the per-profile default exists to prevent. Basecamp 0.3.0
softens that one. Its sockets now carry a per-instance suffix
(`logos_capability_module_89c0a51ce8f4` for alice,
`…_2773aabbbc2b` for bob), so the same-name collision may no longer bite.
That leaves the pin with no job of its own and one hardcoded session path, so
it is gone.

The cost of the default is that it is only as private as scaffold's check, and
that check has a gap. `ensure_private_runtime_dir` (v0.5.0
`src/commands/basecamp.rs`) creates a missing dir 0700, refuses a symlink or a
non-directory, and chmods a loose one, relying on that chmod failing with
`EPERM` as its ownership test. A directory that another local user created
first, already 0700, takes none of those branches, and nothing compares its
owner with ours. The name is predictable (a hash of the project path), so on a
shared machine that user can claim it. Basecamp is then handed a runtime dir it
cannot enter, and every module dies at "Failed to register module for remote
access", the same face as the `sun_path` bug, with nothing naming ownership.
This is denial of launch, not exposure: the sticky bit stops the dir being
swapped under one we own, and a foreign 0700 dir keeps the other user out of
our sockets as much as us out of theirs. The `/run/user/<uid>` pin was
user-owned by construction. Not fixed here, because the fix (compare
`meta.uid()` with `geteuid()`) is scaffold's; it is an upstream ask, not filed
from this change. The symptom to recognise: `ls -ld /tmp/lgs-*` shows another
owner.

The second job, reaching the compositor, moves to the developer's shell. An
absolute `WAYLAND_DISPLAY` is used as-is, whatever `XDG_RUNTIME_DIR` says, so
CLAUDE.md asks for one line in the shell profile:

    case "$WAYLAND_DISPLAY" in ''|/*) ;; *) export WAYLAND_DISPLAY="$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY" ;; esac

`launch` passes it through unchanged. `launch_env` writes `HOME`, `TMPDIR` and
the `XDG_*` names and nothing about Wayland, `apply_launch_env_overrides` only
adds `scaffold.toml`'s names, and the child is built with `Command::env` calls
and no `env_clear` (v0.5.0 `src/commands/basecamp.rs`, `launch_env` and the
`cmd.env` loop in the launch path). Basecamp's launcher then only reads the
variable. That inheritance was observed for `DISPLAY` (Decision 4).

There is no Xwayland fallback to hide a failure. Basecamp's own launcher
(`nix/app.nix:702-704` at 0.3.0) exports `QT_QPA_PLATFORM=wayland` whenever
`WAYLAND_DISPLAY` is set and no platform is chosen. A desktop session always
sets `WAYLAND_DISPLAY`, so Qt is forced onto its Wayland backend, and a
socket it cannot reach is fatal. That is the "exits immediately" the old
comment described. It is also why the control below discriminates. **That
launcher line is the thing to re-read when the Basecamp pin moves.** If a later
launcher stops forcing the platform, Qt can fall back to xcb through Xwayland,
the control row stops failing, and the reasoning here goes false while every
gate stays green.

The absolute value was proved with real `lgs basecamp launch` runs rather than
assumed, set through `scaffold.toml`'s `[basecamp.env]` while that was where it
lived. Environments were read back from `/proc/<pid>/environ`.

| run | `WAYLAND_DISPLAY` | result |
|---|---|---|
| alice | absolute, in `[basecamp.env]` | runs on Wayland; `XDG_RUNTIME_DIR=/tmp/lgs-7d1fbd26-alice`; no `wl_display` error |
| alice, control | not set, so the session's relative `wayland-0` | "Failed to create wl_display (No such file or directory)", then "no Qt platform plugin could be initialized", exit 1 |
| alice + bob together | absolute, in `[basecamp.env]` | both alive past 40 s; `XDG_RUNTIME_DIR` `/tmp/lgs-7d1fbd26-alice` and `…-bob`, each holding only its own sockets; `QT_QPA_PLATFORM=wayland` in both (from the launcher); no `wl_display` error in either log |

The first two rows set `QT_QPA_PLATFORM=wayland` explicitly through a
temporary per-profile env, before the launcher's own rule was found. The
third row had nothing added beyond the absolute `WAYLAND_DISPLAY`.

Delivering that same value from the shell instead of from `scaffold.toml` was
not itself launched. It follows from the pass-through above: both routes end
as the same entry in the child's environment. Checking it takes one launch
after adding the shell line, reading `WAYLAND_DISPLAY` back from the
instance's `/proc/<pid>/environ`.

None of this is visible to any automated gate. The e2e layer runs
`offscreen` under sitometres and never calls `lgs basecamp launch`, and CI
has no compositor. So the table is the evidence, and it is only as current
as this machine.

Why the shell, and not the tracked file. The value depends on the machine and
the session, not the project: a uid, a socket name the compositor chose, and
no value at all on an X11 session. The shell line computes it from the
session's own `XDG_RUNTIME_DIR` and `WAYLAND_DISPLAY`, so it is right for any
uid and does nothing where there is no Wayland. That is the same rule that
keeps `RAD_HOME` out of `scaffold.toml` (Decision 4), and the two now sit
together in CLAUDE.md's exception to "environment goes in `scaffold.toml`".

Rejected:

- **`WAYLAND_DISPLAY = "/run/user/1000/wayland-0"` in `[basecamp.env]`.** It
  works out of the box on exactly one kind of machine, a Wayland session for
  uid 1000, and it was the measured configuration. Everywhere else it is
  wrong in a way the shell line is not. A developer with another uid gets
  "Failed to create wl_display". On an X11 session with no compositor it
  breaks a launch that worked under the pin: setting the variable is what
  makes the launcher force Wayland, where the pin left it unset and Qt chose
  xcb. And it contradicts the rule this change writes into CLAUDE.md, that a
  tracked file carries nothing per developer. What it had over the shell line
  is that a uid-1000 Wayland developer needs no setup; the shell line costs
  every Wayland developer one line once, and a missing line fails loudly with
  the message the docs name.
- **A computed value in `scaffold.toml`**, such as
  `"$XDG_RUNTIME_DIR/wayland-0"`. `apply_launch_env_overrides` inserts each
  `[basecamp.env]` value as a literal `OsString`, with no expansion.
  `[basecamp.env_append]` joins with `:` onto the inherited value, which is a
  path-list operation, not a path.
- **A gitignored per-profile `env_file`.** `launch` fails outright when a
  configured `env_file` is missing (Decision 4), so every fresh clone and
  worktree would need one before its first launch.
- **Keeping the pin with a corrected comment.** That keeps the #89 collision
  for anyone who launches both profiles.
- **Pointing `runtime_dir` at per-profile subdirectories of `/run/user/1000`.**
  That keeps the hardcoded uid and the relative-socket lookup working. But
  `ensure_private_runtime_dir` chmods whatever it is given, and creating
  directories inside the session's runtime dir from a project file is a wider
  reach than naming one socket.

The shell line exports an absolute `WAYLAND_DISPLAY` to everything that
shell starts, not only to `lgs`. libwayland has accepted an absolute value
since 1.15, so any client linked against it reads it the same way. The
launcher is Linux-only, so macOS is unaffected either way.

The embedded node's own control socket moves, and no requirement does.
`resolveSocket` prefers `$XDG_RUNTIME_DIR` (`node-paths`). So, by that rule
rather than by observation (no embedded node was started under `launch`
here), under `lgs basecamp launch` the socket moves from
`/run/user/1000/radicle-alice.sock` to
`/tmp/lgs-<hash>-alice/radicle-alice.sock`. That is still per profile and far
under the cap. `radicle/src/settings_store.h`'s note, which named the pin as
the reason for the cap, now names scaffold's default instead.

### 6. sitometres 0.2.0, because 0.1.2 cannot drive Basecamp 0.3.0

The proposal listed this as something to measure, not assume. Measured:
`browse.yaml` with sitometres 0.1.2 against the 0.3.0 `#app` and this
change's modules fails at step 1, "radicle_ui did not open within 120s". The
cause is in Basecamp's own log, kept by running with `--user-dir`:

    Warning: module 'radicle' in …/modules/radicle was installed for variant
    'linux-amd64' which is not supported on this platform and will not be
    loadable: supported variants [linux-x86_64-dev, linux-amd64-dev].
    …
    Failed to load UI module "radicle_ui" : "Failed to load core dependencies for radicle_ui"

The package itself carries only `variants/linux-amd64-dev/` (`tar tzf
01-radicle.lgx`), so the wrong variant label is 0.1.2's staging, not the
build. The same run's log also has Basecamp looking for the icon under
`plugins/radicle_ui/assets/`, the `assets/` root entry module-builder 0.3's
packages add, which 0.1.2 did not stage. Against the same Basecamp and the
same `.lgx` files, sitometres 0.2.0 passes `browse` 18/18.

Two things were checked before changing the tool that gates every spec:

- **0.2.0's JSON report keeps the shape `ui-tests.yml`'s gate reads.** It has
  a top-level `verdict`, and `steps[]` entries with `name` and `verdict`, one
  per spec step (18 for 18 in `browse`). So "The run proved what the spec
  asks" needs no change.
- **The two specs that failed were not 0.2.0's doing.** `source` and `sync`
  failed at the same step under 0.1.2 on the old stack too. Both turned out to
  be spec defects (Risks).

Rejected: staying on 0.1.2 and working around its staging, for example by
pre-installing the modules into a `--user-dir`. That would test a layout no
user has, through a code path sitometres itself has replaced.

What breaks without this decision: every spec in the `ui-tests.yml` matrix,
at `open`, on every PR.

### 7. `setup` still strips comments, so the warning stays, restated

The proposal asked for this to be settled by running `setup`, not by reading
release notes. Run on this branch's `scaffold.toml` with v0.5.0, `setup`
completed, changed no value, and deleted every comment line
(`git diff --stat scaffold.toml` showed deletions only). That matches
the source: v0.5.0's `cmd_basecamp_setup` ends in an unconditional
`save_project_config`, which its own doc comment calls "a destructive
serialization", and the only comment-preserving path is `init`'s.

So the `scaffold.toml` warning and CLAUDE.md's section both stay true. What
changed is their evidence: both now cite the v0.5.0 run rather than #266's
history. The "Upstream asks" entry records what became of
`fix/preserve-scaffold-toml-comments`: it was opened as
logos-co/scaffold#267 and closed unmerged (`gh pr view 267 --repo
logos-co/scaffold`). CLAUDE.md's "reverts to fixed when it merges" therefore
had nothing left to wait for, and now says to check `save_project_config` in
the pinned release instead.

`basecamp doctor` after that `setup` reports `basecamp pin set | basecamp
bbe5da0e038e / lgpm d3af2972f51d — scaffold defaults`, which is Decision 2's
check from the other side.

### 8. The e2e Basecamp is evaluated and built in two processes

On this change's first two `ui-tests.yml` runs (44899483064, 44901668963)
every job died inside "Build Basecamp", 3 to 18 minutes in. The annotations said
only "The operation was canceled", nix printed no error, and one runner hung
in its post-checkout step afterwards. It was not the job timeout and not
`cancel-in-progress`. A memory and disk sampler added to the step (run
44902411911) showed the cause on a 4-CPU, 16 GB runner with 3 GB of swap:

| phase | `MemAvailable` | swap used | disk free |
|---|---|---|---|
| step start | 14.9 GB | 0 | 87 GB |
| evaluation done, nothing built yet | 4.4 GB | 0 | 86 GB |
| ~1 min into building | 0.6 to 0.9 GB | 1.3 to 2.2 GB | 82 GB |
| end | runner logs "received a shutdown signal", exit 143 | | |

Two things are new with Basecamp 0.3.0, and together they exceed the machine:

- **Evaluating its flake costs about 10 GB**, and a single `nix build
  <flake>#app` keeps that heap for as long as it runs, compile included.
  Scaffold's own dogfooding notes measure the evaluation peak at 4.8 GB or
  more, so this is the flake, not this repo.
- **Most of the Logos libraries are compiled here, not fetched.** The logos
  public cache has them, but its signatures do not verify against the key every
  logos flake publishes (`public:l4Hr…`). Nix logged "ignoring substitute … not
  signed by any of the keys in 'trusted-public-keys'" 117 times, against 4 on
  `main`'s last 0.2.3 run. `nix store verify --store
  https://cache.nix.logos.co/public --trusted-public-keys public:l4Hr…` calls
  `logos-cpp-sdk` "untrusted", while the same probe against cache.nixos.org's
  key verifies a nixpkgs path, so the probe can tell the two apart.

The fix is `nix path-info --derivation <flake>#app`, which evaluates and exits,
followed by `nix build '<drv>^*'`, which evaluates nothing. The compile gets
back the 10 GB. The `.drv` is the same one the single command built
(`7jv1c04m…-logos-basecamp-0.3.0.drv`), and its one output is
`yf8mbqrx…-logos-basecamp-0.3.0`, which is the path scaffold's notes give for
basecamp's own CI build. Checked locally. On the next run (44963625950) all six
jobs passed. The build step's lowest sample once compiling was 8.2 GB available
with no swap used. The 4.3 GB low now falls during evaluation, with nothing
compiling beside it. **Removing it** puts every
`ui-tests.yml` job back to a runner shut down mid-build. No other gate builds
this Basecamp, so nothing else would go red.

The sampler stays in the step. A runner that dies of memory leaves nothing
else in the log, and the 20-second lines cost nothing on a green run.

Rejected:

- **`require-sigs = false`, or adding `cache.nix.logos.co/ci` the way
  scaffold's notes do for a small host.** That would bring back the fetched
  libraries and most of the build time. It is also a security boundary: it
  turns off signature checking for every substituter, cache.nixos.org
  included, in a job that then runs the result against a live seed. A key
  mismatch is upstream's to fix, and the gain is speed, not correctness.
- **Lowering `max-jobs` or `cores`.** That shrinks the compile's share but
  leaves the 10 GB evaluator heap in place for the whole build, and it slows
  every cold run to buy back memory the split gets for free.
- **More swap.** A swapfile on the 87 GB disk would most likely avoid the
  shutdown by paging the idle evaluator heap out. That is the same 10 GB, kept
  and paged, where the split releases it. Not tried.
- **Freeing disk.** The usual nix-on-Actions fix, but disk never dropped
  below 82 GB free, so it does nothing here.

### 9. The pins' copies are checked for agreement, not merged into one

Three of the pins are written in more than one file: `LGS_VERSION` in both
workflows, the `logos-module-builder` tag in both `flake.nix` files, and the
sitometres version in `ui-tests.yml`, `radicle-ui/tests/e2e-env.sh` and the
command in `docs/e2e.md`. A copy left behind on a bump still builds. A
`radicle-ui` on another builder tag links a different SDK and is refused at
runtime; a local run on another sitometres drives Basecamp with a different
tool from CI. Only the e2e layer or a user's Basecamp would notice.

`.github/scripts/check-stack-pins.py`, run by `ci.yml`'s lint job, finds every
copy by the literal shape of the pin, not by named line, and fails when two
copies differ or when a file that must hold one holds none. Searching by shape
means a new copy is checked without editing the script; the required-file list
means rewording a copy out of the pattern fails instead of silently shrinking
what is checked. It is a script file rather than an inline step, unlike the
metadata checks beside it, so that `python3 .github/scripts/check-stack-pins.py`
runs locally exactly what CI runs.

Rejected:

- **One source per pin.** Each copy is read by a different tool: a flake
  cannot read a workflow, a workflow's `env:` cannot source a shell file, and
  `e2e-env.sh` exits when no Basecamp is built, so it cannot be sourced just
  for a variable. Any single source would need a generation step, which is
  more machinery than the copies.
- **Dropping the literal from `docs/e2e.md`.** The doc's command is the one a
  newcomer pastes, and a placeholder would make it not runnable. With the
  agreement checked, the copy can no longer drift unseen.

What it cannot check is that the set is the RIGHT one: that `LGS_VERSION`'s
scaffold names this Basecamp pin and this builder generation. That is
scaffold's `src/constants.rs`, which CI does not have.

**Removing it** turns no gate red, because a drifted copy builds, which is the
whole reason it exists. With the script in place, setting
`e2e-env.sh` to `0.2.1` and `radicle-ui/flake.nix` to `0.3.3` fails it naming
both copies, and changing `ci.yml`'s `LGS_VERSION` to single quotes fails it
with "no copy found in .github/workflows/ci.yml". Both were run and reverted.

## Risks / Trade-offs

- **[The first `ui-tests.yml` run after this change is cold]** The store
  cache keys on `BASECAMP_REV`, and 0.3.0's `#app` is not in the logos public
  cache (`nix path-info --store https://cache.nix.logos.co/public` on the
  built output reports "path is not valid"). → Expected and one-off; the
  header of `ui-tests.yml` records the re-check.
- **[A cold run compiles the Logos libraries, because the logos cache's
  signatures do not verify]** Decision 8. It costs time on every cold run and
  in every one of the six jobs at once, since they start together and none can
  restore a store another has not yet saved. → Accepted for now. Each cold run
  is once per Basecamp pin and flake change. The mismatch is worth reporting
  to the logos cache's owners (not filed from here), and when it is fixed the
  "ignoring substitute" lines disappear from the build log.
- **[`source` and `sync` depended on which repository the live seed lists
  first]** Both failed the same step on the new stack and on the old one, run
  side by side on this machine against the same seed, so the toolchain was not
  the cause. `main`'s last `ui-tests.yml` run (44837719837) had already failed
  `sync`. → Fixed in the specs, in `bc24ded`:
  - `source` step 13: a 33-entry repository put the `README.md` row below the
    900 px offscreen window. The click landed on nothing and still reported
    "clicked". The spec now scrolls the row into view by index first.
  - `sync` step 17: the staleness poll waited behind a cancelled sync's
    requests (next entry). The spec now polls before it starts and cancels the
    sync.
- **[Cancelling a sync leaves the core module busy for minutes]** A product
  defect found by that `sync` failure and not fixed here; Basecamp did not
  cause it, since the same spec failed the same way on 0.2.3. This change's
  only part in it is `sync.yaml`'s step order: a poll issued after a cancel
  queues behind the cancelled sync's requests, so the spec polls first. →
  The defect, its measurement and what a fix must settle are in
  `docs/PLAN.md`, "Known defects in shipped behaviour".
- **[A package from this change does not install into a Basecamp older than
  0.3.0]** module-builder 0.3.x adds an `assets/` root entry, which an older
  `lgpm` rejects ("Forbidden root entry: assets"). → Accepted: catalog users
  move with Basecamp releases, and the README-level claim of what Basecamp a
  release needs is the release notes' job.
- **[A Wayland developer without the shell line cannot launch]** `lgs basecamp
  launch` exits with "Failed to create wl_display", including for whoever
  launched fine on the tracked value before. → The failure is loud and named in
  the `scaffold.toml` comment, CLAUDE.md and README.md, each pointing at the
  one line (Decision 5).
- **[The `ci.yml` release job's `lgx merge` runs only on a `v*` tag]** It uses
  `logos-package` master, and the new `assets/` root entry is the one packaging
  change it meets. → Run by hand over this PR's three CI packages (run
  45011529504) with `lgx` built from `logos-package` `4bdd904`: both modules
  merged, all three variants present, and the UI package kept
  `assets/icon.png`. The release job now asserts that entry after its merge,
  because master can move before the next tag.
