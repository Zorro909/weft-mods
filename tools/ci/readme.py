"""Rewrites the mod releases section of README.md from the repository's GitHub releases.

Reads the releases from stdin, one JSON object per line with `tag`, `url`, `date`, `body` and
`weft`, the Weft commit the tag pins and its release tag (null when unknown), as
`update-readme.sh` lists them. The section between the markers is replaced, or added at the end.
Usage: `python3 tools/ci/readme.py README.md < releases`.
"""

import json
import re
import sys
from dataclasses import dataclass

START = "<!-- mod-releases:start -->"
END = "<!-- mod-releases:end -->"
# The tags of releases: `<id>-v<version>`, the version `X.Y.Z` or a candidate `X.Y.Z-rc.N`, which
# weft-mod-action makes `X.Y.Z-rc.<run>.g<commit>`. The experimental builds of the scripts before
# it, `X.Y.Z-experimental.<weft>.<mod>`, are left out.
TAG = re.compile(r"^(?P<mod>.+)-v(?P<version>\d+\.\d+\.\d+(?P<rc>-rc\.\d+(\.g[0-9a-f]{7})?)?)$")
# The line the release scripts before weft-mod-action wrote into the notes, naming the Weft of a
# release built on an untagged one.
NOTES_WEFT = re.compile(r"Built against Weft (?P<version>\S+), commit (?P<link>\[\w+\]\([^)]*/commit/\w+\))")
WEFT_COMMITS = "https://github.com/Weft-Rust/Weft/commit/"

INTRO = """\
## Downloads

Put a mod's zip as it is into the game's `mods/` folder; each one holds the Windows, Linux and
macOS builds. A mod loads only into a game with the same mod interface as the Weft it was built
against. Release candidates (`X.Y.Z-rc.…`) are builds of `main` on the way to the next release.
"""


@dataclass
class Release:
    mod: str
    version: str
    kind: str
    url: str
    # ISO 8601, so that comparing the strings compares the times.
    published: str
    # The Weft it was built against, in Markdown.
    weft: str


def parse(release: dict) -> Release | None:
    """The release, or None for a tag of another kind."""
    tag = TAG.match(release["tag"])
    if not tag:
        return None
    return Release(
        mod=tag["mod"],
        version=tag["version"],
        kind="rc" if tag["rc"] else "release",
        url=release["url"],
        published=release["date"],
        weft=weft(release["body"] or "", release.get("weft")),
    )


def weft(body: str, pinned: dict | None) -> str:
    """The Weft a release was built against, as `<version> [<commit>](<link>)`: the release tag of
    the Weft commit it pins, else the notes' version, else the commit alone."""
    link = f"[{pinned['commit'][:7]}]({WEFT_COMMITS}{pinned['commit']})" if pinned else ""
    if pinned and pinned["tag"]:
        return f"{pinned['tag'].removeprefix('v')} {link}"
    notes = NOTES_WEFT.search(body)
    if notes:
        return f"{notes['version']} {notes['link']}"
    return link or "unknown"


def newest(releases: list[Release], kind: str, after: str = "") -> Release | None:
    """The newest release of `kind` published after the time `after`."""
    found = [r for r in releases if r.kind == kind and r.published > after]
    return max(found, key=lambda r: r.published, default=None)


def cell(release: Release | None) -> str:
    if release is None:
        return "–"
    return f"[{release.version}]({release.url}), Weft {release.weft}"


def latest_table(mods: dict[str, list[Release]]) -> list[str]:
    """One row per mod: its release, and the candidate newer than that."""
    lines = ["| Mod | Release | Release candidate |", "|---|---|---|"]
    for mod, releases in mods.items():
        release = newest(releases, "release")
        candidate = newest(releases, "rc", release.published if release else "")
        lines.append(f"| {mod} | {cell(release)} | {cell(candidate)} |")
    return lines


def history(mod: str, releases: list[Release]) -> list[str]:
    """Every full release of the mod, newest first."""
    full = sorted((r for r in releases if r.kind == "release"), key=lambda r: r.published, reverse=True)
    if not full:
        return []
    lines = [f"### {mod}", "", "| Version | Released | Weft |", "|---|---|---|"]
    lines += [f"| [{r.version}]({r.url}) | {r.published[:10]} | {r.weft} |" for r in full]
    return lines + [""]


def section(releases: list[Release]) -> str:
    mods: dict[str, list[Release]] = {}
    for release in sorted(releases, key=lambda r: r.mod):
        mods.setdefault(release.mod, []).append(release)
    if not mods:
        return f"{START}\n{INTRO}\nNo releases yet.\n{END}"
    lines = [START, INTRO, *latest_table(mods), "", "## Versions", ""]
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
    readme = replace_section(readme, section([r for r in releases if r]))
    with open(path, "w", encoding="utf-8", newline="\n") as file:
        file.write(readme)


if __name__ == "__main__":
    main()
