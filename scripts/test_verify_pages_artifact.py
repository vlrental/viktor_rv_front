from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.verify_pages_artifact import verify_artifact, verify_html


HTML = """<!doctype html>
<html>
<head>
  <style id="vl-critical-shell"></style>
  <link rel="stylesheet" href="/viktor_rv_front/assets/main-dxhtest.css">
  <script>
    let localStylesheetFailed = false;
    addEventListener("error", () => { localStylesheetFailed = true; });
    const recovery = "vl-css-recovery";
  </script>
</head>
<body><script src="/viktor_rv_front/assets/app-dxhtest.js"></script></body>
</html>
"""


class VerifyPagesArtifactTests(unittest.TestCase):
    def make_artifact(self, main_css: str = ":root{--vl-css-ready:1}") -> tuple[Path, Path]:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        assets = root / "assets"
        assets.mkdir()
        (assets / "main-dxhtest.css").write_text(main_css, encoding="utf-8")
        (assets / "app-dxhtest.js").write_text("export {};", encoding="utf-8")
        html_path = root / "index.html"
        html_path.write_text(HTML, encoding="utf-8")
        return root, html_path

    def test_valid_artifact_passes(self) -> None:
        root, html_path = self.make_artifact()
        self.assertEqual(verify_html(root, html_path), [])

    def test_missing_local_asset_fails(self) -> None:
        root, html_path = self.make_artifact()
        (root / "assets" / "app-dxhtest.js").unlink()
        failures = verify_html(root, html_path)
        self.assertTrue(any("missing asset" in failure for failure in failures))

    def test_missing_social_preview_image_fails(self) -> None:
        root, html_path = self.make_artifact()
        html_path.write_text(
            HTML.replace(
                "</head>",
                '<meta property="og:image" content="https://vlrental.ca/assets/img/park-fintry.webp">\n</head>',
            ),
            encoding="utf-8",
        )
        failures = verify_html(root, html_path)
        self.assertTrue(any("park-fintry.webp" in failure for failure in failures))

        image = root / "assets" / "img" / "park-fintry.webp"
        image.parent.mkdir(parents=True)
        image.write_bytes(b"test-image")
        self.assertEqual(verify_html(root, html_path), [])

    def test_missing_css_marker_fails(self) -> None:
        root, html_path = self.make_artifact(":root{color:#17261c}")
        failures = verify_html(root, html_path)
        self.assertTrue(any("readiness marker" in failure for failure in failures))

    def test_unpinned_external_asset_fails(self) -> None:
        root, html_path = self.make_artifact()
        html_path.write_text(
            HTML.replace(
                "</head>",
                '<link rel="stylesheet" href="https://cdn.example.com/icons@latest/icons.css">\n</head>',
            ),
            encoding="utf-8",
        )
        failures = verify_html(root, html_path)
        self.assertTrue(any("pin external asset versions" in failure for failure in failures))

    def test_incomplete_recovery_script_fails(self) -> None:
        root, html_path = self.make_artifact()
        html_path.write_text(
            HTML.replace("localStylesheetFailed", "stylesheetFailure"),
            encoding="utf-8",
        )
        failures = verify_html(root, html_path)
        self.assertTrue(any("fallback or recovery" in failure for failure in failures))

    def test_indexable_route_requires_visible_snapshot(self) -> None:
        root, _ = self.make_artifact()
        route = root / "delivery" / "index.html"
        route.parent.mkdir()
        route.write_text(
            HTML.replace(
                "</head>", '<meta name="robots" content="index,follow">\n</head>'
            ).replace(
                "<body>",
                '<body><div class="seo-prerender" data-seo-route="/delivery"><main>'
                '<h1>RV delivery</h1><a href="/">Home</a><a href="/rv/a/">RV</a>'
                '<a href="/parks/">Parks</a></main></div>',
            ),
            encoding="utf-8",
        )
        self.assertEqual(verify_html(root, route), [])

        missing = route.read_text(encoding="utf-8").replace('class="seo-prerender"', 'class="removed"')
        route.write_text(missing, encoding="utf-8")
        self.assertTrue(any("missing its visible" in failure for failure in verify_html(root, route)))

        hidden = missing.replace('class="removed"', 'class="seo-prerender" hidden')
        route.write_text(hidden, encoding="utf-8")
        self.assertTrue(any("must not hide" in failure for failure in verify_html(root, route)))

    def test_more_than_two_asset_generations_fails(self) -> None:
        root, html_path = self.make_artifact()
        (root / "404.html").write_text(html_path.read_text(encoding="utf-8"), encoding="utf-8")
        for generation in ("111111", "222222", "333333"):
            (root / "assets" / f"main-dxh{generation}.css").write_text(
                ":root{--vl-css-ready:1}",
                encoding="utf-8",
            )
        failures = verify_artifact(root)
        self.assertTrue(any("more than two generations" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
