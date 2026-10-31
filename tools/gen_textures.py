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
    # Torch: a stick with a red head, glowing when lit. The box shows the middle of the texture.
    for name, head in (("torch_on", (255, 60, 30)), ("torch_off", (110, 25, 20))):
        t = noisy((105, 75, 45), 0.12, 700)
        for y in range(0, 5):
            for x in range(16):
                t[y][x] = tuple(max(0, min(255, int(c * (0.85 + 0.03 * ((x * 7 + y * 3) % 5))))) for c in head) + (255,)
        png(block / f"{name}.png", t)
    png(block / "torch_on_e.png", [[(230 if y < 5 else 0, 0, 0, 255) for _ in range(16)] for y in range(16)])
    item = [[(0, 0, 0, 0)] * 16 for _ in range(16)]
    for y in range(3, 15):
        for x in (7, 8):
            item[y][x] = (255, 60, 30, 255) if y < 6 else (105, 75, 45, 255)
    png(OUT / "item" / "torch.png", item)
    # Repeater and comparator: a stone slab, the posts red (bright when on). Boxes show the part
    # of the texture under them, so the posts take the texture's red middle band.
    for name, seed in (("repeater", 800), ("comparator", 810)):
        for state, red in (("on", (255, 50, 25)), ("off", (110, 25, 20))):
            t = noisy((150, 150, 150), 0.08, seed)
            for y in range(16):
                for x in range(16):
                    if 5 <= x <= 12 and 5 <= y <= 12:
                        t[y][x] = tuple(max(0, min(255, int(c * (0.9 + 0.05 * ((x + y) % 3))))) for c in red) + (255,)
            png(block / f"{name}_{state}.png", t)
            if state == "on":
                png(block / f"{name}_on_e.png", [[(220 if 5 <= x <= 12 and 5 <= y <= 12 else 0, 0, 0, 255) for x in range(16)] for y in range(16)])
        item = [[(0, 0, 0, 0)] * 16 for _ in range(16)]
        for y in range(10, 14):
            for x in range(1, 15):
                item[y][x] = (150, 150, 150, 255)
        posts = (4, 11) if name == "repeater" else (3, 8, 12)
        for px in posts:
            for y in range(5, 10):
                for x in (px, px + 1):
                    item[y][x] = (230, 45, 25, 255)
        png(OUT / "item" / f"{name}.png", item)
    png(block / "button_stone.png", noisy((140, 140, 140), 0.10, 900))
    png(block / "button_wood.png", noisy((160, 120, 70), 0.12, 901))
    png(block / "pressure_plate.png", noisy((135, 135, 135), 0.06, 902))
    # Pistons: stone with a wooden band on the sides, a wooden front (green when sticky), the
    # base's front with a hole for the rod while the head is out.
    side = noisy((125, 125, 125), 0.10, 1000)
    for y in range(0, 4):
        for x in range(16):
            side[y][x] = tuple(max(0, min(255, int(c * (0.9 + 0.05 * (x % 3))))) for c in (165, 125, 75)) + (255,)
    png(block / "piston_side.png", side)
    front = noisy((170, 130, 80), 0.10, 1001)
    for y in (3, 7, 11, 15):
        for x in range(16):
            front[y][x] = (120, 90, 55, 255)
    png(block / "piston_front.png", front)
    sticky = [row[:] for row in front]
    for y in range(3, 13):
        for x in range(3, 13):
            sticky[y][x] = (90, 170 + (x * 5 + y * 3) % 30, 70, 255)
    png(block / "piston_sticky.png", sticky)
    back = noisy((110, 110, 110), 0.10, 1002)
    for y in range(6, 10):
        for x in range(6, 10):
            back[y][x] = (70, 70, 70, 255)
    png(block / "piston_back.png", back)
    inner = noisy((125, 125, 125), 0.10, 1003)
    for y in range(6, 10):
        for x in range(6, 10):
            inner[y][x] = (165, 125, 75, 255)
    png(block / "piston_inner.png", inner)
    # Dust's item: a few specks on transparent.
    rng = random.Random(600)
    item = [[(0, 0, 0, 0)] * 16 for _ in range(16)]
    for _ in range(40):
        x, y = rng.randrange(3, 13), rng.randrange(3, 13)
        item[y][x] = (200 + rng.randrange(55), 20, 15, 255)
    png(OUT / "item" / "dust.png", item)


if __name__ == "__main__":
    main()
