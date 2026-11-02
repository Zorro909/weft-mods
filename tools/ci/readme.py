"""Rewrites the mod releases section of README.md from the repository's GitHub releases.

Reads the releases from stdin, one JSON object per line with `tag`, `url`, `date` and `body`, as
`update-readme.sh` fetches them. The section between the markers is replaced, or added at the end.
Run from the repository root, with the history of the Weft submodule fetched.
Usage: `python3 tools/ci/readme.py README.md < releases`.
"""

import json
import re
import subprocess
import sys
from collections.abc import Callable
from dataclasses import dataclass

START = "<!-- mod-releases:start -->"
END = "<!-- mod-releases:end -->"
# The tags `plan-release.sh` and `plan-experimental.sh` make.
TAG = re.compile(r"^(?P<mod>.+)-v(?P<version>\d+\.\d+\.\d+)(?:-(?P<kind>rc|experimental)\.\S+)?$")
# The line `publish.sh` writes into every release's notes.
WEFT = re.compile(
    r"Built against Weft (?P<version>\S+), commit (?P<link>\[\w+\]\([^)]*/commit/(?P<commit>\w+)\))"
)

INTRO = """\
## Mods

Put a mod's zip as it is into the game's `mods/` folder; each one holds the Windows, Linux and
macOS builds. A mod loads only into a game with the same mod interface as the Weft it was built
against. Release candidates are builds of `main` since the release; experimental builds use Weft's
upstream `master` instead of the Weft this repository pins.
"""


@dataclass
class Release:
    mod: str
    version: str
    kind: str
    label: str
    url: str
    # ISO 8601, so that comparing the strings compares the times.
    published: str
    # The Weft it was built against, in Markdown, and its full commit ("" when unknown).
    weft: str
    weft_commit: str


# Whether the first Weft commit is the second or one of its ancestors.
WeftContains = Callable[[str, str], bool]


def parse(release: dict) -> Release | None:
    """The release, or None for a tag this repository's workflow didn't make."""
    tag = TAG.match(release["tag"])
    if not tag:
        return None
    weft = WEFT.search(release["body"] or "")
    return Release(
        mod=tag["mod"],
        version=tag["version"],
        kind=tag["kind"] or "release",
        label=release["tag"].removeprefix(tag["mod"] + "-v"),
        url=release["url"],
        published=release["date"],
        weft=f"{weft['version']} {weft['link']}" if weft else "unknown",
        weft_commit=weft["commit"] if weft else "",
    )


def weft_contains(older: str, newer: str) -> bool:
    """Whether the Weft commit `newer` includes `older`, by the submodule's history."""
    ancestor = ["git", "-C", "Weft", "merge-base", "--is-ancestor", older, newer]
    return bool(older and newer) and subprocess.run(ancestor, capture_output=True).returncode == 0


def newest(releases: list[Release], kind: str, after: str = "") -> Release | None:
    """The newest release of `kind` published after the time `after`."""
    found = [r for r in releases if r.kind == kind and r.published > after]
    return max(found, key=lambda r: r.published, default=None)


def cell(release: Release | None) -> str:
    if release is None:
        return "–"
    label = release.version if release.kind == "experimental" else release.label
    return f"[{label}]({release.url}), Weft {release.weft}"


def latest_table(mods: dict[str, list[Release]], contains: WeftContains) -> list[str]:
    """One row per mod: its release, the candidate newer than that, and the experimental build on
    a Weft the newest of those two doesn't have yet. The time an experimental build was published
    says nothing, since it runs beside the release."""
    lines = ["| Mod | Release | Release candidate | Experimental |", "|---|---|---|---|"]
    for mod, releases in mods.items():
        release = newest(releases, "release")
        candidate = newest(releases, "rc", release.published if release else "")
        stable = candidate or release
        experimental = newest(releases, "experimental")
        if experimental and stable and contains(experimental.weft_commit, stable.weft_commit):
            experimental = None
        lines.append(f"| {mod} | {cell(release)} | {cell(candidate)} | {cell(experimental)} |")
    return lines


def history(mod: str, releases: list[Release]) -> list[str]:
    """Every full release of the mod, newest first."""
    full = sorted((r for r in releases if r.kind == "release"), key=lambda r: r.published, reverse=True)
    if not full:
        return []
    lines = [f"### {mod}", "", "| Version | Released | Weft |", "|---|---|---|"]
    lines += [f"| [{r.version}]({r.url}) | {r.published[:10]} | {r.weft} |" for r in full]
    return lines + [""]


def section(releases: list[Release], contains: WeftContains) -> str:
    mods: dict[str, list[Release]] = {}
    for release in sorted(releases, key=lambda r: r.mod):
        mods.setdefault(release.mod, []).append(release)
    if not mods:
        return f"{START}\n{INTRO}\nNo releases yet.\n{END}"
    lines = [START, INTRO, *latest_table(mods, contains), "", "## Versions", ""]
    for mod, mod_releases in mods.items():
        lines += history(mod, mod_releases)
    return "\n".join(lines + [END])


def replace_section(readme: str, new: str) -> str:
    start, end = readme.find(START), readme.find(END)
    if start < 0 or end < start:
        return readme.rstrip("\n") + "\n\n" + new + "\n"
    return readme[:start] + new + readme[end + len(END) :]


def main() -> None:
    path = sys.argv[1]
    releases = [parse(json.loads(line)) for line in sys.stdin if line.strip()]
    with open(path, encoding="utf-8") as file:
        readme = file.read()
    readme = replace_section(readme, section([r for r in releases if r], weft_contains))
    with open(path, "w", encoding="utf-8", newline="\n") as file:
        file.write(readme)


if __name__ == "__main__":
    main()
