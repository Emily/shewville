"""Compose the family atlas as a named, looping comparison."""
import json
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

out = Path(__file__).resolve().parents[1] / "exports"
family = json.loads((out / "jamolump-family.json").read_text())
atlas = Image.open(out / "jamolump-family-atlas.png").convert("RGBA")
font = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"
title, name, small = [ImageFont.truetype(font, n) for n in (25, 22, 13)]
base = Image.new("RGB", (840, 428), "#e8e1cf")
d = ImageDraw.Draw(base)
d.text((24, 19), "MEET JAMOLUMP", font=title, fill="#443522")
d.text((24, 54), "Jamothy + Gwendolump · one possible offspring", font=small, fill="#75634b")
for i, member in enumerate(family["members"]):
    x = 24 + i * 264
    d.rounded_rectangle((x, 86, x+256, 387), radius=10,
                       fill="#fff4d2" if i == 1 else "#f3eddc")
    d.text((x+128, 349), member["name"], anchor="mm", font=name, fill="#443522")
    d.text((x+128, 372), "OFFSPRING" if i == 1 else "PARENT", anchor="mm", font=small, fill="#75634b")
d.text((24, 402), "Inherited build shown at the same scale; baby-age proportions are not modeled yet.", font=small, fill="#75634b")
frames = []
for f in range(16):
    canvas = base.copy()
    for i in range(3):
        sprite = atlas.crop((i*64,f*64,(i+1)*64,(f+1)*64)).resize((256,256),Image.Resampling.NEAREST)
        canvas.paste(sprite,(24+i*264,86),sprite)
    frames.append(canvas)
palette = frames[0].quantize(colors=256)
indexed = [f.quantize(palette=palette,dither=Image.Dither.NONE) for f in frames]
indexed[0].save(out / "jamolump-family.gif", save_all=True, append_images=indexed[1:],
                duration=[60,60,60,70]*4,loop=0,disposal=2,optimize=False)
frames[0].save(out / "jamolump-family.png")
gif = Image.open(out / "jamolump-family.gif")
assert gif.n_frames >= 4
assert sum(gif.seek(i) or gif.info["duration"] for i in range(gif.n_frames)) == 1000
