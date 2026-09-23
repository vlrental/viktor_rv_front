from __future__ import annotations

import tempfile
import unittest
import html
import json
import re
from pathlib import Path

from scripts.prepare_pages_artifact import (
    EXPECTED_STYLES,
    LEGACY_REDIRECTS,
    PUBLIC_ROUTES,
    bundle_route_stylesheets,
    page_url,
    prepare_artifact,
)


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
        root = Path(temporary.name)
        (root / "index.html").write_text(SHELL, encoding="utf-8")
        return root

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
        self.assertEqual(len(previews), 48)
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

    def test_private_and_not_found_documents_are_noindex(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")

        for relative in ("checkout/index.html", "account/index.html", "admin/index.html", "404.html"):
            document = (root / relative).read_text(encoding="utf-8")
            self.assertIn('name="robots" content="noindex,nofollow"', document)

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
            self.assertEqual(breadcrumb["itemListElement"][-1]["item"], canonical)
            self.assertNotIn('"@type": "Product"', document)
            for node in graph:
                for unverified_field in ("offers", "review", "aggregateRating", "priceRange"):
                    self.assertNotIn(unverified_field, node, route.path)

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

    def test_park_and_rv_documents_have_relevant_crawlable_links(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        park = (root / "parks/bear-creek/index.html").read_text(encoding="utf-8")
        rv = (root / "rv/jayco26/index.html").read_text(encoding="utf-8")
        hub = (root / "parks-in-our-range/index.html").read_text(encoding="utf-8")

        self.assertIn("Summer camping requires a reservation", park)
        self.assertIn('href="https://example.test/rv/2025-open-range-1/"', park)
        self.assertIn('href="https://example.test/parks-in-our-range/"', rv)
        self.assertIn('href="https://example.test/parks/herald/"', hub)
        self.assertEqual(park.count("<h1>"), 1)

    def test_outer_range_park_snapshots_require_delivery_approval(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")

        for slug in ("swiws", "shuswap-lake", "herald"):
            document = (root / "parks" / slug / "index.html").read_text(encoding="utf-8")
            self.assertIn("150 km" if slug != "herald" else "route and campsite access", document)
            self.assertIn("Before you book:", document)

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
