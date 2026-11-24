# Security review: adopt-specflow

Dimension: **security** only. Material reviewed: `.claude/settings.json` (the
new `permissions`, `extraKnownMarketplaces` and `enabledPlugins`), `.gitignore`,
`.claude/specflow/PROJECT.md`, the `CLAUDE.md` diff, the docs that gained rules
(`docs/rust-ffi.md`, `docs/e2e.md`), and the two comment edits
(`radicle/src/radicle_impl.h`, `radicle-ui/tests/ui/sync.yaml`). There is no
application code diff, so no untrusted-input, FFI or parsing path changed.

Every finding below concerns `.claude/`, which is the owner's. Each is a
proposal for the owner to accept, reject or defer, not something an agent edits
on its own initiative. They are listed under `dev-writer` because that is the
role that wrote the tracked settings in this piece.

What I could not do: I tried to demonstrate the `git -c alias.…=!cmd` escape in
finding 1 and the harness refused the call, so that finding rests on git's
documented alias behaviour, not on a run. None of the permission-matcher
behaviour below was measured against Claude Code's matcher. No mutation tool
applies to prose and config.

- [ ] **`dev-writer`** — `.claude/settings.json:18` — `Bash(git *)` is an unbounded shell-execution and push primitive, so the `gh api` deny list protects nothing
      **Scenario:** git runs arbitrary commands through its own configuration:
      `git -c alias.x='!<any shell command>' x`, `git config alias.x '!…'` followed
      by `git x`, `-c core.sshCommand=…`, `-c credential.helper=!…`. An agent (or
      text it was manipulated by, such as an issue body or a seed response it was
      asked to read) can therefore run any command with no prompt, including
      `gh api repos/o/r -X DELETE …`, which the `deny` entries at lines 50-58 were
      written to refuse. The same allow also covers `git push --force origin
      main`, `git push <any URL> HEAD`, `git reset --hard` and `git clean -fdx`
      without a prompt. `CLAUDE.md` and the specflow block both state that a `gh`
      write "is refused or prompts by design"; with `git *` allowed that is a
      claim about the checker's default, not about what an agent can do.
      **Severity:** medium. Branch protection on `main` (signed commits, admins
      enforced, per the overlay) stops a bad push to `main`, but it does not
      cover the shell escape, a push to another remote, or loss of local work.
      **Suggested direction:** allow the read subcommands the flow actually runs
      (`git status`, `diff`, `log`, `show`, `rev-parse`, `config --get-regexp`,
      `add <path>`, `commit`, `worktree`, `cherry-pick`) and let `git push`
      prompt, since the plugin pushes only by refspec to `piece/<name>` and that
      is one click a piece. If the owner keeps `git *`, the deny list should be
      described as a guard against mistakes, not a boundary, wherever it is
      documented.
      **Awaiting owner decision** (`dev-writer`): `.claude/settings.json` is the owner's, and the runner reserved this choice. Context, read for this: the entry is `/specflow:init`'s own allow list verbatim (its step 1 says "`Bash(git *)` is broad: it covers `git push` and `git reset`" and names branch protection as the backstop), so a narrowing here is also a departure from the plugin that a re-run of init would not undo but would not repeat either. Options: (a) keep `Bash(git *)` and say in `CLAUDE.md`, beside the allow-list paragraph, that the `gh api` deny list is a guard against mistakes, not a boundary; (b) narrow to the subcommands the flow runs (`status`, `diff`, `log`, `show`, `rev-parse`, `config --get-regexp`, `add`, `commit`, `worktree`, `cherry-pick`, `check-ignore`, `grep`, `ls-files`) and let `git push` prompt, one click per piece; (c) raise it upstream on `fryorcraken/agent-spec-flow`, since the list is the plugin's. design.md Context records the list's origin.

