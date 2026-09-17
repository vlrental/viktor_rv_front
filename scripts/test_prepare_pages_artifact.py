from __future__ import annotations

import tempfile
import unittest
import html
import json
import re
from pathlib import Path

from scripts.prepare_pages_artifact import LEGACY_REDIRECTS, PUBLIC_ROUTES, page_url, prepare_artifact


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

    def test_park_and_rv_documents_have_relevant_crawlable_links(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        park = (root / "parks/bear-creek/index.html").read_text(encoding="utf-8")
        rv = (root / "rv/jayco26/index.html").read_text(encoding="utf-8")
        hub = (root / "parks-in-our-range/index.html").read_text(encoding="utf-8")

        self.assertIn("Confirm the campsite number", park)
        self.assertIn('href="https://example.test/rv/2025-open-range-1/"', park)
        self.assertIn('href="https://example.test/parks-in-our-range/"', rv)
        self.assertIn('href="https://example.test/parks/herald/"', hub)
        self.assertEqual(park.count("<h1>"), 1)

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
