# Readability findings — `scaffold-0-5`

Dimension: readability only. Material reviewed is prose, workflow comments,
`scaffold.toml`, two sitometres specs and a one-line header comment, since the
change has no module source diff. The other three dimensions are other
instances' work.

- [x] **`dev-writer`** — `.github/workflows/ui-tests.yml:22-32,60-62,79-87` — a
      new paragraph was inserted into a header whose surrounding claims it
      contradicts, so the header now argues with itself.
      **Scenario:** a reader reaches line 22, "3m14s is comfortably
      PR-affordable", takes it in, and meets the correction at line 25 ("on
      0.3.0 a cold job took 24–39 minutes"). Line 31 then says a fork PR "would
      always run cold, i.e. ~9m", line 60 calls the 8m21s observation the
      comparator for the 120-minute backstop, and line 82 repeats "the ~9m cold
      price". All of those are Basecamp 0.2.3 numbers that the change itself
      says no longer hold for the cold case. CLAUDE.md's "prune as you go"
      applies: patching one paragraph and leaving the three that depend on it
      is the shape that rule was written against.
      **Severity:** medium (comprehension; a reader sizing a PR-trigger or
      timeout decision from this header gets the wrong number).
      **Fixed** in the commit that ticks this box, together with the architecture box on the same
      lines: the header no longer carries any duration, so there is nothing left to contradict. It
      states the shape of the cost (warm nearly free; cold mostly compiling on 0.3.0; which runs
      are cold), and the `timeout-minutes`, `on:` and `BASECAMP_REV` notes point at that.

- [x] **`dev-writer`** — `CLAUDE.md:423`, `scaffold.toml:80`,
      `openspec/changes/scaffold-0-5/design.md` Decision 7 — "deleted all 29
      comment lines" is a count, and it does not match the file it describes.
      **Scenario:** `git grep -c -E "^\s*#" -- scaffold.toml` on this branch
      returns 39. The 29 was taken on an earlier draft of the comment block,
      before it was finished, so a reader who verifies "all 29" finds 39 and
      cannot tell which is right. CLAUDE.md's own "Keeping this file true"
      table lists counts among the things not to write down; the sentence
      survives without one ("deleted every comment line and changed no value").
      **Severity:** low-medium; a concrete, checkable inconsistency in a section
      whose whole point is "measured rather than read off release notes".
      **Fixed** in the commit that ticks this box: the count is gone from all four places
      (CLAUDE.md, `scaffold.toml`, Decision 7, task 3.1), each now "deleted every comment line and
      changed no value".

- [x] **`dev-writer`** — `CLAUDE.md:393-395` — the "one exception" paragraph is
      orphaned from the rule it excepts, and says the same thing as the next
      section.
      **Scenario:** the rule is "put environment in `scaffold.toml`"
      (line 378). Between it and the exception sit the `runtime_dir` paragraph
      and the test-scripts paragraph ("The same principle covers…"), so
      "**The one exception**" opens with no referent. Lines 410-412 in the
      following section then state it again ("It does not go in
      `scaffold.toml`: the value is one developer's path, and the file is
      tracked"). Move the exception up beside line 378-379 (or fold it into
      the Local-mode section and delete it here).
      **Severity:** low; stylistic.
      **Fixed** in the commit that ticks this box: the exception now follows the rule directly
      ("**Except a value that belongs to one developer or one session**"), says where such a value
      goes and why it reaches Basecamp, and names its two instances. The Local-mode section's
      restatement is gone; it says "per the exception above".

