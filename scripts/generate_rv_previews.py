#!/usr/bin/env python3
"""Generate exact WebP previews of the current public RV photos.

The source manifest is intentionally explicit. When admin media changes, refresh
the manifest and generated assets together; unknown URLs use their original.
"""

from concurrent.futures import ThreadPoolExecutor
from io import BytesIO
import argparse
import json
from pathlib import Path
import subprocess
from urllib.parse import urlparse

from PIL import Image, ImageOps

ROOT = Path(__file__).resolve().parents[1]
SOURCES = ROOT / "scripts" / "rv_preview_sources.json"
OUTPUT = ROOT / "public" / "rv-previews"
CARD_MANIFEST = ROOT / "content" / "rv_card_previews.json"
PROJECT_HOST = "pwhlkpwlansarstmstge.supabase.co"
PUBLIC_PREFIX = "/storage/v1/object/public/rental-media/rentals/"
MAX_SOURCE_BYTES = 12 * 1024 * 1024


def encode_derivative(photo: Image.Image, width: int) -> bytes:
    resized = photo.copy()
    resized.thumbnail((width, width), Image.Resampling.LANCZOS)
    output = BytesIO()
    resized.save(output, "WEBP", quality=78, method=6)
    return output.getvalue()


def generate(entry: dict[str, str], cards_only: bool = False) -> tuple[int, int]:
    url = entry["url"]
    parsed = urlparse(url)
    if (
        parsed.scheme != "https"
        or parsed.hostname != PROJECT_HOST
        or not parsed.path.startswith(PUBLIC_PREFIX)
        or parsed.query
        or parsed.fragment
    ):
        raise ValueError(f"Unexpected media URL: {url}")
    image_id = Path(parsed.path).stem
    process = subprocess.Popen(
        ["curl", "--fail", "--silent", "--show-error", "--max-time", "35", url],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    assert process.stdout is not None
    raw = process.stdout.read(MAX_SOURCE_BYTES + 1)
    if len(raw) > MAX_SOURCE_BYTES:
        process.kill()
        process.communicate()
        raise ValueError(f"Media exceeds size limit: {image_id}")
    _, stderr = process.communicate(timeout=40)
    if process.returncode:
        raise subprocess.CalledProcessError(process.returncode, process.args, stderr=stderr)
    photo = ImageOps.exif_transpose(Image.open(BytesIO(raw))).convert("RGB")
    if cards_only:
        sizes = [("card480", 480), ("card800", 800)]
    else:
        sizes = [("thumb", 520)]
    if entry["kind"] == "cover" and not cards_only:
        sizes.extend((("card480", 480), ("card800", 800), ("preview", 1200)))
    total = 0
    for suffix, width in sizes:
        resized = photo.copy()
        resized.thumbnail((width, width), Image.Resampling.LANCZOS)
        if resized.width != min(width, photo.width):
            raise ValueError(f"Unexpected derivative width for {image_id}-{suffix}: {resized.width}")
        target = OUTPUT / f"{image_id}-{suffix}.webp"
        target.write_bytes(encode_derivative(photo, width))
        total += target.stat().st_size
    return len(raw), total


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cards-only", action="store_true")
    args = parser.parse_args()
    entries = json.loads(SOURCES.read_text(encoding="utf-8"))
    if args.cards_only:
        entries = [entry for entry in entries if entry["kind"] == "cover"]
    OUTPUT.mkdir(parents=True, exist_ok=True)
    with ThreadPoolExecutor(max_workers=6) as pool:
        sizes = list(pool.map(lambda entry: generate(entry, args.cards_only), entries))
    covers = [entry for entry in entries if entry["kind"] == "cover"]
    if covers:
        card_manifest = {
            entry["url"]: {
                "small": f"{Path(urlparse(entry['url']).path).stem}-card480.webp",
                "large": f"{Path(urlparse(entry['url']).path).stem}-card800.webp",
                "fallback": f"{Path(urlparse(entry['url']).path).stem}-preview.webp",
                "fallback_width": Image.open(
                    OUTPUT / f"{Path(urlparse(entry['url']).path).stem}-preview.webp"
                ).width,
            }
            for entry in covers
        }
        CARD_MANIFEST.write_text(
            json.dumps(card_manifest, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    print(
        f"Generated {len(entries)} RV photo previews: "
        f"{sum(before for before, _ in sizes) / 1048576:.1f} MiB source -> "
        f"{sum(after for _, after in sizes) / 1048576:.1f} MiB WebP"
    )


if __name__ == "__main__":
    main()
