#!/usr/bin/env python3
"""Prepare both variants and merge the complete full-admin asset closure."""

from __future__ import annotations

import argparse
import hashlib
import shutil
from pathlib import Path

try:
    from scripts.prepare_pages_artifact import prepare_artifact
except ModuleNotFoundError:  # Direct `python3 scripts/...` execution.
    from prepare_pages_artifact import prepare_artifact


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def merge_tree(source: Path, destination: Path) -> None:
    for item in source.rglob("*"):
        if not item.is_file():
            continue
        target = destination / item.relative_to(source)
        if target.exists():
            if digest(item) != digest(target):
                raise RuntimeError(f"asset collision has different bytes: {item.relative_to(source)}")
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(item, target)


def prepare_variants(storefront: Path, admin: Path, site_url: str, sales: Path | None) -> None:
    prepare_artifact(storefront, site_url, sales)
    prepare_artifact(admin, site_url, sales)
    admin_document = admin / "admin/index.html"
    if not admin_document.is_file():
        raise RuntimeError("full build did not produce the prepared admin document")
    merge_tree(admin / "assets", storefront / "assets")
    shutil.copy2(admin_document, storefront / "admin/index.html")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("storefront", type=Path)
    parser.add_argument("admin", type=Path)
    parser.add_argument("site_url")
    parser.add_argument("sales", nargs="?", type=Path)
    args = parser.parse_args()
    prepare_variants(args.storefront.resolve(), args.admin.resolve(), args.site_url, args.sales.resolve() if args.sales else None)
    print("Prepared and merged storefront/admin Pages variants")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