- [x] **`dev-writer`** — `docs/e2e.md:36-46`, `radicle-ui/tests/ui/browse.yaml:60`,
      `radicle-ui/tests/ui/write.yaml:63`, `radicle-ui/tests/ui/sync.yaml:125` —
      three documents disagree about whether this suite asserts on `calls:`.
      **Scenario:** `e2e.md` now says "no spec here has yet relied on a
      `calls:` assertion". `sync.yaml:125` adds
      `expect: calls: ["radicle.remoteListBranches"]`, so one does. Meanwhile
      `browse.yaml` ("deliberately absent throughout") and `write.yaml` ("No
      `calls:` anywhere, on purpose") still explain the absence with the 0.1.x
      behaviour that `e2e.md` now says changed on 0.2.0 / 0.3.0. Someone adding
      a step cannot tell from these whether `calls:` is a usable gate. Update
      the `e2e.md` sentence to name `sync.yaml` as the first user (it is the
      evidence), and say in the two older comments that they predate 0.2.0.
      **Severity:** medium (contradictory guidance in the docs a spec author is
      told to read first).
      **Fixed** in the commit that ticks this box: `e2e.md` names `sync.yaml`'s staleness step as
      the first `calls:` assertion, says it sits beside a `state:` that cannot hold without the call,
      and that it has not been shown failing on its own, so not to count a `calls:` line as a gate
      until someone has. `browse.yaml` and `write.yaml` now say their reasoning was measured on
      sitometres 0.1.x / Basecamp 0.2.3 and point at `e2e.md`. Whether the `calls:` line
      discriminates is the tester's open spec-test box, not answered here.

- [ ] **`tester`** — `radicle-ui/tests/ui/source.yaml:124-142` — the new scroll
      step was inserted between a comment and the step it describes.
      **Scenario:** lines 124-126 ("Using `text:` to SELECT is fine …") explain
      the `text: "README.md"` selector on the click step. After this change the
      comment is followed by a 14-line paragraph and the `scroll the README row
      into view` step, and the click step it was written for is 20 lines
      further on, under a comment about scrolling. A reader of the click step
      meets `match: "contains"` with its justification above something else.
      Put the scroll block before the "A file row addressed by NAME…" comment,
      or give the click step its own comment again.
      **Severity:** low; stylistic.

- [x] **`dev-writer`** — `docs/e2e.md:56-71`, `docs/PLAN.md` "Known defects in
      shipped behaviour", `radicle-ui/tests/ui/sync.yaml:25-33`, design.md
      Risks — the cancelled-sync story (33-entry repository, 116 s
      `ListBranches`, why the spec polls first) is told in full in four places.
      **Scenario:** when the cancel is fixed, all four need editing, and
      `PLAN.md` says that is when the `sync.yaml` order should be reverted. Two
      of them will be missed. The repo's own rule is that a fact lives in one
      place and the others point at it. `PLAN.md` is the one that carries the
      constraint; `e2e.md` and `sync.yaml` can say one sentence and point there.
      **Severity:** low; stylistic, but this is the exact drift the repo's
      CLAUDE.md "Keeping this file true" section describes.
      **Fixed** in the commit that ticks this box: `docs/PLAN.md` is the one full telling.
      `e2e.md` and `sync.yaml` each keep one sentence (why the spec polls before it cancels) and
      point at PLAN; `sync.yaml` also says the order can revert once the defect is fixed, which is
      the edit PLAN already names. `design.md`'s Risks entry is trimmed to the same plus a pointer.

- [x] **`dev-writer`** — `docs/M3-phase0-findings.md:425,457` and
      `docs/M3-phase0-probe_node_runtime.rs.txt:82` — present-tense claims that
      `scaffold.toml` pins `runtime_dir`, which this change removes.
      **Scenario:** the change corrected the same claim in
      `docs/M3-embedded-node-plan.md:728` and `radicle/src/settings_store.h`,
      but `git grep runtime_dir` still finds three more that say "is why
      `runtime_dir` is pinned in `scaffold.toml`" and "`scaffold.toml` pins
      `runtime_dir`". Whoever reads the Phase 0 findings next trusts a pin that
      is not there. If the document is deliberately a dated record, one line at
      its top saying so settles it; otherwise fix the two sentences.
      **Severity:** low.
      **Fixed** in the commit that ticks this box: the probe comment is corrected to the past
      tense; the findings document, which is a dated spike record, keeps its sentences and gains an
      italic "Dated:" note at each, pointing at Decision 5. A note at each passage rather than one
      at the top, because a reader grepping `runtime_dir` lands on the passage, not the header.

- [x] **`dev-writer`** — `docs/e2e.md:20` — the sitometres version is now a
      literal in three places (`ui-tests.yml:199`, `e2e-env.sh:28`, and the
      example command here), kept in step by comments that say "keep in step".
      **Scenario:** the next bump edits two and the third keeps telling
      newcomers to run the version that cannot open the app on this Basecamp.
      This diff touched all three, so the cost is real. `e2e-env.sh` already
      exports `SITOMETRES`; the docs example can use `"$SITOMETRES"` (or say
      "the version `e2e-env.sh` pins") instead of a fourth copy of the number.
      **Severity:** low; stylistic.
      **Fixed** differently from the suggestion, in the commit that ticks this box: the literal
      stays and is now checked. `ci.yml`'s lint runs `.github/scripts/check-stack-pins.py`, which
      requires a sitometres copy in `ui-tests.yml`, `e2e-env.sh` and `docs/e2e.md` and fails if any
      two differ (seen failing with `e2e-env.sh` at `0.2.1`). Rejected the `"$SITOMETRES"` form
      because the doc's command is the one a newcomer pastes into a shell that has not sourced
      `e2e-env.sh`, where it would expand to nothing; a checked literal keeps it runnable and
      cannot drift unseen. The "keep in step" comments now point at the lint. `design.md` Decision 9.

## Areas that read well

- `scaffold.toml`'s rewritten comment block says what a command cannot: why the
  pin existed, the two jobs it did, what replaced each, and when to delete the
  line. The uid caveat sits beside the value.
- The workflow's "Evaluate, THEN build" block carries its measurements, the
  rejected alternatives and the consequence of removal, in the file's existing
  header style. The inline resource sampler is dense but commented as
  intentional.
- `sync.yaml`'s reorder is explained once at the top, and the
  "forget the stand-in's claim" step carries a reason, so its removal would be
  noticed. The 40-zero commit literal is set and read within the same file.
- `design.md` Decision 5's table plus the sentence saying which rows used a
  temporary env, and `tasks.md` 3.3 recording exactly what was and was not
  done, are the right level of candour; unticked 3.3 is honest, not a defect of
  prose.
