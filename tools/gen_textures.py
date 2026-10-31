#!/usr/bin/env python3
"""Draws the redstone mod's placeholder textures (16x16 PNGs) with no dependencies.

    python3 tools/gen_textures.py

Writes into mods/redstone/assets/textures/. Deterministic: the same files every run.
"""

import random
import struct
import zlib
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "mods" / "redstone" / "assets" / "textures"


def png(path, pixels):
    """pixels: 16 rows of 16 (r, g, b, a) tuples."""
    raw = b"".join(b"\x00" + bytes(c for px in row for c in px) for row in pixels)

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

    data = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 16, 16, 8, 6, 0, 0, 0))
    data += chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def noisy(base, spread, seed):
    rng = random.Random(seed)
    rows = []
    for _ in range(16):
        row = []
        for _ in range(16):
            f = 1.0 + rng.uniform(-spread, spread)
            row.append(tuple(max(0, min(255, int(c * f))) for c in base) + (255,))
        rows.append(row)
    return rows


def glow(amount, seed, density=1.0):
    """An emission mask (`_e.png`): red channel = how much a pixel glows."""
    rng = random.Random(seed)
    return [[(int(amount * (0.7 + 0.3 * rng.random())) if rng.random() < density else 0, 0, 0, 255) for _ in range(16)] for _ in range(16)]


def main():
    block = OUT / "block"
    # Dust, one texture per brightness level (spec §8: 4 levels). The boxes give the shape, every
    # face shows the part of the texture under it, so the texture itself is uniform.
    dust = [(70, 8, 8), (130, 10, 6), (190, 20, 10), (255, 40, 20)]
    for level, color in enumerate(dust):
        png(block / f"dust_{level}.png", noisy(color, 0.18, 100 + level))
        if level > 0:
            png(block / f"dust_{level}_e.png", glow(85 * level, 200 + level, 0.8))
    # Lever: base and handle share the state's textures (SDK boxes can't differ per box, G9).
    png(block / "lever.png", noisy((120, 110, 100), 0.12, 300))
    # Lamp.
    png(block / "lamp_off.png", noisy((95, 60, 35), 0.15, 400))
    on = noisy((250, 200, 120), 0.10, 401)
    for i in range(16):
        on[0][i] = on[15][i] = on[i][0] = on[i][15] = (120, 80, 45, 255)
    png(block / "lamp_on.png", on)
    png(block / "lamp_on_e.png", [[(0 if i in (0, 15) or j in (0, 15) else 255, 0, 0, 255) for i in range(16)] for j in range(16)])
    # Power block: red with a lighter cross-hatch.
    pb = noisy((175, 25, 20), 0.12, 500)
    for i in range(16):
        for j in range(16):
            if (i + j) % 5 == 0 or (i - j) % 5 == 0:
                pb[j][i] = (225, 70, 55, 255)
    png(block / "power_block.png", pb)
    # Dust's item: a few specks on transparent.
    rng = random.Random(600)
    item = [[(0, 0, 0, 0)] * 16 for _ in range(16)]
    for _ in range(40):
        x, y = rng.randrange(3, 13), rng.randrange(3, 13)
        item[y][x] = (200 + rng.randrange(55), 20, 15, 255)
    png(OUT / "item" / "dust.png", item)


if __name__ == "__main__":
    main()
