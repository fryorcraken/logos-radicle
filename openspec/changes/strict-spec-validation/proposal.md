# Proposal

## Why

Nothing in CI reads `openspec/specs/`, so a live spec can stop parsing, lose a
scenario or gain a malformed requirement and still merge with every gate green.
The step that would catch it, `openspec validate --specs --strict`, cannot be
added as the specs stand: on `@fission-ai/openspec` 2.1.1 strict mode fails all
nine live specs, on one finding alone — a requirement whose description (the
text between its heading and its first scenario) is longer than 500
characters. Measured with that CLI, 73 of the 102 live requirements are over
the limit.

## What Changes

- **`ci.yml` runs `openspec validate --specs --strict`**, so a live spec that
  fails validation, a requirement grown past the length limit included, fails
  a required check.
- **Every over-long requirement in the nine live specs is cut to 500
  characters or fewer**, under `MODIFIED`, keeping its heading and every one of
  its scenarios byte for byte. What is cut is of two kinds:
  - **Reasoning** — why a rule exists, how a defect shipped, which tests pin
    what, what is still an open question. A spec is a behaviour contract, so it
    leaves the spec. It is carried into this change's `design.md`, or, where it
    names work not done, it is already a GitHub issue.
  - **A second behaviour** stated in the same requirement. It becomes its own
    `ADDED` requirement, with a scenario of its own. There are 37.
- **Reasoning after a requirement's last scenario leaves too**, although the
  length rule does not measure it: one passage in `node-paths`' "Embedded mode
  must never resolve the user's own Radicle home", one in `source-modes`' "A
  mode that cannot start asks its node for nothing". No scenario step changes.
- **No behaviour changes.** Every `MUST` and `MUST NOT` in the live specs is
  still asserted, in the requirement that held it or in one split from it, with
  one exception. `node-paths`' "Socket precedence prefers the runtime directory
  over the home" no longer says separately that `local` mode MUST still find
  the home-derived socket when no runtime directory exists. It does not need
  to: the precedence that requirement states returns that socket in exactly
  that case.
- **One term the live text left implicit is stated.** `embedded-header`'s rule
  is keyed on whether a mode "has an identity", which the live spec never
  defined. Its split-out requirement says a mode has one exactly when it
  derives the `local` method prefix, which is how the view already decides it,
  so a mode the view does not recognise, which `source-modes` routes to the
  seed, shows no identity.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

Each has its over-long requirements restructured to meet the length limit, with
no change to what the module does:

- `embedded-header`
- `embedded-identity`
- `embedded-setup`
- `embedded-state`
- `module-settings`
- `node-config`
- `node-paths`
- `node-seeding`
- `source-modes`

## Impact

- **`.github/workflows/ci.yml`**: a validation step in a required job.
- **`openspec/specs/`**: all nine capabilities, once this change is archived.
  No existing requirement heading changes, so a citation by requirement name
  still resolves.
- **Tests**: no existing scenario changes. The 37 `ADDED` requirements each
  carry a new scenario, which a test has to pin.
- **No code in `radicle/` or `radicle-ui/` changes**, unless a new scenario
  finds the code does not do what the live contract already required.
