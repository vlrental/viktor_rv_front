from __future__ import annotations

import tempfile
import unittest
import html
from pathlib import Path

from scripts.prepare_pages_artifact import LEGACY_REDIRECTS, PUBLIC_ROUTES, prepare_artifact


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
            self.assertIn('<article hidden class="seo-prerender"', document)
            self.assertIn(f'data-seo-route="{route.path}"', document)
            self.assertIn(f"https://example.test{route.path}", document)

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

    def test_sitemap_is_generated_from_public_routes_only(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")
        sitemap = (root / "sitemap.xml").read_text(encoding="utf-8")

        for route in PUBLIC_ROUTES:
            self.assertIn(f"<loc>https://example.test{route.path}</loc>", sitemap)
        self.assertNotIn("/checkout", sitemap)
        self.assertNotIn("/admin", sitemap)

    def test_legacy_routes_redirect_to_current_canonical_pages(self) -> None:
        root = self.make_artifact()
        prepare_artifact(root, "https://example.test")

        for old_path, target_path in LEGACY_REDIRECTS:
            document = (root / old_path.strip("/") / "index.html").read_text(encoding="utf-8")
            target_url = f"https://example.test{target_path}"
            self.assertIn('name="robots" content="noindex,follow"', document)
            self.assertIn(f'rel="canonical" href="{target_url}"', document)
            self.assertIn(f'http-equiv="refresh" content="0; url={target_url}"', document)
            self.assertIn(f'window.location.replace("{target_url}")', document)
            self.assertIn(f'<a href="{target_url}">Continue to the current page</a>', document)


if __name__ == "__main__":
    unittest.main()
