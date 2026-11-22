#!/usr/bin/env python3
"""Renders the "heaviest objects in the universe" image for the Electron slide.

Needs Pillow:  python3 -m venv .venv && .venv/bin/pip install pillow
Run:           .venv/bin/python assets/make_bloat.py
"""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

W, H = 1600, 900
SS = 2
OUT = Path(__file__).with_name("history") / "bloat.png"

FONT_DIR = "/usr/share/fonts/truetype/dejavu"
BOLD = f"{FONT_DIR}/DejaVuSans-Bold.ttf"
MONO = f"{FONT_DIR}/DejaVuSansMono-Bold.ttf"

BASE = (30, 30, 46)  # catppuccin mocha
TEXT = (205, 214, 244)
SUBTEXT = (166, 173, 200)

# (label, radius, fill, glow, mono?)
OBJECTS = [
    ("The Sun", 45, (249, 226, 175), (250, 179, 135), False),
    ("Neutron star", 30, (180, 190, 254), (137, 180, 250), False),
    ("Black hole", 75, (17, 17, 27), (203, 166, 247), False),
    ("node_modules", 175, (243, 139, 168), (235, 160, 172), True),
]


def main():
    w, h, s = W * SS, H * SS, SS
    img = Image.new("RGBA", (w, h), BASE + (255,))

    # A sprinkle of stars
    stars = ImageDraw.Draw(img)
    for i in range(160):
        x = (i * 7919) % W
        y = (i * 104729) % H
        r = 1 + (i % 3 == 0)
        stars.ellipse(((x - r) * s, (y - r) * s, (x + r) * s, (y + r) * s),
                      fill=(88, 91, 112, 255))

    title_font = ImageFont.truetype(BOLD, 56 * s)
    label_font = ImageFont.truetype(BOLD, 34 * s)
    mono_font = ImageFont.truetype(MONO, 44 * s)

    d = ImageDraw.Draw(img)
    title = "Heaviest objects in the universe"
    tw = d.textlength(title, font=title_font)
    d.text(((w - tw) / 2, 60 * s), title, font=title_font, fill=TEXT)

    centres = [180, 450, 760, 1270]
    cy = 470
    for (label, r, fill, glow, mono), cx in zip(OBJECTS, centres):
        halo = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        g = r + 25
        ImageDraw.Draw(halo).ellipse(((cx - g) * s, (cy - g) * s, (cx + g) * s, (cy + g) * s),
                                     fill=glow + (150,))
        img = Image.alpha_composite(img, halo.filter(ImageFilter.GaussianBlur(18 * s)))
        d = ImageDraw.Draw(img)
        d.ellipse(((cx - r) * s, (cy - r) * s, (cx + r) * s, (cy + r) * s), fill=fill)

        font = mono_font if mono else label_font
        lw = d.textlength(label, font=font)
        ly = cy + max(r, 75) + 50
        d.text((cx * s - lw / 2, ly * s), label, font=font, fill=TEXT if not mono else fill)

    caption = "Your todo list app, shipped with its own copy of Chromium"
    cw = d.textlength(caption, font=label_font)
    d.text(((w - cw) / 2, 810 * s), caption, font=label_font, fill=SUBTEXT)

    OUT.parent.mkdir(exist_ok=True)
    img.convert("RGB").resize((W, H), Image.LANCZOS).save(OUT, optimize=True)
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
