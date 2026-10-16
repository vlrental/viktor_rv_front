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
    RUNTIME_LOGO_MANIFEST,
    RUNTIME_LOGO_ORIGINAL,
    RUNTIME_LOGO_REFERENCE,
    assert_same_decoded_pixels,
    logo_derivative,
    runtime_logo_derivative,
)


class ResponsiveImageTests(unittest.TestCase):
    def test_snapshot_logo_is_content_hashed_lossless_and_deterministic(self) -> None:
        first_derivatives, first_manifest = logo_derivative(LOGO_SOURCE)
        second_derivatives, second_manifest = logo_derivative(LOGO_SOURCE)
        self.assertEqual(first_derivatives, second_derivatives)
        self.assertEqual(first_manifest, second_manifest)

        committed = json.loads(LOGO_MANIFEST.read_text(encoding="utf-8"))
        self.assertEqual(committed, first_manifest)
        self.assertEqual(committed["source_sha256"], hashlib.sha256(LOGO_SOURCE.read_bytes()).hexdigest())
        self.assertEqual(set(committed["variants"]), {"72", "144", "512"})
        with Image.open(LOGO_SOURCE) as source:
            source.load()
            for width_text, variant in committed["variants"].items():
                width = int(width_text)
                target = ROOT / "public" / variant["derivative"]
                derivative_bytes = first_derivatives[variant["derivative"]]
                self.assertEqual(target.read_bytes(), derivative_bytes)
                self.assertEqual(variant["derivative_sha256"], hashlib.sha256(derivative_bytes).hexdigest())
                self.assertEqual(
                    variant["derivative"],
                    f"logo-{width}-lossless-{variant['derivative_sha256'][:16]}.webp",
                )
                self.assertEqual(derivative_bytes[:4], b"RIFF")
                self.assertEqual(derivative_bytes[8:12], b"WEBP")
                with Image.open(target) as derivative:
                    derivative.load()
                    expected = source if width == 512 else source.resize((width, width), Image.Resampling.LANCZOS)
                    self.assertEqual((derivative.mode, derivative.size), ("RGB", (width, width)))
                    self.assertIsNone(ImageChops.difference(expected, derivative).getbbox())
        self.assertLess(
            sum((ROOT / "public" / item["derivative"]).stat().st_size for item in committed["variants"].values()),
            LOGO_SOURCE.stat().st_size,
        )

    def test_logo_derivative_uses_the_source_filename_and_rejects_shape_changes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            unexpected = Path(temporary) / "unexpected.png"
            Image.new("RGB", (64, 64), "white").save(unexpected)
            with self.assertRaisesRegex(ValueError, "Unexpected logo image contract"):
                logo_derivative(unexpected)

    def test_runtime_logo_preserves_the_exact_compiled_reference_pixels(self) -> None:
        filename, derivative, manifest = runtime_logo_derivative()
        self.assertEqual(
            manifest,
            json.loads(RUNTIME_LOGO_MANIFEST.read_text(encoding="utf-8")),
        )
        self.assertEqual(manifest["original_source_sha256"], hashlib.sha256(RUNTIME_LOGO_ORIGINAL.read_bytes()).hexdigest())
        self.assertEqual(manifest["reference_sha256"], hashlib.sha256(RUNTIME_LOGO_REFERENCE.read_bytes()).hexdigest())
        self.assertEqual(filename, f"logo-runtime-120-lossless-{hashlib.sha256(derivative).hexdigest()[:16]}.webp")
        self.assertEqual((ROOT / "public" / filename).read_bytes(), derivative)
        with Image.open(RUNTIME_LOGO_REFERENCE) as reference, Image.open(ROOT / "public" / filename) as converted:
            reference.load()
            converted.load()
            self.assertEqual((converted.mode, converted.size), ("RGBA", (120, 120)))
            self.assertIsNone(ImageChops.difference(reference, converted).getbbox())
        self.assertLess(len(derivative), RUNTIME_LOGO_REFERENCE.stat().st_size)
        for component in (ROOT / "src/components/header.rs", ROOT / "src/components/footer.rs"):
            source = component.read_text(encoding="utf-8")
            self.assertIn(f'const RUNTIME_LOGO_FILE: &str = "/{filename}";', source)
            self.assertNotIn('asset!("/assets/img/logo.png"', source)

    def test_rgba_pixel_check_rejects_rgb_changes_with_unchanged_alpha(self) -> None:
        original = Image.new("RGBA", (1, 1), (10, 20, 30, 255))
        changed = Image.new("RGBA", (1, 1), (11, 20, 30, 255))
        with self.assertRaisesRegex(ValueError, "changed decoded pixels"):
            assert_same_decoded_pixels(original, changed, "runtime")


if __name__ == "__main__":
    unittest.main()
