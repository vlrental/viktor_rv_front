#!/usr/bin/env python3
"""Generate deterministic responsive derivatives from committed source images."""

from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
HERO_SOURCE = ROOT / "public" / "hero-bullet-overlook.webp"
HERO_MOBILE = ROOT / "public" / "hero-bullet-overlook-768.webp"


def generate_hero_mobile() -> None:
    with Image.open(HERO_SOURCE) as source:
        image = source.convert("RGB")
        if image.size != (1535, 1024):
            raise ValueError(f"Unexpected hero dimensions: {image.size}")
        height = round(image.height * 768 / image.width)
        resized = image.resize((768, height), Image.Resampling.LANCZOS)
        resized.save(HERO_MOBILE, "WEBP", quality=78, method=6)


def main() -> None:
    generate_hero_mobile()
    print(f"Generated {HERO_MOBILE.relative_to(ROOT)} ({HERO_MOBILE.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
