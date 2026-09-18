#!/usr/bin/env python3
"""Generate exact WebP previews of the current public RV photos.

The source manifest is intentionally explicit. When admin media changes, refresh
the manifest and generated assets together; unknown URLs use their original.
"""

from concurrent.futures import ThreadPoolExecutor
from io import BytesIO
import json
from pathlib import Path
from urllib.parse import urlparse
from urllib.request import urlopen

from PIL import Image, ImageOps

ROOT = Path(__file__).resolve().parents[1]
SOURCES = ROOT / "scripts" / "rv_preview_sources.json"
OUTPUT = ROOT / "public" / "rv-previews"
PROJECT_HOST = "pwhlkpwlansarstmstge.supabase.co"
PUBLIC_PREFIX = "/storage/v1/object/public/rental-media/rentals/"
MAX_SOURCE_BYTES = 12 * 1024 * 1024


def generate(entry: dict[str, str]) -> tuple[int, int]:
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
    with urlopen(url, timeout=35) as response:
        raw = response.read(MAX_SOURCE_BYTES + 1)
    if len(raw) > MAX_SOURCE_BYTES:
        raise ValueError(f"Media exceeds size limit: {image_id}")
    photo = ImageOps.exif_transpose(Image.open(BytesIO(raw))).convert("RGB")
    sizes = [("thumb", 520)]
    if entry["kind"] == "cover":
        sizes.append(("preview", 1200))
    total = 0
    for suffix, width in sizes:
        resized = photo.copy()
        resized.thumbnail((width, width), Image.Resampling.LANCZOS)
        target = OUTPUT / f"{image_id}-{suffix}.webp"
        resized.save(target, "WEBP", quality=78, method=6)
        total += target.stat().st_size
    return len(raw), total


def main() -> None:
    entries = json.loads(SOURCES.read_text(encoding="utf-8"))
    OUTPUT.mkdir(parents=True, exist_ok=True)
    with ThreadPoolExecutor(max_workers=6) as pool:
        sizes = list(pool.map(generate, entries))
    print(
        f"Generated {len(entries)} RV photo previews: "
        f"{sum(before for before, _ in sizes) / 1048576:.1f} MiB source -> "
        f"{sum(after for _, after in sizes) / 1048576:.1f} MiB WebP"
    )


if __name__ == "__main__":
    main()
