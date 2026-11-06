"""Writes `<id>.mod.json`, the machine-readable record `publish.sh` uploads next to each `<id>.zip`.

Weft Launcher reads it for a mod from a GitHub source, and the mod index (`Zorro909/weft-mods-index`)
for its reviewed entries. It holds the mod's name, summary, authors and icon from its language file,
and the release: version, channel, tag, the source and Weft commits, the mod ABI, deps, platforms
and the zip's size and sha256. The format is the launcher's `INDEX_FORMAT.md`, "`<id>.mod.json`".

Run from the repository root, after `pack` and with the Weft checkout the zip was built against.
Needs `gh` and `GH_TOKEN`, to name the Weft commit after its nearest release tag.
Usage: `python3 tools/ci/mod_json.py FOLDER TAG KIND ZIP > <id>.mod.json`.
"""

import datetime
import hashlib
import json
import os
import re
import subprocess
import sys

WEFT_REPO = "Weft-Rust/Weft"
ABI_FILE = "Weft/crates/abi/src/lib.rs"
# What `pack` in `publish.sh` puts into every zip.
PLATFORMS = ["windows-x64", "linux-x64", "macos-arm64"]
CHANNELS = {"release": "release", "rc": "prerelease", "experimental": "experimental"}
SUMMARY_MAX = 140


def abi_hash(source: bytes) -> str:
    """The mod ABI: FNV-1a-64 of the bytes of `crates/abi/src/lib.rs`, as 16 hex characters."""
    h = 0xCBF29CE484222325
    for byte in source:
        h = ((h ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{h:016x}"


def parse_lang(text: str) -> dict[str, str]:
    pairs = (line.split("=", 1) for line in text.splitlines() if "=" in line and not line.startswith("#"))
    return {key.strip(): value.strip() for key, value in pairs}


def summarize(description: str) -> str:
    """The description's first sentence on one line, without color markup, at most 140 characters:
    cut at a comma, or else a word, when it is longer."""
    text = re.sub(r"\{[^{}]*\}", "", description)
    text = re.split(r"\\n|\n", text)[0]
    text = " ".join(re.split(r"(?<=[.!?])\s+", text.strip())[0].split())
    if len(text) <= SUMMARY_MAX:
        return text
    cut = text[: SUMMARY_MAX - 1]
    comma = cut.rfind(", ")
    end = comma if comma >= SUMMARY_MAX // 2 else cut.rfind(" ")
    return cut[:end].rstrip(" ,;:") + "…"


def parse_deps(lib_rs: str) -> list[str]:
    """The deps of `declare_mod!` as a release builds it: without features."""
    pattern = r"(?:#\[cfg\((?P<cfg>[^\]]*)\)\]\s*)?declare_mod!\s*\(\s*deps\s*:\s*\[(?P<deps>[^\]]*)\]"
    for match in re.finditer(pattern, lib_rs):
        cfg = (match.group("cfg") or "").replace(" ", "")
        if not cfg or cfg.startswith("not(feature"):
            return re.findall(r'"([a-z0-9_]+)"', match.group("deps"))
    raise ValueError("no declare_mod!(deps: [..]) without a feature in src/lib.rs")


def weft_tag(commit: str, tags: dict[str, str], ahead_of) -> str:
    """The Weft release tag at `commit`, or else `git describe --tags`' `<tag>-<n>-g<sha7>`, for the
    nearest tag `commit` descends from. `ahead_of(tag, commit)` is the number of commits since the
    tag, or None when the tag is no ancestor."""
    for tag, tag_commit in tags.items():
        if tag_commit == commit:
            return tag
    distances = {tag: ahead_of(tag, commit) for tag in tags}
    known = {tag: n for tag, n in distances.items() if n is not None}
    if not known:
        raise ValueError(f"Weft {commit} descends from no release tag")
    nearest = min(known, key=known.get)
    return f"{nearest}-{known[nearest]}-g{commit[:7]}"


def gh_api(path: str):
    out = subprocess.run(["gh", "api", path], capture_output=True, text=True, check=True)
    return json.loads(out.stdout)


def github_weft_tag(commit: str) -> str:
    tags = {t["name"]: t["commit"]["sha"] for t in gh_api(f"repos/{WEFT_REPO}/tags?per_page=100")
            if re.fullmatch(r"v[0-9A-Za-z.\-]+", t["name"])}

    def ahead_of(tag: str, head: str) -> int | None:
        compare = gh_api(f"repos/{WEFT_REPO}/compare/{tag}...{head}")
        return compare["ahead_by"] if compare["status"] in ("ahead", "identical") else None

    return weft_tag(commit, tags, ahead_of)


def git(*args: str) -> str:
    return subprocess.run(["git", *args], capture_output=True, text=True, check=True).stdout.strip()


def source_repo() -> str:
    if os.environ.get("GITHUB_REPOSITORY"):
        return os.environ["GITHUB_REPOSITORY"]
    url = git("remote", "get-url", "origin")
    return re.sub(r"^(?:https://github\.com/|git@github\.com:)|\.git$", "", url)


def mod_json(folder: str, mod_id: str, tag: str, kind: str, zip_bytes: bytes, repo: str,
             commit: str, weft: dict, published: str) -> dict:
    with open(f"{folder}/assets/lang/en_us.lang", encoding="utf-8") as f:
        lang = parse_lang(f.read())
    with open(f"{folder}/src/lib.rs", encoding="utf-8") as f:
        deps = parse_deps(f.read())
    authors = [a.strip() for a in lang.get(f"mod.{mod_id}.authors", "").split(",") if a.strip()]
    name = lang.get(f"mod.{mod_id}.name", mod_id)
    icon = f"{folder}/assets/textures/gui/icon.png"
    return {
        "schema": 1,
        "id": mod_id,
        "name": name,
        "summary": summarize(lang.get(f"mod.{mod_id}.description", "")) or name,
        "authors": authors or [repo.split("/")[0]],
        "license": None,
        "homepage": f"https://github.com/{repo}/tree/main/{folder}",
        "icon": f"https://raw.githubusercontent.com/{repo}/{commit}/{icon}" if os.path.isfile(icon) else None,
        "version": tag.removeprefix(f"{mod_id}-v"),
        "channel": CHANNELS[kind],
        "tag": tag,
        "published": published,
        "source": {"repo": repo, "commit": commit},
        "weft": weft,
        "deps": [{"id": dep} for dep in deps],
        "platforms": PLATFORMS,
        "asset": {
            "name": f"{mod_id}.zip",
            "url": f"https://github.com/{repo}/releases/download/{tag}/{mod_id}.zip",
            "size": len(zip_bytes),
            "sha256": hashlib.sha256(zip_bytes).hexdigest(),
        },
    }


def main() -> None:
    folder, tag, kind, zip_path = sys.argv[1:5]
    mod_id = os.path.basename(zip_path).removesuffix(".zip")
    with open(zip_path, "rb") as f:
        zip_bytes = f.read()
    with open(ABI_FILE, "rb") as f:
        abi = abi_hash(f.read())
    weft_commit = git("-C", "Weft", "rev-parse", "HEAD")
    weft = {"tag": github_weft_tag(weft_commit), "commit": weft_commit, "abi": abi}
    published = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    record = mod_json(folder, mod_id, tag, kind, zip_bytes, source_repo(), git("rev-parse", "HEAD"),
                      weft, published)
    json.dump(record, sys.stdout, indent=2, ensure_ascii=False)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
