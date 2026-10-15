#!/usr/bin/env python3
"""Generate deterministic responsive derivatives from committed source images."""

import argparse
import hashlib
import json
import tempfile
from pathlib import Path

from PIL import Image, ImageChops


ROOT = Path(__file__).resolve().parents[1]
HERO_SOURCE = ROOT / "public" / "hero-bullet-overlook.webp"
HERO_MOBILE = ROOT / "public" / "hero-bullet-overlook-768.webp"
LOGO_SOURCE = ROOT / "public" / "logo-512.png"
LOGO_MANIFEST = ROOT / "content" / "snapshot_brand_logo.json"


def generate_hero_mobile() -> None:
    with Image.open(HERO_SOURCE) as source:
        image = source.convert("RGB")
        if image.size != (1535, 1024):
            raise ValueError(f"Unexpected hero dimensions: {image.size}")
        height = round(image.height * 768 / image.width)
        resized = image.resize((768, height), Image.Resampling.LANCZOS)
        resized.save(HERO_MOBILE, "WEBP", quality=78, method=6)


def logo_derivative(source_path: Path) -> tuple[bytes, dict[str, object]]:
    source_bytes = source_path.read_bytes()
    with tempfile.TemporaryDirectory() as temporary:
        candidate = Path(temporary) / "logo.webp"
        with Image.open(source_path) as source:
            source.load()
            if source.size != (512, 512) or source.mode != "RGB":
                raise ValueError(f"Unexpected logo image contract: {source.mode} {source.size}")
            source.save(candidate, "WEBP", lossless=True, exact=True, method=6)
            with Image.open(candidate) as converted:
                converted.load()
                if converted.size != source.size or converted.mode != "RGB":
                    raise ValueError("Lossless logo derivative changed dimensions or mode")
                if ImageChops.difference(source, converted).getbbox() is not None:
                    raise ValueError("Lossless logo derivative changed decoded pixels")
        derivative = candidate.read_bytes()

    digest = hashlib.sha256(derivative).hexdigest()
    filename = f"logo-512-lossless-{digest[:16]}.webp"
    manifest: dict[str, object] = {
        "source": source_path.name,
        "source_sha256": hashlib.sha256(source_bytes).hexdigest(),
        "derivative": filename,
        "derivative_sha256": digest,
        "width": 512,
        "height": 512,
        "mode": "RGB",
    }
    return derivative, manifest


def generate_brand_logo(check: bool) -> Path:
    derivative, manifest = logo_derivative(LOGO_SOURCE)
    target = ROOT / "public" / str(manifest["derivative"])
    rendered_manifest = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    if check:
        if not target.is_file() or target.read_bytes() != derivative:
            raise ValueError("Snapshot brand logo derivative is stale")
        if not LOGO_MANIFEST.is_file() or LOGO_MANIFEST.read_text(encoding="utf-8") != rendered_manifest:
            raise ValueError("Snapshot brand logo manifest is stale")
    else:
        target.write_bytes(derivative)
        LOGO_MANIFEST.write_text(rendered_manifest, encoding="utf-8")
    return target


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if not args.check:
        generate_hero_mobile()
    logo = generate_brand_logo(args.check)
    action = "Verified" if args.check else "Generated"
    print(f"{action} {logo.relative_to(ROOT)} ({logo.stat().st_size} bytes)")
    if not args.check:
        print(f"Generated {HERO_MOBILE.relative_to(ROOT)} ({HERO_MOBILE.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
