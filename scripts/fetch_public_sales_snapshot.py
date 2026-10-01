#!/usr/bin/env python3
"""Fetch and validate the public RV sale inventory for Pages prerendering."""

from __future__ import annotations

import json
import sys
import uuid
import subprocess
from pathlib import Path


DEFAULT_URL = "https://api.vlrental.ca/api/v1/rv-sales"
REQUIRED_FIELDS = ("sale_id", "title", "summary", "description", "status", "photos")


def validate_snapshot(value: object) -> list[dict[str, object]]:
    if not isinstance(value, list):
        raise ValueError("public RV sales response must be a list")
    listings: list[dict[str, object]] = []
    seen: set[str] = set()
    for raw in value:
        if not isinstance(raw, dict):
            raise ValueError("every public RV sale listing must be an object")
        missing = [field for field in REQUIRED_FIELDS if field not in raw]
        if missing:
            raise ValueError(f"public RV sale listing is missing: {', '.join(missing)}")
        sale_id = str(raw["sale_id"])
        uuid.UUID(sale_id)
        if sale_id in seen:
            raise ValueError(f"duplicate public RV sale listing: {sale_id}")
        if raw["status"] != "published":
            raise ValueError(f"public RV sale listing is not published: {sale_id}")
        if not str(raw["title"]).strip() or not str(raw["summary"]).strip():
            raise ValueError(f"public RV sale listing has empty crawlable copy: {sale_id}")
        if not isinstance(raw["photos"], list):
            raise ValueError(f"public RV sale listing photos must be a list: {sale_id}")
        seen.add(sale_id)
        listings.append(raw)
    return listings


def fetch_snapshot(url: str) -> list[dict[str, object]]:
    result = subprocess.run(
        ["curl", "--fail", "--silent", "--show-error", "--max-time", "30", "--header", "Accept: application/json", url],
        check=True,
        capture_output=True,
        text=True,
    )
    return validate_snapshot(json.loads(result.stdout))


def main() -> int:
    if len(sys.argv) not in (2, 3):
        print("usage: fetch_public_sales_snapshot.py <output-json> [api-url]", file=sys.stderr)
        return 2
    target = Path(sys.argv[1]).resolve()
    url = sys.argv[2] if len(sys.argv) == 3 else DEFAULT_URL
    listings = fetch_snapshot(url)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(listings, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Fetched {len(listings)} published RV sale listing(s).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
