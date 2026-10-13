from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PUBLIC_CONTENT_ROOTS = (
    ROOT / "src",
    ROOT / "assets",
    ROOT / "public",
)
PROHIBITED = re.compile(
    r"\b(?:boats?|boating|watercraft|bowrider|moomba|wakeboards?|watersports?)\b|four\s+winns",
    re.IGNORECASE,
)
TEXT_SUFFIXES = {".css", ".html", ".js", ".json", ".rs", ".txt", ".xml"}


class RvOnlyContentTests(unittest.TestCase):
    def test_public_product_content_contains_no_boat_offering_terms(self) -> None:
        failures: list[str] = []
        for root in PUBLIC_CONTENT_ROOTS:
            for path in sorted(root.rglob("*")):
                if not path.is_file() or path.suffix.lower() not in TEXT_SUFFIXES:
                    continue
                text = path.read_text(encoding="utf-8")
                for line_number, line in enumerate(text.splitlines(), start=1):
                    if PROHIBITED.search(line):
                        failures.append(f"{path.relative_to(ROOT)}:{line_number}: {line.strip()}")

        self.assertEqual(failures, [], "Boat-related public content found:\n" + "\n".join(failures))

    def test_brand_logo_is_decorative_when_adjacent_name_supplies_link_text(self) -> None:
        for relative in ("src/components/header.rs", "src/components/footer.rs"):
            source = (ROOT / relative).read_text(encoding="utf-8")
            brand = source[source.index('class: "brand"'):]
            brand = brand[:brand.index("}", brand.index('class: "brand-word"'))]
            self.assertIn('class: "brand-mark", src: LOGO, alt: ""', brand, relative)
            self.assertIn('class: "brand-word"', brand, relative)
            self.assertIn('"VL Rental"', brand, relative)
            self.assertNotIn('alt: "VL Rental"', brand, relative)

    def test_accessible_accent_text_is_scoped_to_light_detail_labels(self) -> None:
        main = (ROOT / "assets/main.css").read_text(encoding="utf-8")
        reviews = (ROOT / "assets/css/rv_detail.css").read_text(encoding="utf-8")
        parks = (ROOT / "assets/css/parks.css").read_text(encoding="utf-8")

        self.assertIn("--vl-accent-text: #8B5B12;", main)
        self.assertIn(
            ".rvd-reviews-heading > span { color: var(--vl-accent-text);",
            reviews,
        )
        self.assertIn(
            ".park-detail-content .parks-eyebrow {\n    color: var(--vl-accent-text);\n}",
            parks,
        )
        self.assertIn(".rvd-review-stars > span.filled { color: var(--vl-accent);", reviews)

    def test_static_rv_fallback_uses_shared_high_priority_cover(self) -> None:
        source = (ROOT / "src/pages/rv_detail.rs").read_text(encoding="utf-8")
        component = source[source.index("fn StaticRvFallback"):]
        component = component[:component.index("fn DynamicRvDetail")]
        self.assertIn("static_fallback_cover(listing.slug", component)
        self.assertIn('fetchpriority: "high"', component)
        self.assertIn('decoding: "async"', component)
        self.assertNotIn("photos.first() {", component)


if __name__ == "__main__":
    unittest.main()
