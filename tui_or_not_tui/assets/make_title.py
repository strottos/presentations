#!/usr/bin/env python3
"""Renders the deliberately "proper" GUI-style title slide (title.png).

Needs Pillow:  python3 -m venv .venv && .venv/bin/pip install pillow
Run:           .venv/bin/python assets/make_title.py
"""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

W, H = 1920, 1080
SS = 2  # supersample for smooth edges
OUT = Path(__file__).with_name("title.png")

TITLE = "TUI or Not TUI"
SUBTITLE = "That is the question [y/N]"
AUTHOR = "Steven Trotter"

FONT_DIR = "/usr/share/fonts/truetype/liberation"
BOLD = f"{FONT_DIR}/LiberationSans-Bold.ttf"
REGULAR = f"{FONT_DIR}/LiberationSans-Regular.ttf"


def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def main():
    w, h = W * SS, H * SS

    # Diagonal navy -> blue gradient background
    top_left, bottom_right = (10, 22, 48), (22, 70, 140)
    grad = Image.new("RGB", (w, h))
    px = grad.load()
    for y in range(0, h, 2):
        for x in range(0, w, 2):
            c = lerp(top_left, bottom_right, (x / w * 0.6 + y / h * 0.4))
            px[x, y] = c
            if x + 1 < w:
                px[x + 1, y] = c
            if y + 1 < h:
                px[x, y + 1] = c
                if x + 1 < w:
                    px[x + 1, y + 1] = c
    img = grad.convert("RGBA")

    # Soft translucent geometric shapes on the right, corporate-keynote style
    shapes = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(shapes)
    s = SS
    d.ellipse((1250 * s, -250 * s, 2250 * s, 750 * s), fill=(80, 160, 255, 40))
    d.ellipse((1450 * s, 420 * s, 2150 * s, 1120 * s), fill=(0, 210, 200, 38))
    d.ellipse((1100 * s, 600 * s, 1500 * s, 1000 * s), fill=(140, 110, 255, 34))
    d.polygon(
        [(1500 * s, 0), (1920 * s, 0), (1920 * s, 1080 * s), (1100 * s, 1080 * s)],
        fill=(255, 255, 255, 9),
    )
    shapes = shapes.filter(ImageFilter.GaussianBlur(3 * s))
    img = Image.alpha_composite(img, shapes)

    d = ImageDraw.Draw(img)
    left = 160 * s

    # Accent bar
    d.rounded_rectangle(
        (left, 380 * s, left + 120 * s, 392 * s), radius=6 * s, fill=(0, 210, 200, 255)
    )

    title_font = ImageFont.truetype(BOLD, 150 * s)
    sub_font = ImageFont.truetype(REGULAR, 54 * s)
    author_font = ImageFont.truetype(BOLD, 40 * s)
    small_font = ImageFont.truetype(REGULAR, 30 * s)

    # Title with a subtle drop shadow
    shadow = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    ImageDraw.Draw(shadow).text(
        (left + 4 * s, 430 * s + 6 * s), TITLE, font=title_font, fill=(0, 0, 0, 120)
    )
    img = Image.alpha_composite(img, shadow.filter(ImageFilter.GaussianBlur(8 * s)))
    d = ImageDraw.Draw(img)
    d.text((left, 430 * s), TITLE, font=title_font, fill=(255, 255, 255, 255))

    d.text((left, 615 * s), SUBTITLE, font=sub_font, fill=(190, 215, 245, 255))

    # Author block
    d.line((left, 820 * s, left + 520 * s, 820 * s), fill=(80, 110, 160, 255), width=2 * s)
    d.text((left, 850 * s), AUTHOR, font=author_font, fill=(255, 255, 255, 255))

    # Footer, the hallmark of every Serious Business Presentation
    d.text((left, 1010 * s), "CONFIDENTIAL  •  FOR INTERNAL USE ONLY",
           font=small_font, fill=(110, 135, 180, 255))
    d.text((1760 * s, 1010 * s), "1", font=small_font, fill=(110, 135, 180, 255))

    img.convert("RGB").resize((W, H), Image.LANCZOS).save(OUT, optimize=True)
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
