"""Faults of an entity model, Blockbench's "Bedrock Entity" `<name>.geo.json`
(`Weft/docs/models.md`, "Model from Blockbench").
"""

import json
import math
from pathlib import Path

from textures import size_of


def faults(file: Path, assets: Path) -> list[str]:
    """What is wrong with the model in `file`, a file of the mod whose `assets` these are."""
    try:
        model = json.loads(file.read_text())
    except ValueError as error:
        return [f"not JSON: {error}"]
    geometries = model.get("minecraft:geometry", [])
    if len(geometries) != 1:
        return [f"holds {len(geometries)} geometries; the game reads a file with one"]
    geometry = geometries[0]
    skin = skin_size(geometry)
    found = bone_order_faults(geometry.get("bones", []))
    for bone in geometry.get("bones", []):
        for index, cube in enumerate(bone.get("cubes", [])):
            name = f'cube {index} of bone "{bone.get("name")}"'
            found += cube_faults(name, cube, skin)
    return found + own_texture_faults(file, assets, skin)


def skin_size(geometry: dict) -> tuple[int, int]:
    """The width and height of the texture the model's uvs are laid out on."""
    description = geometry.get("description", {})
    return description.get("texture_width", 64), description.get("texture_height", 32)


def bone_order_faults(bones: list[dict]) -> list[str]:
    """Bones whose parent is missing or comes after them: the game reads parents first."""
    found, seen = [], set()
    for bone in bones:
        parent = bone.get("parent")
        if parent is not None and parent not in seen:
            found.append(f'bone "{bone.get("name")}" comes before its parent "{parent}", or has none')
        seen.add(bone.get("name"))
    return found


def cube_faults(name: str, cube: dict, skin: tuple[int, int]) -> list[str]:
    """What is wrong with one cube: a turn the game leaves out, an unwrap past the texture."""
    found = []
    if any(cube.get("rotation", [])):
        found.append(f"{name} is turned, and the game draws it unturned: turn a bone of its own")
    uv = cube.get("uv")
    if isinstance(uv, list):
        width, height, depth = (math.ceil(side) for side in cube.get("size", [0, 0, 0]))
        right, bottom = uv[0] + 2 * (depth + width), uv[1] + depth + height
        if right > skin[0] or bottom > skin[1]:
            found.append(
                f"{name} unwraps to {right} x {bottom}, past the texture's {skin[0]} x {skin[1]}"
            )
    return found


def own_texture_faults(file: Path, assets: Path, skin: tuple[int, int]) -> list[str]:
    """Whether the texture of the model's own name has the shape its uvs are laid out on."""
    name = file.name.removesuffix(".geo.json")
    texture = assets / "textures" / "entity" / f"{name}.png"
    if not texture.is_file():
        return []
    width, height = size_of(texture)
    if width * skin[1] != height * skin[0] or width % skin[0] != 0:
        return [
            f"textures/entity/{name}.png is {width} x {height}, "
            f"no whole multiple of the model's {skin[0]} x {skin[1]}"
        ]
    return []
