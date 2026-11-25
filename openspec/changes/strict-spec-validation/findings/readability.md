# Readability review: strict-spec-validation

One `code-reviewer` instance covering all four dimensions; this is the
readability file. Comments, names and doc wording were read against the code and
against `CLAUDE.md`'s "Keeping documents true"; nothing here depends on running
a test (none could be run in this pod: no Qt 6 runner, compiler or Nix).

## Clean areas

- **`ci.yml`'s step comment** says why (strict is what makes the warning a
  failure), why the pin is exact, why it is in `Lint`, how to reproduce, and what
  a green does not prove. That is the kind of "why this and not the obvious
  alternative" comment the repo wants, and it points at the design rather than
  restating it. The "same version as the ui-specs job" pointer is true.
- **`docs/OPENSPEC-ARCHIVE.md`'s new list** is specific, each item was
  measured or read from `validator.js` and says which, and the claims I could
  check (the `MODIFIED` rule, the text-before-first-scenario rule, the
  `Spec.md` skip) match the CLI source. "measured on 2.1.1" names its own
  provenance, so it invalidates itself on a bump rather than silently going
  stale. A style preference, not a defect: "a declined promotion, above, and a
  self-contradiction, just above" leans on position words that break the first
  time a paragraph moves; naming the paragraph would not.
- **`CLAUDE.md`'s edit** names the command rather than the version and sends the
  reader to the doc that holds the blind spots.
- **The new tests' names and comments** read as scenarios, state what differs
  between a leg and its control, and name the mutation they were written
  against. They are long, and so is the rest of these files; I counted that as
  house style rather than noise. The few comments that restate the code are
  inside blocks that also say why.

## Findings

- [ ] **`dev-writer`** — `radicle-ui/src/qml/SourceState.qml:183-186` — a code comment records the status of a test run, and in words that read as something else
      **Scenario:** the new last sentence says the scenario test "added after
      that mutation ... expects `[embedded]:local:shown` and so reads as a
      fourth; that one has not been run under the mutation." Three problems for
      the next reader. (1) "has not been run" is process state that is already
      false the day anyone runs it and nobody will edit a `SourceState` comment
      to say so; it is the kind of line `CLAUDE.md`'s "Keeping documents true"
      says to delete, and the claim belongs in `tasks.md` (where the tester's
      un-run mutations are already recorded) not in the component. (2) "reads as
      a fourth" means "a fourth test the mutation would redden", in a comment
      whose previous paragraph is about "a fourth mode"; the two are one
      sentence apart and a reader takes the wrong one. (3) "the scenario test
      added after that mutation" is history, which git has. The accurate,
      durable content is one clause: the rewritten rule is also pinned by
      `test_the_identity_is_shown_in_exactly_the_modes_that_derive_local`.
      **Severity:** low; a readability and document-truth defect, not a
      correctness one. The rest of the rewritten comment is accurate against
      the code.
