from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from PIL import Image, ImageChops

from scripts.generate_responsive_images import (
    LOGO_MANIFEST,
    LOGO_SOURCE,
    ROOT,
    logo_derivative,
)


class ResponsiveImageTests(unittest.TestCase):
    def test_snapshot_logo_is_content_hashed_lossless_and_deterministic(self) -> None:
        first_bytes, first_manifest = logo_derivative(LOGO_SOURCE)
        second_bytes, second_manifest = logo_derivative(LOGO_SOURCE)
        self.assertEqual(first_bytes, second_bytes)
        self.assertEqual(first_manifest, second_manifest)

        committed = json.loads(LOGO_MANIFEST.read_text(encoding="utf-8"))
        self.assertEqual(committed, first_manifest)
        target = ROOT / "public" / committed["derivative"]
        self.assertEqual(target.read_bytes(), first_bytes)
        self.assertEqual(committed["source_sha256"], hashlib.sha256(LOGO_SOURCE.read_bytes()).hexdigest())
        self.assertEqual(committed["derivative_sha256"], hashlib.sha256(first_bytes).hexdigest())
        self.assertEqual(
            committed["derivative"],
            f"logo-512-lossless-{committed['derivative_sha256'][:16]}.webp",
        )
        self.assertLess(target.stat().st_size, LOGO_SOURCE.stat().st_size)
        self.assertEqual(first_bytes[:4], b"RIFF")
        self.assertEqual(first_bytes[8:12], b"WEBP")

        with Image.open(LOGO_SOURCE) as source, Image.open(target) as derivative:
            source.load()
            derivative.load()
            self.assertEqual((source.mode, source.size), ("RGB", (512, 512)))
            self.assertEqual((derivative.mode, derivative.size), ("RGB", (512, 512)))
            self.assertIsNone(ImageChops.difference(source, derivative).getbbox())

    def test_logo_derivative_uses_the_source_filename_and_rejects_shape_changes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            unexpected = Path(temporary) / "unexpected.png"
            Image.new("RGB", (64, 64), "white").save(unexpected)
            with self.assertRaisesRegex(ValueError, "Unexpected logo image contract"):
                logo_derivative(unexpected)


if __name__ == "__main__":
    unittest.main()
