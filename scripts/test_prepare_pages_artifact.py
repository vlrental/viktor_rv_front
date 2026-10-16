from __future__ import annotations

import tempfile
import unittest
import html
import hashlib
import json
import re
from io import BytesIO
from pathlib import Path

from PIL import Image

from scripts.prepare_pages_artifact import (
    EXPECTED_STYLES,
    LEGACY_REDIRECTS,
    PARK_IMAGE_SIZES,
    PUBLIC_ROUTES,
    RV_DETAIL_PRELOADS,
    SNAPSHOT_BRAND_LOGO_FILE,
    bundle_route_stylesheets,
    load_terms_content,
    page_url,
    park_responsive_image,
    prepare_artifact,
    schema_for,
)
from scripts.fetch_public_sales_snapshot import validate_snapshot
from scripts.generate_rv_previews import encode_derivative
from scripts.generate_park_previews import encode as encode_park_preview


SHELL = """<!doctype html>
<html lang="en-CA">
<head>
  <title>Shell</title>
  <meta name="description" content="Shell description">
  <meta name="robots" content="index,follow">
  <link rel="canonical" href="https://vlrental.ca/">
  <meta property="og:title" content="Shell">
  <meta property="og:description" content="Shell description">
  <meta property="og:url" content="https://vlrental.ca/">
  <meta property="og:image" content="https://vlrental.ca/og-image.webp">
  <meta name="twitter:title" content="Shell">
  <meta name="twitter:description" content="Shell description">
  <meta name="twitter:image" content="https://vlrental.ca/og-image.webp">
  <meta name="google-site-verification" content="verification-test-token">
  <script type="application/ld+json">{}</script>
</head>
<body><div id="main"></div></body>
</html>
"""


