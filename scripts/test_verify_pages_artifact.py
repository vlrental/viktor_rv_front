from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.verify_pages_artifact import verify_html


HTML = """<!doctype html>
<html>
<head>
  <style id="vl-critical-shell"></style>
  <link rel="stylesheet" href="/viktor_rv_front/assets/main-dxhtest.css">
  <script>const recovery = "vl-css-recovery";</script>
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

    def test_missing_css_marker_fails(self) -> None:
        root, html_path = self.make_artifact(":root{color:#17261c}")
        failures = verify_html(root, html_path)
        self.assertTrue(any("readiness marker" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
