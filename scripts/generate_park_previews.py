#!/usr/bin/env python3
"""Generate responsive derivatives for explicitly approved park hero sources."""

from __future__ import annotations

from io import BytesIO
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageOps


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "public" / "park-previews"
MANIFEST = ROOT / "content" / "park_detail_previews.json"
SOURCES = {
    "bear-creek": ROOT / "assets" / "img" / "park-bear-creek.webp",
}


def encode(photo: Image.Image, width: int) -> bytes:
    resized = photo.copy()
    resized.thumbnail((width, width), Image.Resampling.LANCZOS)
    output = BytesIO()
    resized.save(output, "WEBP", quality=78, method=6)
    return output.getvalue()


def main() -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    manifest: dict[str, dict[str, object]] = {}
    for slug, source in SOURCES.items():
        photo = ImageOps.exif_transpose(Image.open(source)).convert("RGB")
        variants = []
        for width in (800, 1161):
            encoded = encode(photo, width)
            digest = hashlib.sha256(encoded).hexdigest()
            filename = f"park-{slug}-{width}-{digest[:16]}.webp"
            (OUTPUT / filename).write_bytes(encoded)
            variants.append({"filename": filename, "width": width, "sha256": digest})
        manifest[slug] = {
            "source": str(source.relative_to(ROOT)),
            "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
            "source_width": photo.width,
            "source_height": photo.height,
            "variants": variants,
        }
    MANIFEST.write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(f"Generated responsive park previews for {len(manifest)} route(s)")


if __name__ == "__main__":
    main()
