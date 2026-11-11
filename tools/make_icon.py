#!/usr/bin/env python3
"""Génère assets/icon.png / icon.ico / icon.icns : la photo dans un cercle + un badge minuteur.

Usage : python3 tools/make_icon.py photo.png
"""
import sys
from PIL import Image, ImageDraw

S = 1024
SS = 4  # suréchantillonnage


def main(photo_path):
    n = S * SS
    canvas = Image.new("RGBA", (n, n), (0, 0, 0, 0))
    d = ImageDraw.Draw(canvas)

    # Fond : carré arrondi crème apaisant
    d.rounded_rectangle([0, 0, n - 1, n - 1], radius=int(n * 0.22), fill=(246, 238, 226, 255))

    # Photo recadrée sur le visage, en cercle
    photo = Image.open(photo_path).convert("RGB")
    w, h = photo.size
    box = (int(w * 0.06), int(h * 0.02), int(w * 0.06 + w * 0.80), int(h * 0.02 + w * 0.80))
    photo = photo.crop(box)
    diam = int(n * 0.76)
    photo = photo.resize((diam, diam), Image.LANCZOS)
    mask = Image.new("L", (diam, diam), 0)
    ImageDraw.Draw(mask).ellipse([0, 0, diam - 1, diam - 1], fill=255)
    cx = int(n * 0.44)
    cy = int(n * 0.44)
    x0, y0 = cx - diam // 2, cy - diam // 2
    ring = int(n * 0.014)
    d.ellipse([x0 - ring, y0 - ring, x0 + diam + ring, y0 + diam + ring], fill=(255, 255, 255, 255))
    canvas.paste(photo, (x0, y0), mask)

    # Badge minuteur en bas à droite
    bd = int(n * 0.42)
    bx, by = int(n * 0.97) - bd, int(n * 0.97) - bd
    bc = (bx + bd // 2, by + bd // 2)
    d.ellipse([bx - ring, by - ring, bx + bd + ring, by + bd + ring], fill=(255, 255, 255, 255))
    d.ellipse([bx, by, bx + bd, by + bd], fill=(232, 112, 96, 255))  # corail doux
    inner = int(bd * 0.12)
    d.ellipse([bx + inner, by + inner, bx + bd - inner, by + bd - inner], fill=(255, 250, 244, 255))
    # secteur écoulé (3/4 de tour restant = arc corail clair)
    pad = int(bd * 0.22)
    d.pieslice([bx + pad, by + pad, bx + bd - pad, by + bd - pad], start=-90, end=0, fill=(250, 205, 196, 255))
    # aiguilles
    lw = int(bd * 0.055)
    r = bd // 2 - pad
    d.line([bc, (bc[0], bc[1] - r)], fill=(120, 70, 60, 255), width=lw)
    d.line([bc, (bc[0] + int(r * 0.72), bc[1])], fill=(120, 70, 60, 255), width=lw)
    d.ellipse([bc[0] - lw, bc[1] - lw, bc[0] + lw, bc[1] + lw], fill=(120, 70, 60, 255))
    # petit bouton du chrono
    kw = int(bd * 0.16)
    d.rounded_rectangle(
        [bc[0] - kw // 2, by - int(bd * 0.1), bc[0] + kw // 2, by + int(bd * 0.02)],
        radius=kw // 4, fill=(232, 112, 96, 255),
    )

    icon = canvas.resize((S, S), Image.LANCZOS)
    icon.resize((256, 256), Image.LANCZOS).save("assets/icon.png")
    icon.save("assets/icon.ico", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    icon.save("assets/icon.icns")
    print("ok")


if __name__ == "__main__":
    main(sys.argv[1])
