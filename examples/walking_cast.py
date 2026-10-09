"""Compose the atlas from `cargo run --example walking_cast` into a labeled GIF."""
import json
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "exports"
cast = json.loads((OUT / "walking-cast.json").read_text())
atlas = Image.open(OUT / "walking-cast-atlas.png").convert("RGBA")
font_path = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"
names_font = ImageFont.truetype(font_path, 17)
title_font = ImageFont.truetype(font_path, 28)
sub_font = ImageFont.truetype(font_path, 13)
cell, margin, header = 208, 24, 88
width, height = cell * 5 + margin * 2, cell * 5 + header + margin
base = Image.new("RGB", (width, height), "#e8e1cf")
draw = ImageDraw.Draw(base)
draw.text((margin, 20), "THE SHEWSHEW STROLL", font=title_font, fill="#443522")
draw.text((margin, 57), "25 locals. One very important walk.", font=sub_font, fill="#75634b")
for i, member in enumerate(cast):
    x, y = margin + i % 5 * cell, header + i // 5 * cell
    draw.rounded_rectangle((x + 3, y + 3, x + cell - 4, y + cell - 4),
                          radius=9, fill="#f3eddc" if (i // 5 + i % 5) % 2 == 0 else "#eee6d3")
    draw.text((x + cell // 2, y + cell - 20), member["name"],
              font=names_font, anchor="mm", fill="#443522")

frames = []
for frame in range(16):
    canvas = base.copy()
    for i in range(25):
        sx, sy = i % 5 * 64, (frame * 5 + i // 5) * 64
        sprite = atlas.crop((sx, sy, sx + 64, sy + 64)).resize((192, 192), Image.Resampling.NEAREST)
        x, y = margin + i % 5 * cell + 8, header + i // 5 * cell - 7
        canvas.paste(sprite, (x, y), sprite)
    frames.append(canvas)
# One shared palette avoids colors changing between frames; never dither pixel art.
palette_source = Image.new("RGB", (width, height * 4))
for j, index in enumerate((0, 4, 8, 12)):
    palette_source.paste(frames[index], (0, j * height))
palette = palette_source.quantize(colors=256, method=Image.Quantize.MEDIANCUT)
indexed = [f.quantize(palette=palette, dither=Image.Dither.NONE) for f in frames]
durations = [60, 60, 60, 70] * 4
indexed[0].save(OUT / "shewshew-stroll.gif", save_all=True,
                append_images=indexed[1:], duration=durations, loop=0, disposal=2, optimize=False)
frames[0].save(OUT / "shewshew-stroll-preview.png")
gif = Image.open(OUT / "shewshew-stroll.gif")
assert gif.n_frames >= 4  # Identical consecutive pixel-snapped poses may merge.
assert sum(gif.seek(i) or gif.info["duration"] for i in range(gif.n_frames)) == 1000
print(f"Saved {OUT / 'shewshew-stroll.gif'} ({width}x{height}, {gif.n_frames} frames, 1-second loop)")
