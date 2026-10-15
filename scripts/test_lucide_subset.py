from __future__ import annotations

import json
import unittest
from pathlib import Path

from scripts.generate_lucide_subset import (
    LEGACY_SUBSET_FONT,
    MANIFEST,
    TOKEN_RE,
    directly_used_codepoints,
    mappings,
    used_icon_names,
    sha256,
)


class LucideSubsetTests(unittest.TestCase):
    def test_manifest_covers_every_icon_name_found_in_runtime_sources(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.assertEqual(manifest["used_names"], used_icon_names(mappings()))
        expected_points = {mappings()[name] for name in manifest["used_names"]}
        expected_points.update(directly_used_codepoints())
        self.assertEqual(
            manifest["used_codepoints"],
            [f"U+{point:04X}" for point in sorted(expected_points)],
        )

    def test_a_new_runtime_icon_requires_subset_regeneration(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        icon_map = mappings()
        unused = next(name for name in sorted(icon_map) if name not in manifest["used_names"])
        discovered = used_icon_names(icon_map, {unused})
        self.assertIn(unused, discovered)
        self.assertNotEqual(manifest["used_names"], discovered)

    def test_every_mapped_name_is_detectable_as_a_standalone_quoted_token(self) -> None:
        for name in mappings():
            with self.subTest(name=name):
                self.assertIn(name, TOKEN_RE.findall(f'"{name}"'))

    def test_close_icon_is_in_the_generated_subset(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.assertIn("x", manifest["used_names"])
        self.assertIn("U+E1B2", manifest["used_codepoints"])

    def test_original_font_and_license_remain_present(self) -> None:
        root = Path(__file__).resolve().parents[1]
        self.assertTrue((root / "public/lucide-1.24.0.woff2").is_file())
        self.assertTrue((root / "public/lucide-1.24.0-used.woff2").is_file())
        self.assertTrue((root / "public/vendor/lucide/LICENSE-1.24.0.txt").is_file())

    def test_hashed_subset_filename_matches_bytes_manifest_and_css(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        subset = MANIFEST.parents[1] / "public" / manifest["subset_font"]
        digest = sha256(subset)
        self.assertEqual(manifest["subset_sha256"], digest)
        self.assertEqual(subset.name, f"lucide-1.24.0-used-{digest[:16]}.woff2")
        shell = (MANIFEST.parents[1] / "index.html").read_text(encoding="utf-8")
        self.assertIn(f'url("/{{base_path}}/{subset.name}")', shell)

    def test_deployed_legacy_subset_remains_byte_identical(self) -> None:
        self.assertEqual(
            sha256(LEGACY_SUBSET_FONT),
            "0fd8c3f0ef74b4c98da864f2b4628ffab9998bc3bfc404e5be03f0c999bfac61",
        )


if __name__ == "__main__":
    unittest.main()
