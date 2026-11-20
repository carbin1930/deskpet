"""Generate deskpet app icons (a simple paw print). Run once; output committed to repo."""
from PIL import Image, ImageDraw
import os

OUT = os.path.dirname(os.path.abspath(__file__))
BG = (30, 41, 59)      # slate-800
PAD = (129, 140, 248)  # indigo-400


def paw(size: int) -> Image.Image:
    s = 512
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    # rounded background
    d.rounded_rectangle([16, 16, s - 16, s - 16], radius=110, fill=BG + (255,))

    # main pad
    d.ellipse([150, 250, 362, 420], fill=PAD + (255,))

    # toes
    for cx in (156, 256, 356):
        d.ellipse([cx - 42, 150, cx + 42, 268], fill=PAD + (255,))
    # side toe
    d.ellipse([92, 250, 168, 340], fill=PAD + (255,))

    return img.resize((size, size), Image.LANCZOS)


paw(512).save(os.path.join(OUT, "512x512.png"))
paw(128).save(os.path.join(OUT, "128x128.png"))
paw(32).save(os.path.join(OUT, "32x32.png"))
paw(256).save(os.path.join(OUT, "icon.ico"), sizes=[(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (16, 16)])

try:
    paw(512).save(os.path.join(OUT, "icon.icns"))
    print("icns ok")
except Exception as e:
    print("icns skipped:", e)

print("icons generated")
