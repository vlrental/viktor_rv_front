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


if __name__ == "__main__":
    unittest.main()
