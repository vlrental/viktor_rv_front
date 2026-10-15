#!/usr/bin/env python3
"""Bound Pages asset retention to the immediately previous build union."""

from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path


MARKER = ".vl-pages-bundles-v2.json"


def current_assets(root: Path) -> list[str]:
    return sorted(path.relative_to(root).as_posix() for path in (root / "assets").rglob("*") if path.is_file())


def safe_asset_path(relative: str) -> bool:
    path = Path(relative)
    return relative.startswith("assets/") and not path.is_absolute() and ".." not in path.parts


def retained(previous: Path) -> list[str]:
    marker = previous / MARKER
    if marker.is_file():
        data = json.loads(marker.read_text(encoding="utf-8"))
        paths = data.get("current_union", [])
        if not paths or any(not isinstance(path, str) or not safe_asset_path(path) for path in paths):
            raise RuntimeError("invalid v2 Pages asset marker")
        return paths
    # The v1 cache stored the current asset union flat at its root, before any
    # older generation was merged. Preserve that complete one generation.
    return sorted(f"assets/{path.relative_to(previous).as_posix()}" for path in previous.rglob("*") if path.is_file() and not path.name.startswith(".vl-pages-"))


def cached_source(previous: Path, relative: str) -> Path:
    """Resolve a retained path in either the structured v2 or flat v1 cache."""
    if (previous / MARKER).is_file():
        return previous / relative
    return previous / Path(relative).relative_to("assets")


def main() -> int:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    snapshot = subparsers.add_parser("snapshot")
    snapshot.add_argument("artifact", type=Path)
    snapshot.add_argument("marker", type=Path)
    merge = subparsers.add_parser("merge")
    merge.add_argument("previous", type=Path)
    merge.add_argument("artifact", type=Path)
    stage = subparsers.add_parser("stage")
    stage.add_argument("marker", type=Path)
    stage.add_argument("artifact", type=Path)
    stage.add_argument("cache", type=Path)
    args = parser.parse_args()
    if args.command == "snapshot":
        payload = {"version": 2, "current_union": current_assets(args.artifact)}
        args.marker.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
        return 0
    if args.command == "stage":
        data = json.loads(args.marker.read_text(encoding="utf-8"))
        paths = data.get("current_union", [])
        if not paths or any(not isinstance(path, str) or not safe_asset_path(path) for path in paths):
            raise RuntimeError("invalid current asset union")
        for relative in paths:
            source = args.artifact / relative
            destination = args.cache / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
        shutil.copy2(args.marker, args.cache / MARKER)
        return 0
    if not args.previous.is_dir():
        return 0
    for relative in retained(args.previous):
        source = cached_source(args.previous, relative)
        if not source.is_file():
            raise RuntimeError(f"cached asset marker references missing file: {relative}")
        destination = args.artifact / relative
        if destination.exists():
            if source.read_bytes() != destination.read_bytes():
                raise RuntimeError(f"cached asset collision has different bytes: {relative}")
            continue
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
