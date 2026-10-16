#!/usr/bin/env python3
"""Generate deterministic responsive derivatives from committed source images."""

import argparse
import hashlib
import json
import tempfile
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
HERO_SOURCE = ROOT / "public" / "hero-bullet-overlook.webp"
HERO_MOBILE = ROOT / "public" / "hero-bullet-overlook-768.webp"
LOGO_SOURCE = ROOT / "public" / "logo-512.png"
LOGO_MANIFEST = ROOT / "content" / "snapshot_brand_logo.json"
RUNTIME_LOGO_ORIGINAL = ROOT / "assets" / "img" / "logo.png"
RUNTIME_LOGO_REFERENCE = ROOT / "scripts" / "assets" / "logo-runtime-120-source-6705cd58b49210b8b.png"
RUNTIME_LOGO_MANIFEST = ROOT / "content" / "runtime_brand_logo.json"


def assert_same_decoded_pixels(original: Image.Image, converted: Image.Image, label: str) -> None:
    if converted.size != original.size or converted.mode != original.mode:
        raise ValueError(f"Lossless {label} logo changed dimensions or mode")
    if original.tobytes() != converted.tobytes():
        raise ValueError(f"Lossless {label} logo changed decoded pixels")


def generate_hero_mobile() -> None:
    with Image.open(HERO_SOURCE) as source:
        image = source.convert("RGB")
        if image.size != (1535, 1024):
            raise ValueError(f"Unexpected hero dimensions: {image.size}")
        height = round(image.height * 768 / image.width)
        resized = image.resize((768, height), Image.Resampling.LANCZOS)
        resized.save(HERO_MOBILE, "WEBP", quality=78, method=6)


def encode_lossless_logo(image: Image.Image, label: str) -> bytes:
    with tempfile.TemporaryDirectory() as temporary:
        candidate = Path(temporary) / f"{label}.webp"
        image.save(candidate, "WEBP", lossless=True, exact=True, method=6)
        with Image.open(candidate) as converted:
            converted.load()
            assert_same_decoded_pixels(image, converted, label)
        return candidate.read_bytes()


def logo_derivative(source_path: Path) -> tuple[dict[str, bytes], dict[str, object]]:
    source_bytes = source_path.read_bytes()
    with Image.open(source_path) as source:
        source.load()
        if source.size != (512, 512) or source.mode != "RGB":
            raise ValueError(f"Unexpected logo image contract: {source.mode} {source.size}")
        rendered: dict[str, bytes] = {}
        variants: dict[str, dict[str, object]] = {}
        for width in (72, 144, 512):
            image = (
                source.copy()
                if width == 512
                else source.resize((width, width), Image.Resampling.LANCZOS)
            )
            derivative = encode_lossless_logo(image, f"logo-{width}")
            digest = hashlib.sha256(derivative).hexdigest()
            filename = f"logo-{width}-lossless-{digest[:16]}.webp"
            rendered[filename] = derivative
            variants[str(width)] = {
                "derivative": filename,
                "derivative_sha256": digest,
                "width": width,
                "height": width,
            }

    manifest: dict[str, object] = {
        "source": source_path.name,
        "source_sha256": hashlib.sha256(source_bytes).hexdigest(),
        "mode": "RGB",
        "variants": variants,
    }
    return rendered, manifest


def generate_brand_logo(check: bool) -> Path:
    derivatives, manifest = logo_derivative(LOGO_SOURCE)
    rendered_manifest = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    if check:
        for filename, derivative in derivatives.items():
            target = ROOT / "public" / filename
            if not target.is_file() or target.read_bytes() != derivative:
                raise ValueError(f"Snapshot brand logo derivative is stale: {filename}")
        if not LOGO_MANIFEST.is_file() or LOGO_MANIFEST.read_text(encoding="utf-8") != rendered_manifest:
            raise ValueError("Snapshot brand logo manifest is stale")
    else:
        for filename, derivative in derivatives.items():
            (ROOT / "public" / filename).write_bytes(derivative)
        LOGO_MANIFEST.write_text(rendered_manifest, encoding="utf-8")
    return ROOT / "public" / str(manifest["variants"]["512"]["derivative"])


def runtime_logo_derivative() -> tuple[str, bytes, dict[str, object]]:
    reference_bytes = RUNTIME_LOGO_REFERENCE.read_bytes()
    with Image.open(RUNTIME_LOGO_REFERENCE) as source:
        source.load()
        if source.size != (120, 120) or source.mode != "RGBA":
            raise ValueError(f"Unexpected runtime logo contract: {source.mode} {source.size}")
        derivative = encode_lossless_logo(source, "runtime-logo-120")
    digest = hashlib.sha256(derivative).hexdigest()
    filename = f"logo-runtime-120-lossless-{digest[:16]}.webp"
    manifest: dict[str, object] = {
        "original_source": str(RUNTIME_LOGO_ORIGINAL.relative_to(ROOT)),
        "original_source_sha256": hashlib.sha256(RUNTIME_LOGO_ORIGINAL.read_bytes()).hexdigest(),
        "reference": str(RUNTIME_LOGO_REFERENCE.relative_to(ROOT)),
        "reference_sha256": hashlib.sha256(reference_bytes).hexdigest(),
        "derivative": filename,
        "derivative_sha256": digest,
        "mode": "RGBA",
        "width": 120,
        "height": 120,
    }
    return filename, derivative, manifest


def generate_runtime_logo(check: bool) -> Path:
    filename, derivative, manifest = runtime_logo_derivative()
    rendered_manifest = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    target = ROOT / "public" / filename
    if check:
        if not target.is_file() or target.read_bytes() != derivative:
            raise ValueError("Runtime brand logo derivative is stale")
        if (
            not RUNTIME_LOGO_MANIFEST.is_file()
            or RUNTIME_LOGO_MANIFEST.read_text(encoding="utf-8") != rendered_manifest
        ):
            raise ValueError("Runtime brand logo manifest is stale")
    else:
        target.write_bytes(derivative)
        RUNTIME_LOGO_MANIFEST.write_text(rendered_manifest, encoding="utf-8")
    return target


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if not args.check:
        generate_hero_mobile()
    logo = generate_brand_logo(args.check)
    runtime_logo = generate_runtime_logo(args.check)
    action = "Verified" if args.check else "Generated"
    print(f"{action} {logo.relative_to(ROOT)} ({logo.stat().st_size} bytes)")
    print(
        f"{action} {runtime_logo.relative_to(ROOT)} "
        f"({runtime_logo.stat().st_size} bytes)"
    )
    if not args.check:
        print(f"Generated {HERO_MOBILE.relative_to(ROOT)} ({HERO_MOBILE.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
