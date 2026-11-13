"""Faults of a block or item model, Blockbench's "Java Block/Item" (`Weft/docs/models.md`).

Coordinates are texels, 16 to a block. The engine's mesher draws every corner of a block's box
at a whole texel (`quad_pos` in `Weft/crates/engine/src/gpu_layout.rs`), so a box that ends
between two lands on one of them, and two faces looking the same way in one plane flicker. An
item's model is drawn in the hand, as it is written.
"""

import json
from dataclasses import dataclass
from itertools import combinations
from pathlib import Path

# For each side a face looks to: the axis it looks along, which end of the box it is on (0 its
# `from`, 1 its `to`), and the two axes its rectangle spans.
SIDES = {
    "west": (0, 0, (2, 1)),
    "east": (0, 1, (2, 1)),
    "down": (1, 0, (0, 2)),
    "up": (1, 1, (0, 2)),
    "north": (2, 0, (0, 1)),
    "south": (2, 1, (0, 1)),
}
# How far a texture's `uv` may reach.
UV_RANGE = (0, 16)


@dataclass
class Face:
    """One drawn face of a box: where its plane lies and the rectangle it covers there."""

    box: str
    side: str
    plane: float
    low: tuple[float, float]
    high: tuple[float, float]

    def overlaps(self, other: "Face") -> bool:
        """Whether both look the same way in one plane and share some of it."""
        same_plane = self.side == other.side and self.plane == other.plane
        shared = all(
            max(self.low[i], other.low[i]) < min(self.high[i], other.high[i]) for i in (0, 1)
        )
        return same_plane and shared


def faults(file: Path, assets: Path) -> list[str]:
    """What is wrong with the model in `file`, a file of the mod whose `assets` these are."""
    try:
        model = json.loads(file.read_text())
    except ValueError as error:
        return [f"not JSON: {error}"]
    boxes = model.get("elements", [])
    found = missing_textures(model, assets)
    for index, box in enumerate(boxes):
        found += box_faults(name_of(box, index), box)
    if "block" not in file.relative_to(assets).parts:
        return found
    for index, box in enumerate(boxes):
        found += between_texels(name_of(box, index), box)
    drawn = [face for index, box in enumerate(boxes) for face in faces_of(name_of(box, index), box)]
    for one, other in combinations(drawn, 2):
        if one.box != other.box and one.overlaps(other):
            found.append(
                f"the {one.side} faces of {one.box} and {other.box} share the plane at "
                f"{one.plane:g}: they flicker; move one a texel or leave its face out"
            )
    return found


def name_of(box: dict, index: int) -> str:
    return f'box "{box["name"]}"' if box.get("name") else f"box {index}"


def missing_textures(model: dict, assets: Path) -> list[str]:
    """The textures the model names as files of this mod that are not there."""
    found = []
    for key, texture in model.get("textures", {}).items():
        of_this_mod = not texture.startswith("#") and ":" not in texture
        if of_this_mod and not (assets / "textures" / f"{texture}.png").is_file():
            found.append(f'texture "{key}" names {texture}, and there is no textures/{texture}.png')
    return found


def box_faults(name: str, box: dict) -> list[str]:
    """What is wrong with one box by itself."""
    start, end = box.get("from", [0, 0, 0]), box.get("to", [0, 0, 0])
    found = []
    if any(low > high for low, high in zip(start, end)):
        found.append(f"{name} goes from {start} back to {end}")
    for side, face in box.get("faces", {}).items():
        outside = [value for value in face.get("uv", []) if not UV_RANGE[0] <= value <= UV_RANGE[1]]
        if outside:
            found.append(f"{name}: the uv of its {side} face leaves the texture ({outside})")
    return found


def between_texels(name: str, box: dict) -> list[str]:
    """The fault of a block's box with a corner the mesher would move."""
    between = [value for value in box.get("from", []) + box.get("to", []) if value != round(value)]
    if not between or is_turned(box):
        return []
    return [f"{name} has corners between texels ({between}): the game draws them at whole ones"]


def is_turned(box: dict) -> bool:
    return box.get("rotation", {}).get("angle", 0) != 0


def faces_of(name: str, box: dict) -> list[Face]:
    """The faces of an unturned box that the model draws."""
    if is_turned(box) or "from" not in box or "to" not in box:
        return []
    ends = (box["from"], box["to"])
    drawn = []
    for side in box.get("faces", {}):
        if side not in SIDES:
            continue
        axis, end, (across, down) = SIDES[side]
        low = (ends[0][across], ends[0][down])
        high = (ends[1][across], ends[1][down])
        drawn.append(Face(name, side, ends[end][axis], low, high))
    return drawn
