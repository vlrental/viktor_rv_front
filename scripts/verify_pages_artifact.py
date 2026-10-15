#!/usr/bin/env python3
"""Verify that a GitHub Pages artifact contains every referenced local asset."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlparse


PROJECT_ROOT = Path(__file__).resolve().parents[1]
SNAPSHOT_LOGO_MANIFEST = json.loads(
    (PROJECT_ROOT / "content" / "snapshot_brand_logo.json").read_text(encoding="utf-8")
)


class AssetCollector(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.urls: set[str] = set()

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        values = dict(attrs)
        if tag == "link" and values.get("href"):
            self.urls.add(values["href"] or "")
        elif tag == "img" and values.get("src"):
            self.urls.add(values["src"] or "")
        elif tag == "script" and values.get("src"):
            self.urls.add(values["src"] or "")
        elif tag == "meta" and (
            values.get("property") == "og:image" or values.get("name") == "twitter:image"
        ) and values.get("content"):
            self.urls.add(values["content"] or "")


def artifact_path(root: Path, raw_url: str) -> Path | None:
    parsed = urlparse(raw_url)
    if parsed.netloc and parsed.netloc not in {"vlrental.ca", "vlrental.github.io"}:
        return None

    path = unquote(parsed.path)
    if "/assets/" in path:
        return root / "assets" / path.split("/assets/", 1)[1]
    if "/rv-previews/" in path:
        return root / "rv-previews" / path.split("/rv-previews/", 1)[1]

    name = Path(path).name
    if Path(name).suffix.lower() in {
        ".ico",
        ".js",
        ".json",
        ".png",
        ".webmanifest",
        ".webp",
        ".woff2",
    }:
        return root / name
    return None


def verify_html(root: Path, html_path: Path) -> list[str]:
    failures: list[str] = []
    html = html_path.read_text(encoding="utf-8")
    collector = AssetCollector()
    collector.feed(html)
    collector.urls.update(re.findall(r'url\(["\']?([^"\')]+\.woff2)["\']?\)', html))

    if html_path.name == "index.html" and 'name="robots" content="index,follow' in html:
        relative = html_path.parent.relative_to(root).as_posix()
        route = "/" if relative == "." else f"/{relative}"
        marker = f'<div class="seo-prerender" data-seo-route="{route}">'
        if marker not in html:
            failures.append(f"{route} is missing its visible route-specific snapshot")
        if len(re.findall(r"<h1(?:\s[^>]*)?>", html)) != 1:
            failures.append(f"{route} must have exactly one initial HTML h1")
        snapshot = html.split(marker, 1)[-1].split("</main>", 1)[0]
        if len(re.findall(r'<a\s+[^>]*href="[^"]+"', snapshot)) < 3:
            failures.append(f"{route} needs crawlable internal links")
        if re.search(r'<(?:div|article)\b(?=[^>]*\bseo-prerender\b)(?=[^>]*\bhidden(?:\s|=|>))[^>]*>', html):
            failures.append(f"{route} must not hide the initial HTML content")

    if any("@latest" in url for url in collector.urls):
        failures.append(f"{html_path.name} must pin external asset versions")

    for url in sorted(collector.urls):
        path = artifact_path(root, url)
        if path is not None and not path.is_file():
            failures.append(f"{html_path.name} references missing asset: {url}")

    local_css_links = [
        url for url in collector.urls
        if re.search(r"/assets/[^/]+\.css(?:$|[?#])", url)
        and urlparse(url).netloc in {"", "vlrental.ca", "vlrental.github.io"}
    ]
    if len(local_css_links) != 1:
        failures.append(f"{html_path.name} must reference exactly one bundled local stylesheet")
    main_links = [url for url in local_css_links if re.search(r"/main-dxh[^/]*\.css(?:$|[?#])", url)]
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


def verify_snapshot_brand_logo(root: Path) -> list[str]:
    filename = str(SNAPSHOT_LOGO_MANIFEST["derivative"])
    expected = str(SNAPSHOT_LOGO_MANIFEST["derivative_sha256"])
    path = root / filename
    if not path.is_file():
        return [f"missing snapshot brand logo: {filename}"]
    actual = hashlib.sha256(path.read_bytes()).hexdigest()
    expected_name = f"logo-512-lossless-{actual[:16]}.webp"
    if actual != expected or filename != expected_name:
        return [f"snapshot brand logo hash mismatch: {filename}"]
    return []


def application_bundle_refs(root: Path, html_path: Path) -> tuple[set[str], set[str]]:
    html = html_path.read_text(encoding="utf-8")
    scripts = set(re.findall(r"/assets/(viktor_rv_front-dxh[0-9a-f]+\.js)", html))
    wasm: set[str] = set()
    for script in scripts:
        script_path = root / "assets" / script
        if script_path.is_file():
            wasm.update(re.findall(r"(viktor_rv_front_bg-dxh[0-9a-f]+\.wasm)", script_path.read_text(encoding="utf-8")))
    return scripts, wasm


def verify_bundle_split(root: Path, html_paths: list[Path]) -> list[str]:
    failures: list[str] = []
    home = root / "index.html"
    admin = root / "admin/index.html"
    if not home.is_file() or not admin.is_file():
        return ["storefront and admin documents are both required"]
    storefront_refs = application_bundle_refs(root, home)
    admin_refs = application_bundle_refs(root, admin)
    if any(len(refs) != 1 for refs in storefront_refs + admin_refs):
        failures.append("storefront and admin documents must each reference one JS/WASM pair")
        return failures
    if storefront_refs == admin_refs:
        failures.append("admin document must reference the distinct full application bundle")
    for label, refs in (("storefront", storefront_refs), ("admin", admin_refs)):
        for filename in refs[0] | refs[1]:
            if not (root / "assets" / filename).is_file():
                failures.append(f"{label} bundle references missing asset: assets/{filename}")
    for path in html_paths:
        if path == admin or not path.is_file():
            continue
        if application_bundle_refs(root, path) != storefront_refs:
            failures.append(f"{path.relative_to(root)} must reference the storefront bundle")
    return failures


def verify_artifact(root: Path) -> list[str]:
    failures = verify_snapshot_brand_logo(root)
    html_paths = sorted(root.glob("**/index.html"))
    html_paths.append(root / "404.html")
    for path in html_paths:
        if not path.is_file():
            failures.append(f"missing {path.relative_to(root)}")
        else:
            failures.extend(verify_html(root, path))
    failures.extend(verify_bundle_split(root, html_paths))

    expected_routes = {
        "index.html",
        "delivery/index.html",
        "parks-in-our-range/index.html",
        "parks/bear-creek/index.html",
        "parks/fintry/index.html",
        "parks/ellison/index.html",
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
        limit = 4 if stem in {"viktor_rv_front", "viktor_rv_front_bg"} else 2
        if len(names) > limit:
            failures.append(
                f"assets contain more than {limit} bounded generations of {stem}.{suffix}: "
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
