"""Faults of a texture: a file that is no PNG. Its look is judged on a sheet (`tools/sheet.sh`)."""

import struct
from pathlib import Path

PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def size_of(file: Path) -> tuple[int, int]:
    """The width and height of a PNG, from its header; raises ValueError for any other file."""
    header = file.read_bytes()[:24]
    if len(header) < 24 or header[:8] != PNG_SIGNATURE or header[12:16] != b"IHDR":
        raise ValueError("no PNG")
    return struct.unpack(">II", header[16:24])


def faults(file: Path) -> list[str]:
    """What is wrong with the texture in `file`."""
    try:
        size_of(file)
    except ValueError:
        return ["is no PNG file, whatever its name says"]
    return []
