# Design review: adopt-specflow

Reviewed: `design.md` Decisions 1 to 6 against the diff (`origin/main...HEAD`),
the overlay, issue #51 (body and the owner's comment), issues #59, #92 and #93,
and `docs/PLAN.md` as it stood on `origin/main`.

The decisions are in good shape. Decisions 2 to 5 each name what moved, where it
went and what was considered. I checked these against the tree and they hold:

- `git grep` for `PLAN.md`, `.claude/agents` and `RUNNER.md` outside the archive
  and this change returns only the specflow block's generic line in `CLAUDE.md`,
  as task 4.2 says.
- Every Decision 2 row has a landing section in the overlay.
- The `.gitignore` re-admits for `.claude/agents/` are gone and
  `!.claude/settings.json` stays.
- `rust-ffi.md`'s "Platform scope" matches the issue's correction.
  `two_homes_get_two_different_identities` exists at
  `radicle/rust-ffi/tests/profile_init.rs:53`, and #59 is the open issue for the
  second half.
- Decision 6's measurements reproduce. In this tree `openspec validate --specs
  --strict` ends `0 passed, 9 failed` and `openspec validate --specs` ends
  `9 passed, 0 failed`, with the same long-requirement warnings. The deferral to
  #93 contradicts the issue's "Done when", and the departure is argued from
  that measurement. I judge it justified.

What is wrong is the prose around Decision 6 and one decision-table row.

- [x] **`dev-writer`** — `proposal.md:49-51,73-82,90` and `tasks.md:5` still describe the CI step as part of this change
      Decision 6 and `tasks.md`'s Deferred section say `ci.yml` is untouched and
      the step is issue #93. Four other places say otherwise. `proposal.md`'s
      What Changes lists "Add `openspec validate --specs --strict` to `ci.yml`".
      Open question 2 ends "Not decided here". Impact lists
      `.github/workflows/ci.yml` under "Files edited". The stage block's tests
      row says "the new `ci.yml` step is exercised by the PR's own CI run".
      **Verified:** `git diff origin/main...HEAD --stat` shows no `ci.yml`. A
      reader of the proposal or the stage row is told a gate exists that does
      not. Open question 2 should record the owner's answer (deferred to #93),
      as question 1 records its answer. The Impact entry and the tests-row
      reason should be corrected to match.
      **Fixed** by the `spec-writer` in `c95dc25`, checked against the tree: `proposal.md`'s What Changes has no CI bullet and a "Not in this change" paragraph naming #93 (lines 52-54); Open question 2 ends "Answered by the owner: the step is moved out of this piece into issue #93" (lines 85-87); Impact has "**Not edited:** `.github/workflows/ci.yml`" (line 97); `tasks.md:5`'s strike reason says `ci.yml` is not edited here and names #93. `git grep -n "ci.yml" -- openspec/changes/adopt-specflow/proposal.md openspec/changes/adopt-specflow/tasks.md` shows those statements, task 1.1's citation of the `build` matrix and the Deferred note, and nothing claiming an edit.

- [x] **`dev-writer`** — `design.md` Decision 6 omits what the deferral costs
      The entry has the constraint (strict fails all nine specs) and names the
      three ways out. It does not say what the deferral forecloses. The issue
      asked for the step as archived foundations task 4.22 (a declined
      promotion of a delta into `openspec/specs/` ships an unpublished
      contract). Until #93 lands, nothing in CI validates a live spec at all,
      so a structurally broken spec merges unseen. #93's last section adds that
      even the shipped step could not see a declined promotion, and the entry
      does not carry that either. The entry should state both in a sentence
      each, so the gap stays visible while #93 is open. Deferring is the
      owner's decision. This box asks only for what it costs to be written
      down.
      **Fixed** in this commit: Decision 6 gains "What the deferral costs, while #93 is open", a sentence for each: nothing in CI validates a live spec, so a structurally broken one can merge with every required check green (the `closer`'s pre-archive validation is a stage, not a gate); and, from #93's last section (read fresh), `validate --specs` cannot see a declined promotion, which only a check that every capability a proposal names has a live spec would.

- [x] **`dev-writer`** — `design.md:70` and the Goals line "Every PLAN.md passage with a future either has an issue or a durable home" are contradicted by the table's own row
      The "Deferred, deliberately: patch open/update ... and delegate/identity
      management" row says "no issue" and "nothing tracks either item". That
      is a PLAN.md passage with a future and, by the row, neither an issue nor a
      home, which is the failure the goal names. The text is not lost, though.
      `docs/M3-embedded-node-plan.md:677-678` still carries the same deferral,
      and the row does not cite it. Either cite that line as the durable home
      and narrow the goal to say scope lives in the tracker where an issue
      exists, or leave the row as the owner's open item and say in the goal
      that two passages are not yet homed. Right now the goal is false as
      written. This also needs the owner's call (the hand-back is supposed to
      carry it), so the runner should check it is reported.
      **Awaiting owner decision** (`dev-writer`, owner decision 1 as the runner reserved it; no issue created). Options: (a) the owner files issues for patch open/update and delegate/identity management, and the row names them; (b) `docs/M3-embedded-node-plan.md:677-678` is their durable home, and the row cites it as such. Done meanwhile, without choosing: the Goal now reads "…either has an issue or a durable home, except the two deferrals Decision 1 marks as awaiting the owner", which is true as written, and the row cites the M3 line as where the same deferral and its reason survive. Either option then needs only the row and that Goal clause edited. The runner has it in the hand-back.
      **Fixed** in this commit, on the owner's option (a): the owner filed #96 (patch open/update) and #97 (delegate/identity management). Decision 1's row now names both, and still cites the M3 line (and the M2.2 design for patches) as where the reason also survives; the Goal is back to its unqualified form, which is now true as written; the Risks row about untracked deferrals is gone.

No findings against the code: both source-file edits are comments and match the
decisions that name them.
