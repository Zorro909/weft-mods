"""Checks a mod's models and textures for the faults that otherwise show only in the game.

Reads the files under `<mod>/assets/` and prints each fault as `<file>: <what is wrong>`; ends
with 1 when there is any. It takes a second and needs neither the game nor Blockbench, so it
runs first: `tools/check-mods.sh` runs it for every mod.
Usage: `python3 tools/assets/lint.py <mod folder>...`.
"""

import sys
from pathlib import Path

import block_models
import entity_models
import textures


def faults_of(mod: Path) -> list[str]:
    """Every fault in the mod's assets, each naming its file from the mod's folder."""
    assets = mod / "assets"
    found = []
    for model in sorted((assets / "models").rglob("*.json")):
        check = entity_models.faults if model.name.endswith(".geo.json") else block_models.faults
        found += [f"{model}: {fault}" for fault in check(model, assets)]
    for picture in sorted((assets / "textures").rglob("*.png")):
        found += [f"{picture}: {fault}" for fault in textures.faults(picture)]
    return found


def main(folders: list[str]) -> int:
    if not folders:
        print("lint.py: name at least one mod folder", file=sys.stderr)
        return 2
    found = [fault for folder in folders for fault in faults_of(Path(folder))]
    for fault in found:
        print(fault)
    print(f"{len(found)} faults in the assets of: {' '.join(folders)}")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
