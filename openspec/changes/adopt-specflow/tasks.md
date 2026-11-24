## Stages

- [ ] ~~spec — `spec-writer`~~ — no behaviour change in either module; the proposal is written and the change declares `skip_specs: true`
- [ ] design + code — `dev-writer`
- [ ] ~~tests — `tester`~~ — no spec requirement to test; the new `ci.yml` step is exercised by the PR's own CI run
- [ ] review: correctness — `code-reviewer`
- [ ] review: security — `code-reviewer`
- [ ] review: readability — `code-reviewer`
- [ ] review: architecture — `code-reviewer`
- [ ] ~~review: spec-test — `spec-test-reviewer`~~ — no spec delta and no tests for it to read
- [ ] review: design — `design-reviewer`
- [ ] findings all ticked, `findings/` deleted — `closer`
- [ ] `openspec validate --strict`, then `archive` — `closer`
- [ ] CI green, title/body checked, PR merged — `closer`
