# Design review: scaffold-0-5

Reviewed `design.md` (8 decisions) against the diff `origin/main...HEAD` and
against `docs/PLAN.md` as it stands on `origin/main`.

The decisions are in good shape. Every one was found implemented as recorded:
the pins match Decisions 1 to 3, `runtime_dir` is gone and `WAYLAND_DISPLAY`
is absolute (5), `SITOMETRES` is `0.2.0` in both `ui-tests.yml` and
`e2e-env.sh` (6), and the evaluate-then-build split is in the workflow (8).
Nothing contradicts a recorded decision. The entries carry alternatives, costs
and, where the decision is a guard (the pin set, the split), what goes red
without it. I checked the one place the split could silently differ from the
single command: `nix build "$drv^*"` drops `--accept-flake-config`, but
`ui-tests.yml:328-330` already sets `accept-flake-config = true` and the logos
substituter in `nix.conf`, so the build step still sees them. No finding there.

PLAN.md on `origin/main` had nothing about `runtime_dir`, scaffold versions or
sitometres to migrate, and this change contradicts nothing in it.

What remains is one stale cross-reference, one half-finished task, one
duplicated block, and one trap in the wrong place.

- [ ] **`dev-writer`** — `tasks.md:64` cites the wrong decision
      Task 3.1 says `basecamp doctor` reports the pin set as "scaffold defaults"
      "(design Decision 6)". Decision 6 is sitometres. That sentence is in
      Decision 7, and it is Decision 2's check from the other side. A reader
      following the pointer lands on the wrong entry.

- [ ] **`dev-writer`** — task 3.3 is unticked and Decision 4 rests on it
      `tasks.md:74` is the only unticked task, and its own text says why:
      no launch was driven into Local mode and none ran with `RAD_HOME`
      exported. Decision 4 states the route back honestly as following from
      inheritance rather than from a run, so the design is not overclaiming.
      But an unticked box blocks the merge. Either run it, or rewrite 3.3 to
      what was actually done (the environment read back from `/proc`) and tick
      it, leaving the `RAD_HOME` claim in Decision 4 as the recorded
      untested step.

- [ ] **`dev-writer`** — the cancel-sync defect is written out twice
      `design.md:407-418` (Risks) and `docs/PLAN.md` "Known defects in shipped
      behaviour" both carry the mechanism (`cancelSync()` only bumps
      `syncEpoch`) and the 116 s measurement. The defect is unbuilt work, so
      PLAN owns it; the design's job is only why `sync.yaml` is ordered as it
      is. Trim the Risks entry to that and a pointer to PLAN, or the two copies
      will drift. Note the PLAN copy also names `bc24ded`, a commit the design
      names too.

- [ ] **`dev-writer`** — `docs/e2e.md:14` still gives the single-process Basecamp build
      Step 1 of the e2e doc is the same `nix build "github:logos-co/logos-basecamp/...#app"`
      that Decision 8 shows needs about 10 GB for evaluation and holds it
      through the compile. The trap lives only in a `ui-tests.yml` comment and
      the archived design. Whoever builds the e2e Basecamp locally on a small
      machine meets the same memory failure with no hint, and `e2e-env.sh:38`
      prints the same one-step command in its error. Add two lines to
      `docs/e2e.md` step 1: the memory figure, and the
      `nix path-info --derivation` then `nix build '<drv>^*'` form as the
      fallback.

- [ ] **`dev-writer`** — Decision 5's evidence is only as current as one machine (suggestion)
      The Wayland table cannot be re-run by any gate, which the entry says. It
      does not say what would make a future reader doubt it: a Basecamp
      launcher that stops forcing `QT_QPA_PLATFORM=wayland` (the claim cites
      `nix/app.nix:702-704` at 0.3.0) would turn the "no Xwayland fallback"
      reasoning false while everything stays green. Add one sentence that the
      launcher line is the thing to re-read when the Basecamp pin moves.

- [ ] **`dev-writer`** — frozen research doc still says `runtime_dir` is pinned (suggestion)
      `docs/M3-phase0-findings.md:425` and `:457` state the pin as current and
      are not annotated, while `M3-embedded-node-plan.md:728` was updated. The
      findings are frozen, so a one-line pointer to Decision 5 is enough. Its
      measured socket paths (under `/run/user/1000`) are also no longer what
      `launch` produces, which Decision 5 notes only for the embedded node's
      socket.
