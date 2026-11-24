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

- [ ] **`dev-writer`** — `proposal.md:49-51,73-82,90` and `tasks.md:5` still describe the CI step as part of this change
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

- [ ] **`dev-writer`** — `design.md` Decision 6 omits what the deferral costs
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

- [ ] **`dev-writer`** — `design.md:70` and the Goals line "Every PLAN.md passage with a future either has an issue or a durable home" are contradicted by the table's own row
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

No findings against the code: both source-file edits are comments and match the
decisions that name them.
