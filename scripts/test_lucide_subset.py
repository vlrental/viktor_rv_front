from __future__ import annotations

import json
import unittest
from pathlib import Path

from scripts.generate_lucide_subset import (
    MANIFEST,
    TOKEN_RE,
    directly_used_codepoints,
    mappings,
    used_icon_names,
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


if __name__ == "__main__":
    unittest.main()
