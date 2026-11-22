# scaffold-0-5 — security review

Dimension: **security** only (one `code-reviewer` instance; correctness,
readability and architecture are other instances').

- [x] **`dev-writer`** — `scaffold.toml` (removed `runtime_dir`), `design.md`
      Decision 5 — the move to `/tmp/lgs-<hash>-<profile>` trades a user-owned
      directory for a predictable name in world-writable `/tmp`, and the design
      records the 0700 creation but not the ownership gap.
      **Scenario (read from scaffold v0.5.0 `src/commands/basecamp.rs`,
      `ensure_private_runtime_dir`; not run):** on a shared machine another local
      user computes `/tmp/lgs-<first 8 hex of sha256(canonical project path)>-alice`
      and creates it first, mode 0700, owned by themselves. `lgs basecamp launch
      alice` takes the `Ok(meta)` arm: not a symlink, is a directory, and
      `mode & 0o077 == 0` so the `set_permissions` call (the only thing that
      would fail on a foreign owner) is never made. No uid comparison exists.
      Basecamp is then handed `XDG_RUNTIME_DIR`/`TMPDIR` pointing at a directory
      it cannot enter, and every module dies at "Failed to register module for
      remote access" — the same silent-segfault face as the `sun_path` bug, with
      nothing naming ownership. A pre-created 0755 dir fails loudly (chmod
      error), and a symlink is refused, so this is availability only: sticky
      `/tmp` stops the attacker swapping the dir out from under a good one, and
      a 0700 foreign dir stops them reading the QtRO sockets. Under the old
      `/run/user/<uid>` pin that dir was user-owned by construction.
      **Severity:** low (developer workstation, denial of launch, no data
      exposure). **Needed:** a sentence under Decision 5's "cost" list saying
      the default is only as private as scaffold's check, with the
      ownership-gap symptom, and (optional) an upstream ask to compare
      `meta.uid()` with `geteuid()`. No code change is asked of this repo, since
      the design already rejected putting `runtime_dir` back.
      **Measured:** not mutated; there is no test layer here that launches.
      **Fixed** in the commit that ticks this box, as asked (prose only): Decision 5 now has a
      paragraph on the default's cost. Re-read at v0.5.0 to confirm: the function's own doc comment
      claims "Someone else's: `set_permissions` fails with `EPERM`", but that call is reached only
      when `mode & 0o077 != 0`, so a foreign 0700 dir passes, exactly as described here. The
      paragraph gives the symptom (`ls -ld /tmp/lgs-*` shows another owner), says it is denial of
      launch and not exposure, and names the `meta.uid()` vs `geteuid()` comparison as an upstream
      ask, not filed from this change.

- [x] **`dev-writer`** — `CLAUDE.md` "Local mode under `lgs basecamp launch`"
      (and `docs/e2e.md` step-9 paragraph) — the documented route back to
      Local mode is "put `RAD_HOME` in your shell profile", which is
      persistent and applies to **every** `lgs`-launched instance, alice and bob
      alike, and Local mode is where the compose boxes write.
      **Scenario:** a developer follows it with their real `~/.radicle`.
      Thereafter `lgs basecamp launch alice` and `launch bob` both share that
      one home, key and node, and a comment or new issue posted from either
      (gated on `getCapabilities().canWriteLocal`, which will be true with the
      key unlocked) is signed with their real identity and written to their
      real storage. Before this change that was the default behaviour (HOME was
      inherited); after it, the per-profile `HOME` makes the safe state the
      default and this paragraph is the only thing that opts out of it, so it
      is the place the warning belongs. As written it mentions contention with
      `local.yaml` and nothing about writes or the shared identity.
      **Severity:** low, doc only. **Needed:** one sentence saying the export
      also makes writes real, and that a throwaway profile (the way
      `write.yaml` seeds one) is the safe value for `RAD_HOME` when trying a
      write.
      **Fixed** in the commit that ticks this box: CLAUDE.md's Local-mode section has a bold
      paragraph, "The export also makes writes real, in every instance": it reaches alice and bob
      alike, with the key unlocked `canWriteLocal` is true and a post is signed with that profile's
      identity, and to try a write point `RAD_HOME` at a throwaway profile seeded the way
      `write.yaml`'s is (`docs/writes.md`). `docs/e2e.md`'s step-9 paragraph says "It also makes that
      instance's writes real" and points there. Decision 4 records why the warning sits beside the
      recipe.

## Areas checked and clean

Prose rather than boxes, because nobody needs to act on them.

- **Pins are the ones they claim to be.** `[repos.basecamp].pin`
  `bbe5da0e…` is the commit under `logos-basecamp` tag `0.3.0`;
  `[repos.lgpm].pin` `d3af2972…` exists in `logos-package-manager` and both
  equal scaffold v0.5.0's `DEFAULT_BASECAMP_PIN` / `DEFAULT_LGPM_PIN`
  (read from `src/constants.rs` at tag `v0.5.0`). The `logos-module-builder`
  entry in `radicle/flake.lock` locks rev `6e46d913…`, which is the commit
  under that repo's `0.3.0` tag. Neither is a fork-only SHA.
- **`radicle/flake.lock` (46k-line diff).** Every locked node is type `github`
  with owner `logos-co`, `NixOS` or `oxalica`; none lacks a `narHash` or a
  `rev`; the `nixpkgs-fetch` node that feeds `fetchCargoVendor` is byte-identical
  to `origin/main`'s, and no Rust dependency moved, so the vendor hash in
  `radicle/flake.nix` has nothing to follow. `radicle-ui` has no tracked lock
  by design (Decision 3 records why); its builder input is a tag ref, resolved
  on each build, which is the same exposure it had at `0.2.6`.
- **Supply chain of the tools.** `logos-scaffold 0.5.0` is on crates.io and
  installed `--locked`. `@paradoxcomputer/sitometres@0.2.0` is an exact version
  (not a range), published with npm provenance and registry signatures, no
  install-time script beyond `prepare`/`build` for its own development; its
  one dependency `yaml ^2.8.1` floats under `npx`, as 0.1.2's did.
- **The new CI build step.** `BASECAMP_REV` is regex-gated to 40 lowercase hex
  before it reaches `$GITHUB_ENV`, so `"github:logos-co/logos-basecamp/$BASECAMP_REV#app"`
  and `"$drv^*"` cannot be steered by shell metacharacters. The workflow is
  `pull_request` (not `_target`), takes no secrets, and `accept-flake-config`
  was already set in `nix.conf` before this change. The memory sampler prints
  only `date`, `/proc/meminfo`, `free` and `df`; the `trap … EXIT` reaps it.
  The substitute-signature failures the comment describes are nix *refusing*
  unverifiable substitutes and building from source, which is the safe
  direction; the comment rightly does not suggest relaxing `require-sigs`.
- **Local profile isolation.** The per-profile `HOME` scaffold 0.5.0 sets
  narrows what an `lgs`-launched Basecamp can reach (no `~/.radicle` unless
  `RAD_HOME` is exported); `LocalStore`'s `RAD_HOME`-else-`$HOME/.radicle`
  resolution is unchanged and was not touched. No C++, Rust or QML source
  changed, so the FFI `guarded()` surface, the QtRO boundary and the
  `canWriteLocal` gating are unaffected by this diff; the only source edit is
  a comment in `settings_store.h`.
- **The new `WAYLAND_DISPLAY = "/run/user/1000/wayland-0"`.** A tracked,
  machine-specific path, but it only selects a socket in a 0700 directory the
  wrong uid cannot traverse, so a mismatched machine fails to start rather
  than connecting elsewhere. A usability cost, not a security one; the
  comment already says how to adjust it.
- **The spec edits.** `source.yaml`'s `eval:` reads and scrolls an in-app
  `ListView` and throws on absence; `sync.yaml` only reorders steps and adds
  one `eval:` that resets a property. Neither touches a profile, the
  filesystem or the network beyond what the specs already did.
- **No error-message or path leaks** were introduced; no prose edit adds a key
  or credential, and `docs/PLAN.md`'s new known-defect paragraph is
  correctness/availability material, not a security claim.
