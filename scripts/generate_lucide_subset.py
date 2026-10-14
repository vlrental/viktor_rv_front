#!/usr/bin/env python3
"""Generate and verify the locally used Lucide icon-font partition."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CSS_PATH = ROOT / "assets" / "main.css"
SOURCE_FONT = ROOT / "public" / "lucide-1.24.0.woff2"
SUBSET_FONT = ROOT / "public" / "lucide-1.24.0-used.woff2"
MANIFEST = ROOT / "content" / "lucide_used_icons.json"
SHELL = ROOT / "index.html"
BLOCK_START = "            /* lucide-font-partition:start */"
BLOCK_END = "            /* lucide-font-partition:end */"
MAPPING_RE = re.compile(
    r'\.icon-([a-z0-9-]+)::before\s*\{\s*content:\s*"\\([0-9a-fA-F]+)";\s*\}'
)
TOKEN_RE = re.compile(r"[a-z][a-z0-9-]+")
ESCAPED_POINT_RE = re.compile(r"\\([eE][0-9a-fA-F]{3})")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def mappings() -> dict[str, int]:
    return {name: int(point, 16) for name, point in MAPPING_RE.findall(CSS_PATH.read_text())}


def runtime_sources() -> list[Path]:
    paths = list((ROOT / "src").rglob("*.rs"))
    paths += [path for path in (ROOT / "content").rglob("*.json") if path != MANIFEST]
    paths += list((ROOT / "assets").rglob("*.js"))
    paths += list((ROOT / "assets").rglob("*.css"))
    paths += list((ROOT / "public").rglob("*.js"))
    paths.append(SHELL)
    return sorted(set(paths))


def used_icon_names(icon_map: dict[str, int], extra_tokens: set[str] | None = None) -> list[str]:
    tokens: set[str] = set()
    for path in runtime_sources():
        text = path.read_text(encoding="utf-8")
        if path == CSS_PATH:
            text = MAPPING_RE.sub("", text)
        tokens.update(TOKEN_RE.findall(text))
    tokens.update(extra_tokens or set())
    return sorted(tokens & icon_map.keys())


def directly_used_codepoints() -> set[int]:
    points: set[int] = set()
    for path in runtime_sources():
        text = path.read_text(encoding="utf-8")
        if path == CSS_PATH:
            text = MAPPING_RE.sub("", text)
        points.update(int(value, 16) for value in ESCAPED_POINT_RE.findall(text))
    return points


def unicode_ranges(points: set[int]) -> str:
    if not points:
        raise ValueError("unicode range cannot be empty")
    ordered = sorted(points)
    ranges: list[tuple[int, int]] = []
    start = previous = ordered[0]
    for point in ordered[1:]:
        if point == previous + 1:
            previous = point
            continue
        ranges.append((start, previous))
        start = previous = point
    ranges.append((start, previous))
    return ", ".join(
        f"U+{start:04X}" if start == end else f"U+{start:04X}-{end:04X}"
        for start, end in ranges
    )


def render_block(used_points: set[int], fallback_points: set[int]) -> str:
    return "\n".join(
        [
            BLOCK_START,
            '            @font-face {',
            '                font-family: "lucide";',
            '                src: url("/{base_path}/lucide-1.24.0-used.woff2") format("woff2");',
            '                font-style: normal;',
            '                font-weight: 400;',
            '                font-display: swap;',
            f'                unicode-range: {unicode_ranges(used_points)};',
            '            }',
            '            @font-face {',
            '                font-family: "lucide";',
            '                src: url("/{base_path}/lucide-1.24.0.woff2") format("woff2");',
            '                font-style: normal;',
            '                font-weight: 400;',
            '                font-display: swap;',
            f'                unicode-range: {unicode_ranges(fallback_points)};',
            '            }',
            BLOCK_END,
        ]
    )


def replace_block(shell: str, block: str) -> str:
    pattern = re.compile(re.escape(BLOCK_START) + r".*?" + re.escape(BLOCK_END), re.DOTALL)
    updated, count = pattern.subn(block, shell, count=1)
    if count != 1:
        raise ValueError("index.html must contain one Lucide partition block")
    return updated


def build_subset(target: Path, used_points: set[int]) -> tuple[set[int], set[int]]:
    from fontTools import subset
    from fontTools.pens.recordingPen import DecomposingRecordingPen
    from fontTools.ttLib import TTFont

    original = TTFont(SOURCE_FONT)
    full_points = set(original.getBestCmap())
    if not used_points <= full_points:
        raise ValueError(f"Lucide source is missing points: {sorted(used_points - full_points)}")

    generated = TTFont(SOURCE_FONT)
    generated.recalcBBoxes = False
    generated.recalcTimestamp = False
    options = subset.Options()
    options.flavor = "woff2"
    options.recalc_bounds = False
    options.recalc_timestamp = False
    subsetter = subset.Subsetter(options=options)
    subsetter.populate(unicodes=used_points)
    subsetter.subset(generated)
    generated.recalcBBoxes = False
    generated.recalcTimestamp = False
    generated.save(target)

    subset_font = TTFont(target)
    if set(subset_font.getBestCmap()) != used_points:
        raise ValueError("generated subset cmap differs from the requested points")
    original_glyphs = original.getGlyphSet()
    subset_glyphs = subset_font.getGlyphSet()
    for point in sorted(used_points):
        original_name = original.getBestCmap()[point]
        subset_name = subset_font.getBestCmap()[point]
        original_pen = DecomposingRecordingPen(original_glyphs)
        subset_pen = DecomposingRecordingPen(subset_glyphs)
        original_glyphs[original_name].draw(original_pen)
        subset_glyphs[subset_name].draw(subset_pen)
        if original_pen.value != subset_pen.value:
            raise ValueError(f"outline changed for U+{point:04X}")
        if original["hmtx"][original_name] != subset_font["hmtx"][subset_name]:
            raise ValueError(f"horizontal metrics changed for U+{point:04X}")
    return full_points, full_points - used_points


def generated_outputs(directory: Path) -> tuple[Path, str, str]:
    icon_map = mappings()
    names = used_icon_names(icon_map)
    used_points = {icon_map[name] for name in names} | directly_used_codepoints()
    subset_path = directory / SUBSET_FONT.name
    full_points, fallback_points = build_subset(subset_path, used_points)
    if used_points & fallback_points or used_points | fallback_points != full_points:
        raise ValueError("Lucide unicode partitions must be disjoint and complete")
    future_points = {point for point in icon_map.values() if point not in used_points}
    if not future_points <= fallback_points:
        raise ValueError("unused mapped icons must remain available from the full font")
    manifest = json.dumps(
        {
            "version": 1,
            "source_font": SOURCE_FONT.name,
            "source_sha256": sha256(SOURCE_FONT),
            "mapping_sha256": sha256(CSS_PATH),
            "used_names": names,
            "used_codepoints": [f"U+{point:04X}" for point in sorted(used_points)],
        },
        indent=2,
        sort_keys=True,
    ) + "\n"
    return subset_path, manifest, render_block(used_points, fallback_points)


def run(check: bool) -> None:
    with tempfile.TemporaryDirectory() as temporary:
        subset_path, manifest, block = generated_outputs(Path(temporary))
        shell = SHELL.read_text(encoding="utf-8")
        updated_shell = replace_block(shell, block)
        if check:
            if not SUBSET_FONT.is_file() or SUBSET_FONT.read_bytes() != subset_path.read_bytes():
                raise SystemExit("Lucide subset is stale; run scripts/generate_lucide_subset.py")
            if not MANIFEST.is_file() or MANIFEST.read_text(encoding="utf-8") != manifest:
                raise SystemExit("Lucide used-icon manifest is stale; regenerate it")
            if updated_shell != shell:
                raise SystemExit("Lucide @font-face partition is stale; regenerate it")
        else:
            SUBSET_FONT.write_bytes(subset_path.read_bytes())
            MANIFEST.write_text(manifest, encoding="utf-8")
            SHELL.write_text(updated_shell, encoding="utf-8")
    print(f"Verified {len(json.loads(manifest)['used_names'])} used Lucide icon names")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    run(parser.parse_args().check)


if __name__ == "__main__":
    main()
