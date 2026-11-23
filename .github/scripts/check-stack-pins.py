#!/usr/bin/env python3
"""Every copy of a Logos stack pin names the same version.

The scaffold release, the module-builder generation and the sitometres version
are each written in more than one file, because each file is read by a
different tool: a workflow cannot source a shell script, and a flake cannot
read a workflow. They used to be held together by "keep in step with" comments.
A copy left behind still builds, and fails only in the e2e layer or in a
user's Basecamp, so the agreement is asserted here instead.

Each pin is found by searching for its literal shape rather than by reading
named lines, so a NEW copy is checked too. `required` names the files that
must hold one, so renaming the variable in a file cannot make the check pass
by finding nothing.

Run from the repository root: `python3 .github/scripts/check-stack-pins.py`.
"""

import re
import subprocess
import sys

PINS = [
    {
        "name": "LGS_VERSION (the scaffold release)",
        "pattern": r'LGS_VERSION: "([^"]+)"',
        "paths": [".github/workflows"],
        "required": [".github/workflows/ci.yml", ".github/workflows/ui-tests.yml"],
    },
    {
        # Both modules must link the SDK set of the one Basecamp scaffold
        # supports; a module built against another is refused at runtime.
        "name": "logos-module-builder tag",
        "pattern": r'github:logos-co/logos-module-builder/([^"]+)"',
        "paths": ["radicle/flake.nix", "radicle-ui/flake.nix"],
        "required": ["radicle/flake.nix", "radicle-ui/flake.nix"],
    },
    {
        # CI and the local wrapper scripts must drive Basecamp with the same
        # tool, and the documented command must be one that can open the app.
        "name": "sitometres version",
        "pattern": r"@paradoxcomputer/sitometres@([0-9][^\"' ]*)",
        "paths": [".github", "radicle-ui", "docs"],
        "required": [
            ".github/workflows/ui-tests.yml",
            "radicle-ui/tests/e2e-env.sh",
            "docs/e2e.md",
        ],
    },
]


def copies(pattern, paths):
    """(file, line, version) for every match of `pattern` under `paths`."""
    out = subprocess.run(
        ["git", "grep", "-n", "-I", "-E", pattern, "--", *paths],
        capture_output=True,
        text=True,
    )
    # git grep exits 1 for "no match", which the `required` check reports.
    if out.returncode not in (0, 1):
        sys.exit(f"git grep failed: {out.stderr.strip()}")
    found = []
    for row in out.stdout.splitlines():
        path, line, text = row.split(":", 2)
        for version in re.findall(pattern, text):
            found.append((path, int(line), version))
    return found


def check(pin):
    """Error strings for one pin; empty when every copy agrees."""
    found = copies(pin["pattern"], pin["paths"])
    errors = []
    for path in pin["required"]:
        if not any(f == path for f, _, _ in found):
            errors.append(f"{pin['name']}: no copy found in {path}")
    versions = sorted({v for _, _, v in found})
    if len(versions) > 1:
        where = ", ".join(f"{f}:{n} = {v}" for f, n, v in found)
        errors.append(f"{pin['name']}: copies disagree ({where}); move them together")
    if not errors:
        print(f"ok: {pin['name']} is {versions[0]} in all {len(found)} copies")
    return errors


def main():
    errors = [e for pin in PINS for e in check(pin)]
    for e in errors:
        print(f"::error::{e}")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
