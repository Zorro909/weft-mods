"""Tests for `commit-bumps.sh`. Run: `python3 -m unittest discover -s tools/ci`. Needs git and cargo."""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("commit-bumps.sh")

# Local submodules need `protocol.file.allow`; the identity is for the fixture's own commits.
GIT_ENV = {
    **os.environ,
    "GIT_CONFIG_COUNT": "1",
    "GIT_CONFIG_KEY_0": "protocol.file.allow",
    "GIT_CONFIG_VALUE_0": "always",
    "GIT_AUTHOR_NAME": "test",
    "GIT_AUTHOR_EMAIL": "test@example.com",
    "GIT_COMMITTER_NAME": "test",
    "GIT_COMMITTER_EMAIL": "test@example.com",
}

MOD_MANIFEST = """[workspace]

[package]
name = "demo"
version = "0.1.0"
edition = "2021"

[dependencies]
weft_sdk = { path = "../Weft/crates/sdk" }
"""

SDK_MANIFEST = """[package]
name = "weft_sdk"
version = "0.1.0"
edition = "2021"
"""


def run(cwd: Path, *args: str) -> str:
    return subprocess.run(args, cwd=cwd, env=GIT_ENV, check=True, capture_output=True, text=True).stdout


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def make_engine(path: Path) -> None:
    write(path / "crates/sdk/Cargo.toml", SDK_MANIFEST)
    write(path / "crates/sdk/src/lib.rs", "")
    run(path, "git", "init", "-q")
    run(path, "git", "add", ".")
    run(path, "git", "commit", "-q", "-m", "engine")


def make_mods(path: Path, engine: Path, origin: Path) -> None:
    """A repository like this one, with one mod on the engine, pushed to ORIGIN's main."""
    run(path.parent, "git", "init", "-q", "-b", "main", path.name)
    run(path, "git", "submodule", "add", "-q", str(engine), "Weft")
    write(path / "demo/Cargo.toml", MOD_MANIFEST)
    write(path / "demo/src/lib.rs", "")
    run(path, "cargo", "generate-lockfile", "--quiet", "--manifest-path", "demo/Cargo.toml")
    write(path / "tools/ci/commit-bumps.sh", SCRIPT.read_text())
    (path / "tools/ci/commit-bumps.sh").chmod(0o755)
    run(path, "git", "add", ".")
    run(path, "git", "commit", "-q", "-m", "mods")
    run(path, "git", "push", "-q", str(origin), "HEAD:main")


class CommitBumps(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.root)

    def test_bumps_in_a_checkout_without_the_engine(self):
        origin = self.root / "origin.git"
        run(self.root, "git", "init", "-q", "--bare", str(origin))
        make_engine(self.root / "engine")
        make_mods(self.root / "mods", self.root / "engine", origin)
        # As CI checks out the release plan: without submodules.
        run(self.root, "git", "clone", "-q", str(origin), "ci")

        plan = json.dumps([{"folder": "demo", "id": "demo", "version": "0.1.1-rc.1", "bump": True}])
        ref = run(self.root / "ci", "tools/ci/commit-bumps.sh", plan, "main").strip()

        manifest = run(origin, "git", "show", f"{ref}:demo/Cargo.toml")
        self.assertIn('version = "0.1.1-rc.1"', manifest, "the bump reaches main")
        self.assertEqual(run(origin, "git", "rev-parse", "main").strip(), ref, "main holds the bump")


if __name__ == "__main__":
    unittest.main()