- [ ] **`dev-writer`** — `.claude/settings.json:19-36` — the allow list composes into a prompt-free read-then-exfiltrate chain
      **Scenario:** `Bash(grep *)` reads any file the user can read, including
      outside the working directory, where the `Read` tool would stop and ask:
      `grep -r . ~/.radicle/keys`, `grep -r . ~/.ssh`. `CLAUDE.md` tells
      developers to export `RAD_HOME` at a profile for interactive runs, and a
      real profile's `keys/radicle` is the key that signs writes. The result then
      leaves through a channel that is also allowed: `gh pr create --repo <any
      repo> --body '<the text>'`, `gh pr edit` on a public PR, `git push` to an
      arbitrary URL, or a `gh api` GET with the data in the query string. The
      `Grep` tool the specflow block already calls free makes `Bash(grep *)`
      largely redundant, but it widens the read to the whole filesystem.
      **Severity:** medium. It needs an agent steered by untrusted text, which
      this flow does expose (issue and PR bodies, seed data in a spec run, other
      branches' files), and the pod this was reviewed in is not the only machine
      that will load this tracked file.
      **Suggested direction:** drop `Bash(grep *)` and use `Grep`, or accept it
      knowingly. A pattern cannot forbid `--repo` on `gh pr create`/`gh pr edit`,
      so if those stay, say in `CLAUDE.md` that the owner accepted the residual
      risk and why, next to the paragraph about what is allow-listed (lines
      128-131).
      **Awaiting owner decision** (`dev-writer`): reserved by the runner with the `git *` entry above; `.claude/settings.json` is the owner's. `Bash(grep *)` is also in `/specflow:init`'s list. Options: (a) drop `Bash(grep *)`, leaving the `Grep` tool (and `git grep`, under `git *`) for searches; (b) keep it and record the accepted residual risk in `CLAUDE.md` beside the allow-list paragraph, as suggested; (c) raise it upstream, since the entry is the plugin's.

- [ ] **`dev-writer`** — `.claude/settings.json:43` — `Bash(sh radicle-ui/tests/*)` matches a path that leaves the directory
      **Scenario:** the trailing `*` matches `../`, so
      `sh radicle-ui/tests/../../tmp/x.sh` is inside the pattern. `Write` is free
      and `./tmp/` is the sanctioned scratch space, so an agent can author a
      script there and run it without a prompt. `CLAUDE.md` line 146 justifies
      the entry as "the test scripts take no arguments", which is the property a
      bare `*` does not enforce. This is independent of finding 1 and survives
      narrowing `git *`.
      **Severity:** low to medium; I did not measure whether Claude Code
      normalises the path before matching, so confirm that before acting.
      **Suggested direction:** list the three scripts by name
      (`check-qml-syntax.sh`, `lint-qml.sh`, `run-qml-tests.sh`), which matches
      the "take no arguments" rationale exactly.
      **Awaiting owner decision** (`dev-writer`): the fix is an edit to `.claude/settings.json`, which is the owner's and which I was told not to edit, so I cannot make it; this entry was not on the runner's list of reserved decisions and is flagged to it as such. Not measured: whether Claude Code normalises `../` before matching (a probe would cost the owner a prompt or run the escape). Options: (a) replace the entry with one per script, by name (`run-local-e2e.sh` is also in `radicle-ui/tests/` and would need its own if wanted); (b) keep it and accept the residual risk. `CLAUDE.md`'s allow-list paragraph was reworded for the readability finding on the same lines and no longer states more than the rule does.

- [ ] **`dev-writer`** — `.claude/settings.json:5-12` — the plugin marketplace is tracked from a moving ref
      **Scenario:** `extraKnownMarketplaces.agent-spec-flow` names
      `fryorcraken/agent-spec-flow` with no `ref`, and `enabledPlugins` names no
      version. Whatever that repository's default branch holds is what every
      contributor who trusts this folder loads, as agents whose tools are "All
      tools", running under the permissions above. A compromised account, a
      force-push or a bad release there reaches every session on its next
      update, with no commit in this repository saying so. This repo's own
      `CLAUDE.md` rejects exactly this shape elsewhere (the catalog action
      "pinned to `@v1.3`, because a moving tag would let the release pipeline
      change under the catalog without a commit here saying so"; sitometres:
      "pin the version, never a range or `latest`"), and records that an open
      branch changed under it three times.
      **Severity:** low to medium. The source is the owner's own repository,
      which is why it is not higher.
      **Suggested direction:** pin the marketplace source to a tag or commit if
      the settings schema takes a `ref`, and write the pin's reason where the
      other pins are written. I did not check the schema.
      **Awaiting owner decision** (`dev-writer`): an edit to `.claude/settings.json`, which is the owner's; not on the runner's reserved list, flagged to it. The unpinned shape is what `/specflow:init` writes (`{ "source": "github", "repo": "<its repo>" }`, no `ref`). I did not check whether the settings schema takes a `ref`. Options: (a) pin the marketplace source to a tag or commit, if the schema takes one, and record why beside the other pins in `CLAUDE.md`; (b) keep tracking the default branch, accepted because the source is the owner's own repository; (c) raise it upstream so init writes a pinned source.

- [x] **`dev-writer`** — `.gitignore:41-44` — four recursive directory re-admits contradict the comment above them, and two of them are directories this repo never tracks
      **Scenario:** the comment at lines 19-31 argues that `!.claude/<dir>/`
      makes everything beneath it trackable and says to re-admit by name unless
      "everything in it is meant to be tracked". The new block then re-admits
      `.claude/specflow/`, `.claude/skills/`, `.claude/commands/` and
      `.claude/hooks/` as directories. Measured with `git check-ignore -v`:
      `.claude/hooks/pre.sh`, `.claude/commands/x.md` and
      `.claude/specflow/settings.local.json` produce no match, so all three are
      now trackable, whereas `.claude/settings.local.json` is still ignored.
      `git ls-files .claude` shows only `settings.json` and
      `specflow/PROJECT.md` are tracked, so nothing needs the other three
      re-admits. Consequences: a hook script is code that runs on tool calls, so
      one dropped into `.claude/hooks/` and swept in by a loose `git add`
      becomes a tracked, auto-executing file for every clone; and `lgs init` and
      `openspec init` write generated skills and commands into `.claude/skills/`
      and `.claude/commands/`, which the comment this change removed named as
      "generated or machine-local". `CLAUDE.md` says the same of `lgs init`'s
      `.claude/skills/`.
      **Severity:** low. It needs a careless add, and "never `git add -A`" is the
      mitigation the flow already relies on, but the file argues against its own
      rule and `.claude/hooks/` is the one that executes.
      **Suggested direction:** re-admit `.claude/specflow/PROJECT.md` by name, or
      the directory plus a re-ignore of `settings.local.json`, and drop the
      `skills`, `commands` and `hooks` lines until something tracked lives
      there. The block is written by the plugin's init, so this may belong
      upstream, in which case defer it with that destination.
      **Fixed** in this commit, with the suggested narrowing rejected. Dropping `skills`, `commands` and `hooks` would undo `/specflow:init`'s stated reason for them (step 2: a project skill, command or hook must be tracked to reach a worktree cut from `HEAD`), and a re-run of init appends them again. Instead: the comment above `.claude/*` now says why the block is the exception and what it costs; `.claude/skills/basecamp/` (`lgs init`'s skill) and `.claude/**/settings.local.json` are re-ignored by name after the block; and the overlay's `## Never commit` names generated skills and commands under `.claude/`, and says a file under `.claude/hooks/` is committed only on the owner's request because it runs on tool calls for every clone. Check: `git check-ignore -v --no-index .claude/specflow/settings.local.json .claude/hooks/settings.local.json` now matches `.gitignore:85`; `.claude/hooks/pre.sh` remains trackable by design. `openspec init` 2.1.1 with `--tools claude` writes to the user's `~/.claude`, not the project's (measured in `./tmp/`), so there is no openspec name to re-ignore. Decision 2 records it.

Clean areas, in prose: the `radicle_impl.h` and `sync.yaml` edits change comment
text only, and the claims they now make (issue #92, the `seedRepo`
allow-is-not-enough footgun) match the code and the issue. `docs/rust-ffi.md`'s
new rules ("never point a test at the developer's real Radicle home", the
two-home isolation test) are consistent with `tests/profile_init.rs`, and the
text is honest that the second half of the isolation assertion (neither home
written by the other's init) is not yet checked there and is issue #59's. The
`docs/e2e.md` embedded-home paragraph is flagged by its own text as reasoning
from the specs, and the specs it cites (`node-paths` says an embedded home MUST
NOT be the `RAD_HOME` path) support it. The overlay's `## code-reviewer`
section keeps every security rule the deleted `code-reviewer.md` held: untrusted
seed and on-disk input, the sandbox boundary and the home-from-the-view example,
writes into the real Radicle home, error messages leaking paths or key material,
and the narrow interface. The `canWriteLocal` gate is in `## Hazards`. No rule
was dropped from the security checklist in the slimming.
