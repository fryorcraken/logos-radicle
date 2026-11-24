#!/usr/bin/env python3
"""Every live OpenSpec spec validates, and every one of them was validated.

Runs `openspec validate --specs` over `openspec/specs/` and fails on:

- a spec the CLI reports invalid: a structural error, such as a requirement
  with no scenario or with no MUST/SHALL. Warnings are printed and do NOT fail
  the run, which is why this is not `--strict`.
- a capability directory the CLI did not report on. The CLI exits 0 having
  validated nothing when it finds no specs, and it skips a directory with no
  `spec.md` in it without a word, so its own exit code cannot say whether it
  looked at everything.

Both reasons, and the CLI version, are recorded in the `ci-validate-specs`
change's design.md (under `openspec/changes/archive/` once merged).

Run it from anywhere: `python3 .github/scripts/validate-specs.py`. It runs the
CLI from the repository root, because `openspec` takes its root from the cwd.
"""

import json
import os
import subprocess
import sys
from pathlib import Path

# Exact, so a new CLI release cannot change what this gate rejects without a
# commit here saying so. This is the only copy: write the version nowhere else.
OPENSPEC = "@fission-ai/openspec@2.1.1"

ROOT = Path(__file__).resolve().parents[2]
SPECS = ROOT / "openspec" / "specs"


def live_capabilities():
    """Names of the capability directories under openspec/specs/."""
    if not SPECS.is_dir():
        return set()
    return {p.name for p in SPECS.iterdir() if p.is_dir()}


def run_validate():
    """(exit code, parsed JSON report or None, stderr) from the pinned CLI."""
    out = subprocess.run(
        ["npx", "--yes", OPENSPEC, "validate", "--specs", "--json", "--no-interactive"],
        cwd=ROOT,
        env={**os.environ, "OPENSPEC_TELEMETRY": "0"},
        capture_output=True,
        text=True,
    )
    try:
        report = json.loads(out.stdout)
    except json.JSONDecodeError:
        report = None
    return out.returncode, report, out.stderr


def annotation(text):
    """`text` escaped for a GitHub workflow command, which ends at a newline."""
    return text.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")


def print_report(items):
    """Every spec's verdict and every issue, errors also as PR annotations."""
    for item in items:
        print(f"{'ok' if item['valid'] else 'INVALID'}: spec/{item['id']}")
        for issue in item["issues"]:
            print(f"  {issue['level']} {issue['path']}: {issue['message']}")
            if issue["level"] == "ERROR":
                spec = f"openspec/specs/{item['id']}/spec.md"
                print(f"::error file={spec}::{annotation(issue['path'] + ': ' + issue['message'])}")


def problems(returncode, report, expected):
    """Why this run fails, as strings; empty when it passes."""
    if report is None:
        return [f"openspec printed no JSON report (exit {returncode})"]
    items = report["items"]
    errors = [f"spec/{i['id']} is invalid" for i in items if not i["valid"]]
    # NO SPEC: the proposal requires a failure on a structural error and says
    # nothing about a run that validated nothing. This fails it, because a gate
    # that measured nothing and exits 0 is the qmllint step's old defect.
    if not expected:
        errors.append(f"no capability directories under {SPECS.relative_to(ROOT)}/")
    # NO SPEC: a capability directory the CLI skipped, which it does silently
    # for a directory with no spec.md, fails the run rather than passing it.
    seen = {i["id"] for i in items}
    for name in sorted(expected - seen):
        errors.append(f"openspec/specs/{name}/ was not validated: is its spec.md missing?")
    if returncode != 0 and not errors:
        errors.append(f"openspec exited {returncode} with every spec reported valid")
    return errors


def main():
    returncode, report, stderr = run_validate()
    if stderr.strip():
        print(stderr, file=sys.stderr)
    if report is not None:
        print_report(report["items"])
    errors = problems(returncode, report, live_capabilities())
    for e in errors:
        print(f"::error::{e}")
    if not errors:
        print(f"ok: all {len(report['items'])} live specs valid (warnings are advisory)")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
