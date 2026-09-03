#!/usr/bin/env python3
"""Verify that a GitHub Pages artifact contains every referenced local asset."""

from __future__ import annotations

import re
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlparse


class AssetCollector(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.urls: set[str] = set()

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        values = dict(attrs)
        if tag == "link" and values.get("href"):
            self.urls.add(values["href"] or "")
        elif tag == "script" and values.get("src"):
            self.urls.add(values["src"] or "")


def artifact_path(root: Path, raw_url: str) -> Path | None:
    parsed = urlparse(raw_url)
    if parsed.scheme or parsed.netloc:
        return None

    path = unquote(parsed.path)
    if "/assets/" in path:
        return root / "assets" / path.split("/assets/", 1)[1]

    name = Path(path).name
    if Path(name).suffix.lower() in {
        ".ico",
        ".js",
        ".json",
        ".png",
        ".webmanifest",
        ".webp",
    }:
        return root / name
    return None


def verify_html(root: Path, html_path: Path) -> list[str]:
    failures: list[str] = []
    html = html_path.read_text(encoding="utf-8")
    collector = AssetCollector()
    collector.feed(html)

    if any("@latest" in url for url in collector.urls):
        failures.append(f"{html_path.name} must pin external asset versions")

    for url in sorted(collector.urls):
        path = artifact_path(root, url)
        if path is not None and not path.is_file():
            failures.append(f"{html_path.name} references missing asset: {url}")

    main_links = [url for url in collector.urls if re.search(r"/main-dxh[^/]*\.css(?:$|[?#])", url)]
    if len(main_links) != 1:
        failures.append(f"{html_path.name} must reference exactly one hashed main stylesheet")
    else:
        main_path = artifact_path(root, main_links[0])
        if main_path and main_path.is_file():
            css = main_path.read_text(encoding="utf-8")
            if not re.search(r"--vl-css-ready\s*:\s*1", css):
                failures.append(f"{main_path.name} is missing the CSS readiness marker")

    recovery_markers = (
        'id="vl-critical-shell"',
        "vl-css-recovery",
        "localStylesheetFailed",
        'addEventListener("error"',
    )
    if any(marker not in html for marker in recovery_markers):
        failures.append(f"{html_path.name} is missing CSS fallback or recovery markup")

    return failures


def verify_artifact(root: Path) -> list[str]:
    failures: list[str] = []
    html_paths = sorted(root.glob("**/index.html"))
    html_paths.append(root / "404.html")
    for path in html_paths:
        if not path.is_file():
            failures.append(f"missing {path.relative_to(root)}")
        else:
            failures.extend(verify_html(root, path))

    expected_routes = {
        "index.html",
        "delivery/index.html",
        "parks-in-our-range/index.html",
        "rv/jayco26/index.html",
    }
    actual_routes = {path.relative_to(root).as_posix() for path in html_paths if path.is_file()}
    for route in sorted(expected_routes - actual_routes):
        failures.append(f"missing prerendered route: {route}")

    generations: dict[tuple[str, str], list[str]] = {}
    assets = root / "assets"
    if assets.is_dir():
        for path in assets.rglob("*"):
            if not path.is_file():
                continue
            match = re.match(r"^(.+)-dxh[0-9a-f]+\.([^.]+)$", path.name)
            if match:
                key = (match.group(1), match.group(2))
                generations.setdefault(key, []).append(path.name)

    for (stem, suffix), names in sorted(generations.items()):
        if len(names) > 2:
            failures.append(
                f"assets contain more than two generations of {stem}.{suffix}: "
                + ", ".join(sorted(names))
            )

    return failures


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: verify_pages_artifact.py <artifact-root>", file=sys.stderr)
        return 2

    root = Path(sys.argv[1]).resolve()
    failures = verify_artifact(root)

    if failures:
        print("GitHub Pages artifact verification failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("GitHub Pages artifact references and CSS recovery checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