class PreparePagesArtifactTests(unittest.TestCase):
    def make_artifact(self) -> Path:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name) / "public"
        root.mkdir()
        (root / "index.html").write_text(SHELL, encoding="utf-8")
        (root / "hero-bullet-overlook-768.webp").write_bytes(b"small hero")
        (root / "hero-bullet-overlook.webp").write_bytes(b"large hero")
        return root

    def make_sales_snapshot(self) -> Path:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        path = Path(temporary.name) / "sales.json"
        path.write_text(json.dumps([
            {
                "sale_id": "6df950db-7844-495e-870a-d438e3dcf801",
                "title": "2015 Keystone Bullet 272BHS",
                "summary": "Family travel trailer for sale in Kelowna.",
                "description": "A well maintained RV with a large slide-out.",
                "status": "published",
                "photos": [{"source_url": "https://api.vlrental.ca/cover.webp"}],
            },
            {
                "sale_id": "d8bd4da5-b2dc-406f-aa61-30827e214d2a",
                "title": "2014 Forest River Rockwood 18 ft",
                "summary": "Compact travel trailer for sale in Kelowna.",
                "description": "An 18 ft model with a 21 ft bumper-to-bumper length.",
                "status": "published",
                "photos": [{"source_url": "https://api.vlrental.ca/rockwood.webp"}],
            },
        ]), encoding="utf-8")
        return path

    def test_current_twenty_seven_public_documents_have_unique_valid_identity(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test", self.make_sales_snapshot())
        documents = []
        for path in root.rglob("index.html"):
            document = path.read_text(encoding="utf-8")
            if 'name="robots" content="index,follow' not in document:
                continue
            title = re.search(r"<title>(.*?)</title>", document, re.DOTALL)
            description = re.search(
                r'<meta\s+name="description"\s+content="([^"]*)"', document, re.DOTALL
            )
            canonical = re.search(r'<link rel="canonical" href="([^"]*)"', document)
            schema = re.search(
                r'<script type="application/ld\+json">(.*?)</script>', document, re.DOTALL
            )
            self.assertIsNotNone(title, path)
            self.assertIsNotNone(description, path)
            self.assertIsNotNone(canonical, path)
            self.assertIsNotNone(schema, path)
            self.assertEqual(document.count("<h1>"), 1, path)
            json.loads(schema.group(1))
            documents.append((title.group(1), description.group(1), canonical.group(1)))

        self.assertEqual(len(documents), 27)
        self.assertEqual(len({item[0] for item in documents}), 27)
        self.assertEqual(len({item[1] for item in documents}), 27)
        self.assertEqual(len({item[2] for item in documents}), 27)

    def test_bundles_all_route_css_in_original_order_for_pages_base(self) -> None:
        root = self.make_artifact()
        assets = root / "assets"
        assets.mkdir()
        ordered = sorted(EXPECTED_STYLES - {"main"}) + ["main"]
        links = []
        for name in ordered:
            (assets / f"{name}-dxhtest.css").write_text(
                f".style-{name} {{ color: green; }}\n" + (":root { --vl-css-ready: 1; }" if name == "main" else ""),
                encoding="utf-8",
            )
            links.append(f'<link rel="stylesheet" href="/viktor_rv_front/assets/{name}-dxhtest.css" type="text/css">')

        bundled = bundle_route_stylesheets("<head>" + "".join(links) + "</head>", root)
        self.assertEqual(bundled.count('rel="stylesheet"'), 1)
        match = re.search(r'href="/viktor_rv_front/assets/(main-dxh[^"]+\.css)"', bundled)
        self.assertIsNotNone(match)
        css = (assets / match.group(1)).read_text(encoding="utf-8")
        self.assertEqual([css.index(f".style-{name}") for name in ordered], sorted(css.index(f".style-{name}") for name in ordered))
        self.assertIn("--vl-css-ready: 1", css)

    def test_bundling_removes_unreferenced_main_css_but_keeps_previous_bundle(self) -> None:
        root = self.make_artifact()
        assets = root / "assets"
        assets.mkdir()
        links = []
        for name in sorted(EXPECTED_STYLES):
            (assets / f"{name}-dxh1111.css").write_text(
                f".style-{name} {{ color: green; }}\n"
                + (":root { --vl-css-ready: 1; }" if name == "main" else ""),
                encoding="utf-8",
            )
            links.append(f'<link rel="stylesheet" href="/assets/{name}-dxh1111.css" type="text/css">')
        (assets / "main-dxh2222.css").write_text("old unreferenced main source", encoding="utf-8")
        previous = assets / "main-dxh3333.css"
        previous.write_text("/* VL route CSS: about */\nprevious bundle", encoding="utf-8")

        bundled = bundle_route_stylesheets("<head>" + "".join(links) + "</head>", root)
        current = re.search(r"/assets/(main-dxh[0-9a-f]+\.css)", bundled)
        self.assertIsNotNone(current)
        self.assertEqual(
            {path.name for path in assets.glob("main-dxh*.css")},
            {current.group(1), previous.name},
        )

    def test_bundling_fails_if_any_route_css_is_missing(self) -> None:
        root = self.make_artifact()
        links = "".join(
            f'<link rel="stylesheet" href="/assets/{name}-dxhtest.css" type="text/css">'
            for name in sorted(EXPECTED_STYLES)
        )
        with self.assertRaises(FileNotFoundError):
            bundle_route_stylesheets(links, root)

    def test_preserves_exact_precompressed_rv_previews(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")

        source = Path(__file__).resolve().parents[1] / "public" / "rv-previews"
        previews = list(source.glob("*.webp"))
        self.assertEqual(len(previews), 60)
        for preview in previews:
            self.assertEqual((root / "rv-previews" / preview.name).read_bytes(), preview.read_bytes())

    def test_creates_unique_public_route_documents(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")

        for route in PUBLIC_ROUTES:
            path = root / ("index.html" if route.path == "/" else f"{route.path.strip('/')}/index.html")
            self.assertTrue(path.is_file(), route.path)

            document = path.read_text(encoding="utf-8")
            self.assertIn(f"<title>{html.escape(route.title)}</title>", document)
            self.assertIn('<div class="seo-prerender"', document)
            self.assertIn(f'data-seo-route="{route.path}"', document)
            self.assertIn(f'<h1>{html.escape(route.heading)}</h1>', document)
            self.assertNotIn(' hidden class="seo-prerender"', document)
            canonical = page_url("https://example.test", route.path)
            self.assertIn(f'rel="canonical" href="{canonical}"', document)
            self.assertIn(f'property="og:url" content="{canonical}"', document)
            self.assertIn('name="google-site-verification" content="verification-test-token"', document)

        home = (root / "index.html").read_text(encoding="utf-8")
        about = (root / "about" / "index.html").read_text(encoding="utf-8")
        self.assertIn('imagesrcset="/hero-bullet-overlook-768.webp 768w, /hero-bullet-overlook.webp 1535w"', home)
        self.assertIn('class="seo-prerender-home-poster"', home)
        self.assertIn('fetchpriority="high"', home)
        self.assertNotIn("hero-bullet-overlook-768", about)
        jayco = (root / "rv" / "jayco26" / "index.html").read_text(encoding="utf-8")
        self.assertIn("sleeps up to four", jayco.lower())
        self.assertIn("full 30 ft bumper-to-bumper length", jayco)
        rockwood = (root / "rv" / "2014-forest-river-rockwood" / "index.html").read_text(
            encoding="utf-8"
        )
        self.assertIn("sleeps up to four", rockwood.lower())
        self.assertIn("full 21 ft bumper-to-bumper length", rockwood)
        for slug, capacity, length in [
            ("2015-keystone-bullet", "sleeps up to eight", "full 30 ft"),
            ("2025-open-range-1", "sleeps up to eight", "full 30 ft"),
            ("2025-highland-ridge-2", "sleeps up to eight", "full 30 ft"),
            ("2017-keystone-outback-ultra", "sleeps up to six", "full 31 ft"),
        ]:
            document = (root / "rv" / slug / "index.html").read_text(encoding="utf-8")
            self.assertIn(capacity, document.lower())
            self.assertIn(length, document)

        for old_path, target_path in LEGACY_REDIRECTS:
            document = (root / old_path.strip("/") / "index.html").read_text(encoding="utf-8")
            canonical = page_url("https://example.test", target_path)
            self.assertIn(f'rel="canonical" href="{canonical}"', document)

    def test_lucide_font_is_local_and_does_not_add_a_blocking_stylesheet(self) -> None:
        project = Path(__file__).resolve().parents[1]
        dioxus = (project / "Dioxus.toml").read_text(encoding="utf-8")
        shell = (project / "index.html").read_text(encoding="utf-8")
        main_css = (project / "assets" / "main.css").read_text(encoding="utf-8")
        manifest = json.loads(
            (project / "content" / "lucide_used_icons.json").read_text(encoding="utf-8")
        )

        self.assertNotIn("unpkg.com/lucide-static", dioxus)
        self.assertIn(
            f'src: url("/{{base_path}}/{manifest["subset_font"]}") format("woff2")', shell
        )
        self.assertIn('src: url("/{base_path}/lucide-1.24.0.woff2") format("woff2")', shell)
        self.assertEqual(shell.count("font-display: swap"), 2)
        self.assertEqual(shell.count("unicode-range:"), 2)
        self.assertIn('.icon-map-pin::before { content: "\\e111"; }', main_css)
        self.assertIn('.icon-calendar::before { content: "\\e063"; }', main_css)
        self.assertIn('.icon-phone::before { content: "\\e133"; }', main_css)
        self.assertTrue((project / "public" / "lucide-1.24.0.woff2").is_file())
        self.assertTrue((project / "public" / "lucide-1.24.0-used.woff2").is_file())
        self.assertTrue((project / "public" / manifest["subset_font"]).is_file())
        self.assertTrue((project / "public" / "vendor" / "lucide" / "LICENSE-1.24.0.txt").is_file())

    def test_detail_preloads_are_route_specific_and_match_runtime_assets(self) -> None:
        root = self.make_artifact()
        assets = root / "assets"
        assets.mkdir()
        bear_asset = "park-bear-creek-dxh1234.jpg"
        (assets / bear_asset).write_bytes(b"fingerprinted bear creek")
        prepare_artifact(root, "https://example.test")

        jayco = (root / "rv" / "jayco26" / "index.html").read_text(encoding="utf-8")
        jayco_preview = "/rv-previews/d147526c-bf9c-464a-8a21-71bd2d391e7b-preview.webp"
        self.assertIn(
            f'<link rel="preload" as="image" href="{jayco_preview}" fetchpriority="high">',
            jayco,
        )
        self.assertIn(
            f'<img class="seo-prerender-detail-poster" src="{jayco_preview}"',
            jayco,
        )
        self.assertIn('width="1140" height="456"', jayco)
        self.assertIn('alt="Jayco 26′ Fifth Wheel Rental main photo"', jayco)
        self.assertEqual(jayco.count(jayco_preview), 2)
        bear = (root / "parks" / "bear-creek" / "index.html").read_text(encoding="utf-8")
        self.assertIn(
            f'<link rel="preload" as="image" href="/assets/{bear_asset}"',
            bear,
        )
        self.assertIn(
            f'<img class="seo-prerender-detail-poster" src="/assets/{bear_asset}"',
            bear,
        )
        self.assertEqual(bear.count(f"/assets/{bear_asset}"), 4)
        for filename, width in [
            ("park-bear-creek-800-9a21f3645d45295e.webp", 800),
            ("park-bear-creek-1161-86b723844c054826.webp", 1161),
        ]:
            candidate = f"https://example.test/park-previews/{filename} {width}w"
            self.assertEqual(bear.count(candidate), 2)
        bear_route = next(route for route in PUBLIC_ROUTES if route.path == "/parks/bear-creek")
        pages_srcset, _ = park_responsive_image(
            bear_route,
            "https://vlrental.github.io/viktor_rv_front",
            "/viktor_rv_front/assets/park-bear-creek.jpg",
        )
        self.assertIn(
            "https://vlrental.github.io/viktor_rv_front/park-previews/"
            "park-bear-creek-800-9a21f3645d45295e.webp 800w",
            pages_srcset,
        )
        self.assertNotIn(".webp/ ", pages_srcset)
        self.assertTrue(
            pages_srcset.endswith(
                "/viktor_rv_front/assets/park-bear-creek.jpg 1600w"
            )
        )
        self.assertNotIn("/viktor_rv_front/viktor_rv_front/", pages_srcset)
        self.assertEqual(bear.count(f'imagesizes="{PARK_IMAGE_SIZES}"'), 1)
        self.assertEqual(bear.count(f' sizes="{PARK_IMAGE_SIZES}"'), 1)
        self.assertIn(f'imagesizes="{PARK_IMAGE_SIZES}" fetchpriority="high"', bear)
        self.assertIn(f'sizes="{PARK_IMAGE_SIZES}" width="1140"', bear)
        about = (root / "about" / "index.html").read_text(encoding="utf-8")
        self.assertNotIn('<link rel="preload" as="image"', about)
        self.assertNotIn('class="seo-prerender-detail-poster"', about)
        self.assertNotIn("park-bear-creek", jayco)
        self.assertNotIn("d147526c", bear)

    def test_shared_rv_cover_map_matches_all_preloads_and_files(self) -> None:
        project = Path(__file__).resolve().parents[1]
        covers = json.loads((project / "content" / "rv_cover_previews.json").read_text(encoding="utf-8"))
        self.assertEqual(len(covers), 6)
        self.assertEqual(
            RV_DETAIL_PRELOADS,
            {f"/rv/{slug}": filename for slug, filename in covers.items()},
        )
        for filename in covers.values():
            self.assertRegex(filename, r"^[0-9a-f-]{36}-preview\.webp$")
            self.assertTrue((project / "public" / "rv-previews" / filename).is_file())

    def test_card_preview_map_matches_exact_cover_sources_and_dimensions(self) -> None:
        project = Path(__file__).resolve().parents[1]
        sources = json.loads(
            (project / "scripts" / "rv_preview_sources.json").read_text(encoding="utf-8")
        )
        covers = {entry["url"] for entry in sources if entry["kind"] == "cover"}
        cards = json.loads(
            (project / "content" / "rv_card_previews.json").read_text(encoding="utf-8")
        )
        self.assertEqual(set(cards), covers)
        self.assertEqual(len(cards), 6)

        preview_root = project / "public" / "rv-previews"
        for source_url, variants in cards.items():
            image_id = Path(source_url).stem
            self.assertEqual(variants["small"], f"{image_id}-card480.webp")
            self.assertEqual(variants["large"], f"{image_id}-card800.webp")
            self.assertEqual(variants["fallback"], f"{image_id}-preview.webp")
            with Image.open(preview_root / variants["small"]) as small:
                self.assertEqual(small.width, 480)
            with Image.open(preview_root / variants["large"]) as large:
                self.assertEqual(large.width, 800)
            with Image.open(preview_root / variants["fallback"]) as fallback:
                self.assertEqual(fallback.width, variants["fallback_width"])
                self.assertGreaterEqual(fallback.width, large.width)

    def test_card_preview_webp_encoding_is_deterministic(self) -> None:
        project = Path(__file__).resolve().parents[1]
        source = next((project / "public" / "rv-previews").glob("*-preview.webp"))
        with Image.open(source) as image:
            rgb = image.convert("RGB")
        first = encode_derivative(rgb, 480)
        second = encode_derivative(rgb, 480)
        self.assertEqual(first, second)
        with Image.open(BytesIO(first)) as derivative:
            self.assertEqual(derivative.width, 480)

    def test_bear_creek_responsive_previews_are_content_addressed_and_exact(self) -> None:
        project = Path(__file__).resolve().parents[1]
        manifest = json.loads(
            (project / "content" / "park_detail_previews.json").read_text(encoding="utf-8")
        )
        self.assertEqual(set(manifest), {"bear-creek"})
        bear = manifest["bear-creek"]
        self.assertEqual((bear["source_width"], bear["source_height"]), (1600, 900))
        source = project / bear["source"]
        self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(), bear["source_sha256"])
        with Image.open(source) as image:
            rgb = image.convert("RGB")
        for variant in bear["variants"]:
            target = project / "public" / "park-previews" / variant["filename"]
            encoded = encode_park_preview(rgb, variant["width"])
            self.assertEqual(target.read_bytes(), encoded)
            self.assertEqual(hashlib.sha256(encoded).hexdigest(), variant["sha256"])
            self.assertIn(variant["sha256"][:16], variant["filename"])

    def test_mobile_hero_is_a_smaller_webp_and_legacy_jpeg_is_preserved(self) -> None:
        project = Path(__file__).resolve().parents[1]
        mobile = project / "public" / "hero-bullet-overlook-768.webp"
        legacy = project / "public" / "hero-bullet-overlook-768.jpg"
        self.assertTrue(mobile.is_file())
        self.assertTrue(legacy.is_file())
        self.assertLess(mobile.stat().st_size, legacy.stat().st_size)
        self.assertEqual(mobile.read_bytes()[:4], b"RIFF")
        self.assertEqual(mobile.read_bytes()[8:12], b"WEBP")

    def test_snapshot_brand_uses_hashed_lossless_derivative_without_changing_schema_logo(self) -> None:
        root = self.make_artifact()
        (root / SNAPSHOT_BRAND_LOGO_FILE).write_bytes(b"Dioxus-mutated-WebP")
        prepare_artifact(root, "https://example.test", self.make_sales_snapshot())
        document = (root / "index.html").read_text(encoding="utf-8")
        header = document.split('<header class="seo-prerender-header">', 1)[1].split("</header>", 1)[0]
        self.assertRegex(SNAPSHOT_BRAND_LOGO_FILE, r"^logo-512-lossless-[0-9a-f]{16}\.webp$")
        self.assertIn(
            f'<img src="/{SNAPSHOT_BRAND_LOGO_FILE}" width="36" height="36" alt="">VL Rental',
            header,
        )
        schema = json.loads(
            re.search(r'<script type="application/ld\+json">(.*?)</script>', document, re.DOTALL).group(1)
        )
        organization = next(node for node in schema["@graph"] if node["@type"] == "Organization")
        self.assertEqual(organization["logo"], "https://example.test/logo-512.png")
        project = Path(__file__).resolve().parents[1]
        self.assertEqual(
            (root / SNAPSHOT_BRAND_LOGO_FILE).read_bytes(),
            (project / "public" / SNAPSHOT_BRAND_LOGO_FILE).read_bytes(),
        )

    def test_private_and_not_found_documents_are_noindex(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")

        for relative in ("checkout/index.html", "account/index.html", "admin/index.html", "404.html"):
            document = (root / relative).read_text(encoding="utf-8")
            self.assertIn('name="robots" content="noindex,nofollow"', document)

    def test_published_sale_has_unique_static_document_sitemap_and_hub_link(self) -> None:
        root = self.make_artifact()
        snapshot = self.make_sales_snapshot()
        prepare_artifact(root, "https://example.test", snapshot)
        sale_id = "6df950db-7844-495e-870a-d438e3dcf801"
        document = (root / "rv-sales" / sale_id / "index.html").read_text(encoding="utf-8")
        self.assertIn('<title>2015 Keystone Bullet 272BHS for Sale | VL Rental</title>', document)
        self.assertIn('"@type": "ItemPage"', document)
        self.assertIn('href="https://example.test/rv-sales/"', document)
        self.assertIn('Family travel trailer for sale in Kelowna.', document)
        self.assertIn('property="og:image" content="https://api.vlrental.ca/cover.webp"', document)
        canonical = f"https://example.test/rv-sales/{sale_id}/"
        self.assertIn(f'rel="canonical" href="{canonical}"', document)
        self.assertIn(canonical, (root / "sitemap.xml").read_text(encoding="utf-8"))
        hub = (root / "rv-sales" / "index.html").read_text(encoding="utf-8")
        self.assertIn(f'href="{canonical}"', hub)
        legacy = (root / "rv-sales" / "listing" / "index.html").read_text(encoding="utf-8")
        self.assertIn('name="robots" content="noindex,nofollow"', legacy)
        self.assertIn("URLSearchParams", legacy)
        self.assertNotIn("/rv-sales/listing/</loc>", (root / "sitemap.xml").read_text(encoding="utf-8"))

    def test_public_sales_snapshot_rejects_non_published_rows(self) -> None:
        row = {
            "sale_id": "6df950db-7844-495e-870a-d438e3dcf801",
            "title": "RV", "summary": "Summary", "description": "Description",
            "status": "archived", "photos": [],
        }
        with self.assertRaisesRegex(ValueError, "not published"):
            validate_snapshot([row])

    def test_deactivated_sale_disappears_from_generated_routes_and_sitemap(self) -> None:
        root = self.make_artifact()
        snapshot = self.make_sales_snapshot()
        sale_id = "6df950db-7844-495e-870a-d438e3dcf801"
        prepare_artifact(root, "https://example.test", snapshot)
        sale_document = root / "rv-sales" / sale_id / "index.html"
        self.assertTrue(sale_document.is_file())

        empty = root / "empty-sales.json"
        empty.write_text("[]\n", encoding="utf-8")
        (root / "index.html").write_text(SHELL, encoding="utf-8")
        prepare_artifact(root, "https://example.test", empty)
        self.assertFalse(sale_document.exists())
        self.assertNotIn(sale_id, (root / "sitemap.xml").read_text(encoding="utf-8"))

    def test_delivery_contains_service_schema_and_search_copy(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        document = (root / "delivery" / "index.html").read_text(encoding="utf-8")

        self.assertIn('"@type": "Service"', document)
        self.assertIn('"@type": "BreadcrumbList"', document)
        self.assertIn("RV Rental Delivery &amp; Setup in Kelowna", document)

    def test_rv_routes_use_rental_service_schema_without_product_claims(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")

        for route in (route for route in PUBLIC_ROUTES if route.path.startswith("/rv/")):
            document = (root / route.path.strip("/") / "index.html").read_text(encoding="utf-8")
            match = re.search(r'<script type="application/ld\+json">(.*?)</script>', document, re.DOTALL)
            self.assertIsNotNone(match, route.path)
            graph = json.loads(match.group(1))["@graph"]
            canonical = page_url("https://example.test", route.path)
            service = next(node for node in graph if node.get("@id") == f"{canonical}#page")
            breadcrumb = next(node for node in graph if node.get("@type") == "BreadcrumbList")

            self.assertEqual(service["@type"], "Service", route.path)
            self.assertEqual(service["provider"], {"@id": "https://example.test/#organization"})
            self.assertEqual(service["url"], canonical)
            self.assertEqual(service["serviceType"], "Delivered RV rental")
            self.assertEqual(service["areaServed"], ["Kelowna", "Okanagan, British Columbia"])
            self.assertEqual(breadcrumb["itemListElement"][-1]["item"], canonical)
            self.assertNotIn('"@type": "Product"', document)
            for node in graph:
                for unverified_field in ("offers", "review", "aggregateRating", "priceRange"):
                    self.assertNotIn(unverified_field, node, route.path)

    def test_schema_uses_stable_organization_and_route_specific_primary_images(self) -> None:
        for route in PUBLIC_ROUTES:
            schema = json.loads(schema_for(route, "https://example.test"))
            canonical = page_url("https://example.test", route.path)
            organization = next(node for node in schema["@graph"] if node.get("@type") == "Organization")
            page = next(node for node in schema["@graph"] if node.get("@id") == f"{canonical}#page")
            image = next(node for node in schema["@graph"] if node.get("@type") == "ImageObject")

            self.assertEqual(organization["image"], "https://example.test/og-image.webp")
            self.assertEqual(page["primaryImageOfPage"], {"@id": f"{canonical}#primaryimage"})
            self.assertEqual(image["@id"], f"{canonical}#primaryimage")
            self.assertEqual(image["url"], image["contentUrl"])

    def test_faq_schema_comes_from_the_same_fifty_approved_answers(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        document = (root / "faq/index.html").read_text(encoding="utf-8")
        match = re.search(r'<script type="application/ld\+json">(.*?)</script>', document, re.DOTALL)
        self.assertIsNotNone(match)
        graph = json.loads(match.group(1))["@graph"]
        page = next(node for node in graph if node.get("@type") == "FAQPage")

        self.assertEqual(len(page["mainEntity"]), 50)
        self.assertEqual(page["mainEntity"][0]["@type"], "Question")
        self.assertEqual(page["mainEntity"][0]["acceptedAnswer"]["@type"], "Answer")
        self.assertIn("Two 20 lb propane tanks are supplied", json.dumps(page["mainEntity"]))

    def test_home_targets_real_kelowna_rv_queries(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        document = (root / "index.html").read_text(encoding="utf-8")

        self.assertIn("RV Rental Kelowna", document)
        self.assertIn("camper rental in Kelowna", document)
        self.assertNotIn("boat", document.lower())
        self.assertIn('href="https://example.test/rv/2025-open-range-1/"', document)
        self.assertIn('href="https://example.test/parks/bear-creek/"', document)
        self.assertIn('href="https://example.test/delivery/"', document)

    def test_home_prerenders_the_six_approved_faqs(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        document = (root / "index.html").read_text(encoding="utf-8")

        expected = [
            "Do you deliver and set up the RV?",
            "How much does an RV rental in Kelowna cost?",
            "What is included with an RV rental?",
            "What is the minimum RV rental period?",
            "Can I use the RV without full hookups?",
            "Are pets allowed in the rental RVs?",
        ]
        self.assertEqual(document.count('<section class="seo-prerender-faq-group"'), 1)
        for question in expected:
            self.assertIn(f"<summary>{question}</summary>", document)

    def test_faq_prerender_contains_all_fifty_crawlable_answers(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        document = (root / "faq/index.html").read_text(encoding="utf-8")

        self.assertEqual(document.count('<section class="seo-prerender-faq-group"'), 5)
        self.assertEqual(document.count("<details id=\"faq-"), 50)
        self.assertIn("Two 20 lb propane tanks are supplied", document)
        self.assertIn("non-refundable CA$100 pet fee", document)
        self.assertIn('href="https://example.test/contact/"', document)
        self.assertIn('href="https://example.test/terms/"', document)
        self.assertNotIn("propane refill", document.lower())

    def test_terms_prerender_uses_complete_canonical_content(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        document = (root / "terms/index.html").read_text(encoding="utf-8")
        terms = load_terms_content()

        self.assertEqual(len(terms["sections"]), 20)
        self.assertEqual(document.count('<section id="terms-'), 21)
        for phrase in [
            "RV Preparation Fee: CA$97 once per booking.",
            "Stationary Plus Protection: CA$150 fixed per trip",
            "separate refundable CA$1,000 damage deposit",
            "100% of the trip price when the request is received 14 days or less before delivery",
        ]:
            self.assertIn(phrase, document)
        self.assertIn("21. Contact and notices", document)

    def test_park_and_rv_documents_have_relevant_crawlable_links(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        park = (root / "parks/bear-creek/index.html").read_text(encoding="utf-8")
        rv = (root / "rv/jayco26/index.html").read_text(encoding="utf-8")
        hub = (root / "parks-in-our-range/index.html").read_text(encoding="utf-8")

        self.assertIn("full bumper-to-bumper trailer length", park)
        self.assertIn("Confirm the services for your booked site", park)
        self.assertIn("30-amp or 50-amp service", park)
        self.assertIn('href="https://example.test/rv/2014-forest-river-rockwood/"', park)
        self.assertIn('href="https://example.test/rv/2015-keystone-bullet/"', park)
        self.assertIn('href="https://example.test/parks-in-our-range/"', rv)
        self.assertIn('href="https://example.test/parks/herald/"', hub)
        self.assertEqual(park.count("<h1>"), 1)

    def test_every_rv_snapshot_has_the_approved_campsite_planning_section(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")

        rv_documents = [
            root / route.path.lstrip("/") / "index.html"
            for route in PUBLIC_ROUTES
            if route.path.startswith("/rv/")
        ]
        self.assertEqual(len(rv_documents), 6)
        for path in rv_documents:
            document = path.read_text(encoding="utf-8")
            self.assertEqual(document.count("Plan your campsite"), 1)
            self.assertIn(
                "Reserve your campsite separately. We confirm RV fit and delivery access before delivery.",
                document,
            )
            self.assertIn('href="https://example.test/parks/bear-creek/"', document)
            self.assertIn('href="https://example.test/parks/okanagan-lake/"', document)
            self.assertIn('href="https://example.test/parks-in-our-range/"', document)

        expected_layouts = {
            "jayco26": "Sleep layout: fixed bed, convertible sofa and convertible dinette.",
            "2015-keystone-bullet": "Sleep layout: fixed bed, bunk beds, pull-out sofa and convertible dinette.",
            "2014-forest-river-rockwood": "Sleep layout: fixed bed and pull-out sofa.",
            "2025-open-range-1": "Sleep layout: fixed bed, bunk beds, pull-out sofa and convertible dinette.",
            "2017-keystone-outback-ultra": "Sleep layout: fixed bed, bunk beds, pull-out sofa and convertible dinette.",
            "2025-highland-ridge-2": "Sleep layout: fixed bed, bunk beds, pull-out sofa and convertible dinette.",
        }
        for slug, phrase in expected_layouts.items():
            document = (root / "rv" / slug / "index.html").read_text(encoding="utf-8")
            self.assertEqual(document.count(phrase), 1)
            if slug == "2014-forest-river-rockwood":
                self.assertNotIn("dinette", document.lower())
            else:
                self.assertEqual(document.count("convertible dinette"), 1)

        expected_capacities = {
            "jayco26": "sleeps up to four",
            "2015-keystone-bullet": "sleeps up to eight",
            "2014-forest-river-rockwood": "sleeps up to four",
            "2025-open-range-1": "sleeps up to eight",
            "2017-keystone-outback-ultra": "sleeps up to six",
            "2025-highland-ridge-2": "sleeps up to eight",
        }
        for slug, phrase in expected_capacities.items():
            document = (root / "rv" / slug / "index.html").read_text(encoding="utf-8")
            self.assertIn(phrase, document.lower())

        jayco_electrical = "Electrical connection: this RV uses a regular 50-amp campsite hookup."
        thirty_amp_electrical = (
            "Electrical connection: this RV uses a regular 30-amp campsite hookup. "
            "If your site offers only 50-amp service, request a suitable adapter before delivery; "
            "it is not automatically included."
        )
        for slug in expected_layouts:
            document = (root / "rv" / slug / "index.html").read_text(encoding="utf-8")
            expected = jayco_electrical if slug == "jayco26" else thirty_amp_electrical
            self.assertEqual(document.count(expected), 1)

        api_aliases = {
            "brand-new-jayco-26ft-5th-wheel-3": "jayco26",
            "2015-keystone-bullet-272bhs-bunk-bed": "2015-keystone-bullet",
            "forest-river-rockwood-19-ft": "2014-forest-river-rockwood",
            "brand-new-open-range-conventional-26-bhs-bunk-bed-1": "2025-open-range-1",
            "keystone-outback-ultra-lite-family-trailer-with-kids-room": "2017-keystone-outback-ultra",
            "brand-new-highland-ridge-open-range-26bhs-bunk-bed-2": "2025-highland-ridge-2",
        }
        for alias, canonical in api_aliases.items():
            redirect = (root / "rv" / alias / "index.html").read_text(encoding="utf-8")
            self.assertIn(f'https://example.test/rv/{canonical}/', redirect)

        for unrelated in ["index.html", "delivery/index.html", "parks/bear-creek/index.html"]:
            document = (root / unrelated).read_text(encoding="utf-8")
            self.assertNotIn("Plan your campsite", document)

    def test_bear_creek_snapshot_keeps_route_specific_planning_copy(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        document = (root / "parks/bear-creek/index.html").read_text(encoding="utf-8")

        self.assertIn("Westside Road access approval", document)
        self.assertIn("Summer camping and the RV are reserved separately", document)
        self.assertNotIn("Okanagan Mountain Provincial Park", document)

    def test_okanagan_lake_snapshot_identifies_the_two_park_campgrounds(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        document = (root / "parks/okanagan-lake/index.html").read_text(encoding="utf-8")

        self.assertIn("11 km north of Summerland", document)
        self.assertIn("distinct North or South campground", document)
        self.assertIn("full bumper-to-bumper trailer length and hookups", document)
        self.assertIn("Okanagan Mountain Provincial Park", document)

    def test_outer_range_park_snapshots_require_delivery_approval(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")

        for slug in ("swiws", "shuswap-lake", "herald"):
            document = (root / "parks" / slug / "index.html").read_text(encoding="utf-8")
            self.assertIn("150 km", document)
            self.assertIn("Before you book:", document)

    def test_researched_park_snapshots_keep_access_facts_and_official_sources(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        expected = {
            "fintry": "34 km north of Kelowna",
            "ellison": "25th Avenue about 16 km",
            "kekuli-bay": "3 km beyond the Highway 97 turn-off",
            "okanagan-falls": "500 metres from Highway 97",
            "vaseux-lake": "25 km south of Penticton",
            "swiws": "2 km south of Osoyoos",
            "shuswap-lake": "19 km along paved road",
            "herald": "14 km east of Highway 1 at Tappen",
        }
        for slug, access_fact in expected.items():
            document = (root / "parks" / slug / "index.html").read_text(encoding="utf-8")
            self.assertIn(access_fact, document)
            self.assertIn("Arrange the campsite separately", document)
            self.assertIn("full bumper-to-bumper", document)
            self.assertIn("booked-site hookups", document)
            self.assertIn("30-amp or 50-amp", document)
            self.assertIn("Source:", document)
            self.assertIn("checked October 14, 2026", document)
            self.assertIn('href="https://bcparks.ca/', document)
        for slug in ("swiws", "shuswap-lake", "herald"):
            document = (root / "parks" / slug / "index.html").read_text(encoding="utf-8")
            self.assertIn("Delivery is not guaranteed", document)
            self.assertIn("actual", document)
            self.assertIn("150 km", document)
        shuswap = (root / "parks/shuswap-lake/index.html").read_text(encoding="utf-8")
        self.assertIn("Copper Island is not an RV destination", shuswap)

        falls = (root / "parks/okanagan-falls/index.html").read_text(encoding="utf-8")
        self.assertIn(
            "campsites are closed to the general public on the third weekend of September each year for the Salmon Feast",
            falls,
        )
        self.assertIn("Check current park notices before planning your dates.", falls)
        self.assertIn('href="https://bcparks.ca/sxwexwnitkw-park/"', falls)
        swiws = (root / "parks/swiws/index.html").read_text(encoding="utf-8")
        self.assertIn("limits camping at sẁiẁs to seven nights per calendar year", swiws)
        self.assertIn("Check the current park rules before reserving.", swiws)
        self.assertIn('href="https://bcparks.ca/swiws-park/"', swiws)

    def test_every_park_snapshot_has_the_approved_rv_comparison_links(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        park_documents = [
            root / route.path.lstrip("/") / "index.html"
            for route in PUBLIC_ROUTES
            if route.path.startswith("/parks/")
        ]
        self.assertEqual(len(park_documents), 10)
        for path in park_documents:
            document = path.read_text(encoding="utf-8")
            self.assertEqual(document.count("Compare RVs for your campsite"), 1)
            self.assertIn(
                "These are rental options, not a guarantee that they fit this park. Confirm the site’s permitted full trailer length and delivery route.",
                document,
            )
            self.assertIn(
                'href="https://example.test/rv/2014-forest-river-rockwood/">Rockwood · 21 ft · sleeps 4',
                document,
            )
            self.assertIn(
                'href="https://example.test/rv/2015-keystone-bullet/">Keystone Bullet · 30 ft · sleeps 8',
                document,
            )
            self.assertIn(
                'href="https://example.test/#home-rentals">Compare all RV rentals',
                document,
            )
            self.assertNotIn("/rv/jayco26/", document)

    def test_sitemap_is_generated_from_public_routes_only(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        sitemap = (root / "sitemap.xml").read_text(encoding="utf-8")

        for route in PUBLIC_ROUTES:
            self.assertIn(f"<loc>{page_url('https://example.test', route.path)}</loc>", sitemap)
        self.assertNotIn("/checkout", sitemap)
        self.assertNotIn("/admin", sitemap)

    def test_all_provincial_park_guides_are_public_and_indexable(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        park_paths = {
            "/parks/bear-creek",
            "/parks/fintry",
            "/parks/ellison",
            "/parks/kekuli-bay",
            "/parks/okanagan-lake",
            "/parks/okanagan-falls",
            "/parks/vaseux-lake",
            "/parks/swiws",
            "/parks/shuswap-lake",
            "/parks/herald",
        }
        public_paths = {route.path for route in PUBLIC_ROUTES}
        self.assertTrue(park_paths <= public_paths)

        for path in park_paths:
            document = (root / path.strip("/") / "index.html").read_text(encoding="utf-8")
            self.assertIn('name="robots" content="index,follow', document)
            canonical = page_url("https://example.test", path)
            self.assertIn(f'rel="canonical" href="{canonical}"', document)

    def test_park_preview_images_exist_in_artifact(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")

        for image in {route.image for route in PUBLIC_ROUTES if route.image.startswith("/assets/")}:
            target = root / image.lstrip("/")
            self.assertTrue(target.is_file(), image)
            self.assertGreater(target.stat().st_size, 0, image)

    def test_legacy_routes_redirect_to_current_canonical_pages(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")

        for old_path, target_path in LEGACY_REDIRECTS:
            document = (root / old_path.strip("/") / "index.html").read_text(encoding="utf-8")
            target_url = page_url("https://example.test", target_path)
            self.assertIn('name="robots" content="noindex,follow"', document)
            self.assertIn(f'rel="canonical" href="{target_url}"', document)
            self.assertIn(f'http-equiv="refresh" content="0; url={target_url}"', document)
            self.assertIn(f'window.location.replace("{target_url}")', document)
            self.assertIn(f'<a href="{target_url}">Continue to the current page</a>', document)

        self.assertIn(("/portfolio/jayco26", "/rv/jayco26"), LEGACY_REDIRECTS)
        self.assertIn(("/portfolio/2025openrange1", "/rv/2025-open-range-1"), LEGACY_REDIRECTS)

if __name__ == "__main__":
    unittest.main()
