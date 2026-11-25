# Security review: strict-spec-validation

One `code-reviewer` instance covering all four dimensions; this is the
security file. Nothing here was run against a Qt, C++ or Rust toolchain (none in
this pod); the workflow and CLI claims were checked by running the pinned CLI
and reading its source and metadata.

## What the change exposes

The only new attack surface is the `Lint` step: CI downloads and runs a
third-party npm CLI on every pull request (including from forks), and that CLI
parses markdown a PR author controls (`openspec/specs/**`).

## Clean areas, checked

- **Token and permissions.** The workflow sets no top-level `permissions:`; the
  repository default is read (`gh api repos/fryorcraken/logos-radicle/actions/
  permissions/workflow` returns `default_workflow_permissions: read`), and only
  the `release` job asks for `contents: write`. So the step runs with a
  read-only token and no secrets in its environment. `Lint` also runs on `main`
  pushes and tags, with the same read default.
- **The CLI package.** `@fission-ai/openspec@2.1.1`: MIT (compatible with this
  repo's MIT/Apache-2.0), no `preinstall`/`install`/`postinstall` script in it or
  in any of the 66 packages `npm install` resolved for it (searched every
  `package.json`), declared `engines` `>=22.20.0`, which the Node 24 step
  satisfies. A new dependency question: it is needed (nothing else reads the
  live specs), maintained, and licence-compatible.
- **Telemetry and the version check.** `OPENSPEC_TELEMETRY=0` is read by
  `dist/telemetry/opt-out.js` as disabled, and that file's own header says the
  version check shares it, so no outbound call beyond the registry fetch is made
  by the step. The comment's claim holds.
- **Untrusted spec input.** The CLI reads `openspec/specs/**` and
  `openspec/config.yaml` from the PR. I found no code-execution hook in what the
  validate path reads; it is parsed as markdown/YAML. The CLI has a
  `spec-path-containment.js`. I did not audit it, and did not fuzz the parser.
- **Secrets and paths in test fixtures.** The Rust and C++ tests create
  throwaway profiles under scratch directories with a made-up passphrase
  (`correct horse battery`) and scratch aliases; no real key, home or path is
  referenced, and `radHome` in the C++ tests is the test's own `ScopedRadHome`
  directory. The Rust `assert_eq!(before, after)` on a `BTreeMap<String,
  Vec<u8>>` of the `keys` directory would print the bytes of the throwaway
  encrypted key into a CI log if it ever failed; they are not secret, so there
  is nothing to act on, but it is the one place a fixture's key material can
  reach a log.
- **No reachable panic, index or allocation from untrusted input** was added:
  the source diff is a workflow step, docs and a comment; the Rust additions are
  tests.

## Findings

- [x] **`dev-writer`** — `.github/workflows/ci.yml:132-135` — "pinned exactly ... so an upstream release cannot change what this gate rejects" is not true of the transitive dependencies
      **Scenario:** `npx --yes @fission-ai/openspec@2.1.1` pins the CLI, but its
      `dependencies` are caret ranges resolved fresh on every run, with no
      lockfile and no integrity check beyond npm's own: `zod ^4.4.3`, `yaml
      ^2.8.3`, `diff ^9.0.0`, `commander ^14.0.0`, `chalk ^6.0.1`, `ora ^9.4.1`,
      `picomatch ^4.0.4`, `tinyglobby ^0.2.17`, `brace-expansion ^5.0.12`
      (`npm view @fission-ai/openspec@2.1.1 dependencies`; only `skills` and
      `cross-spawn` are exact). The step runs `zod`'s schema validation over PR
      input, so a minor release of `zod` or `yaml` can change what passes or
      fails `Lint` with no commit here, which is the thing the comment says the
      pin prevents; and a compromised release of any of those nine packages runs
      in this job. Today the damage is bounded (read-only token, no secrets, no
      install scripts), so this is a supply-chain posture and accuracy finding,
      not an exploit. Either soften the comment to say the CLI is pinned and its
      dependencies are not (and that the repo accepts that), or pin the tree
      (a committed lockfile and `npm ci`, or a vendored copy).
      **Severity:** low. **Measured:** dependency ranges read from
      `npm view`; the install resolved 66 packages with no install scripts.
      **Fixed, by softening the comment; the tree stays unpinned, on
      purpose**, in the commit "Act on the dev-writer review findings in
      strict-spec-validation". The comment beside the step now says the CLI
      is pinned so a new *CLI* release cannot move what the gate rejects,
      that its own dependencies are not pinned and resolve their caret
      ranges fresh each run, `zod` (which runs the spec schemas) and `yaml`
      among them, so a release of one can still move a verdict, and that
      this is accepted, pointing at Decision 2. Decision 2's "Not pinned"
      paragraph had claimed the rules live in the CLI's own code, which the
      pin fixes; it now says the length rule is the CLI's but the schemas
      run on `zod` (`dist/core/schemas/*.schema.js`), gives the measured
      ranges (`npm view @fission-ai/openspec@2.1.1 dependencies` today: 13
      direct, 11 caret ranges, `skills` and `cross-spawn` exact; the two
      ranged ones this finding does not list are `@inquirer/core` and
      `@inquirer/prompts`), and records
      the lockfile with `npm ci` as considered and not taken, with why. The
      Risks entry now covers a verdict moving as well as a compromise.
      Comment and prose only; no test can fail without it.
