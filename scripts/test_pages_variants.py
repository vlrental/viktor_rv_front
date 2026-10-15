from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from scripts.manage_pages_asset_cache import MARKER, retained
from scripts.prepare_pages_variants import merge_tree
from scripts.verify_pages_artifact import verify_bundle_split


class PagesVariantTests(unittest.TestCase):
    def test_full_asset_closure_merges_and_rejects_byte_collisions(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            destination = root / "destination"
            (source / "nested").mkdir(parents=True)
            destination.mkdir()
            (source / "nested/full.js").write_bytes(b"full")
            merge_tree(source, destination)
            self.assertEqual((destination / "nested/full.js").read_bytes(), b"full")
            (source / "shared.css").write_bytes(b"full")
            (destination / "shared.css").write_bytes(b"storefront")
            with self.assertRaisesRegex(RuntimeError, "different bytes"):
                merge_tree(source, destination)

    def test_v2_cache_retains_only_recorded_current_union(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "assets").mkdir()
            (root / "assets/current.js").write_bytes(b"current")
            (root / "assets/stale.js").write_bytes(b"stale")
            (root / MARKER).write_text(
                json.dumps({"version": 2, "current_union": ["assets/current.js"]}),
                encoding="utf-8",
            )
            self.assertEqual(retained(root), ["assets/current.js"])

    def test_flat_v1_cache_retains_its_complete_single_generation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "app-dxha.js").write_bytes(b"js")
            (root / "app_bg-dxhb.wasm").write_bytes(b"wasm")
            (root / ".vl-pages-generation-v1").touch()
            self.assertEqual(retained(root), ["assets/app-dxha.js", "assets/app_bg-dxhb.wasm"])

    def test_cache_cli_migrates_flat_v1_then_bounds_next_v2_generation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            script = Path(__file__).with_name("manage_pages_asset_cache.py")
            artifact = root / "artifact"
            flat_v1 = root / "flat-v1"
            marker = root / "current.json"
            staged_v2 = root / "staged-v2"
            next_artifact = root / "next-artifact"
            next_marker = root / "next.json"

            (artifact / "assets").mkdir(parents=True)
            (artifact / "assets/storefront.js").write_bytes(b"storefront-v1")
            (artifact / "assets/storefront.wasm").write_bytes(b"storefront-wasm-v1")
            flat_v1.mkdir()
            (flat_v1 / "legacy.js").write_bytes(b"legacy-js")
            (flat_v1 / "legacy.wasm").write_bytes(b"legacy-wasm")
            (flat_v1 / ".vl-pages-generation-v1").touch()

            def run(*arguments: Path | str) -> None:
                subprocess.run(
                    [sys.executable, str(script), *(str(argument) for argument in arguments)],
                    check=True,
                )

            run("snapshot", artifact, marker)
            run("merge", flat_v1, artifact)
            self.assertEqual((artifact / "assets/legacy.js").read_bytes(), b"legacy-js")
            self.assertEqual((artifact / "assets/legacy.wasm").read_bytes(), b"legacy-wasm")
            run("stage", marker, artifact, staged_v2)
            self.assertFalse((staged_v2 / "assets/legacy.js").exists())

            (next_artifact / "assets").mkdir(parents=True)
            (next_artifact / "assets/storefront-v2.js").write_bytes(b"storefront-v2")
            (next_artifact / "assets/storefront-v2.wasm").write_bytes(b"storefront-wasm-v2")
            run("snapshot", next_artifact, next_marker)
            run("merge", staged_v2, next_artifact)
            self.assertEqual((next_artifact / "assets/storefront-v2.js").read_bytes(), b"storefront-v2")
            self.assertEqual((next_artifact / "assets/storefront.js").read_bytes(), b"storefront-v1")
            self.assertFalse((next_artifact / "assets/legacy.js").exists())

    def test_verifier_requires_distinct_admin_and_storefront_bundles(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "admin").mkdir()
            (root / "assets").mkdir()
            storefront = '<script src="/assets/viktor_rv_front-dxha.js"></script>'
            admin = '<script src="/assets/viktor_rv_front-dxhc.js"></script>'
            (root / "assets/viktor_rv_front-dxha.js").write_text('"viktor_rv_front_bg-dxhb.wasm"', encoding="utf-8")
            (root / "assets/viktor_rv_front-dxhc.js").write_text('"viktor_rv_front_bg-dxhd.wasm"', encoding="utf-8")
            (root / "assets/viktor_rv_front_bg-dxhb.wasm").write_bytes(b"storefront")
            (root / "assets/viktor_rv_front_bg-dxhd.wasm").write_bytes(b"admin")
            (root / "index.html").write_text(storefront, encoding="utf-8")
            (root / "admin/index.html").write_text(admin, encoding="utf-8")
            self.assertEqual(
                verify_bundle_split(root, [root / "index.html", root / "admin/index.html"]),
                [],
            )
            (root / "admin/index.html").write_text(storefront, encoding="utf-8")
            self.assertTrue(verify_bundle_split(root, [root / "index.html", root / "admin/index.html"]))

    def test_verifier_rejects_missing_storefront_or_admin_wasm(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "admin").mkdir()
            (root / "assets").mkdir()
            (root / "index.html").write_text('<script src="/assets/viktor_rv_front-dxha.js"></script>', encoding="utf-8")
            (root / "admin/index.html").write_text('<script src="/assets/viktor_rv_front-dxhc.js"></script>', encoding="utf-8")
            (root / "assets/viktor_rv_front-dxha.js").write_text('"viktor_rv_front_bg-dxhb.wasm"', encoding="utf-8")
            (root / "assets/viktor_rv_front-dxhc.js").write_text('"viktor_rv_front_bg-dxhd.wasm"', encoding="utf-8")
            (root / "assets/viktor_rv_front_bg-dxhb.wasm").write_bytes(b"storefront")
            failures = verify_bundle_split(root, [root / "index.html", root / "admin/index.html"])
            self.assertIn("admin bundle references missing asset: assets/viktor_rv_front_bg-dxhd.wasm", failures)
            (root / "assets/viktor_rv_front_bg-dxhd.wasm").write_bytes(b"admin")
            (root / "assets/viktor_rv_front_bg-dxhb.wasm").unlink()
            failures = verify_bundle_split(root, [root / "index.html", root / "admin/index.html"])
            self.assertIn("storefront bundle references missing asset: assets/viktor_rv_front_bg-dxhb.wasm", failures)

    def test_admin_boundary_is_hard_navigation_with_retry_and_loop_guard(self) -> None:
        project = Path(__file__).resolve().parents[1]
        header = (project / "src/components/header.rs").read_text(encoding="utf-8")
        admin = (project / "src/pages/admin.rs").read_text(encoding="utf-8")
        self.assertEqual(header.count('href: api::frontend_path("/admin")'), 2)
        self.assertNotIn('to: Route::Admin {}', header)
        transition = admin.split("async fn redirect_storefront_admin", 1)[1].split("async fn navigate_evidence_preview", 1)[0]
        self.assertIn("performance.getEntriesByType('navigation')", transition)
        self.assertIn("initial.pathname", transition)
        storefront = admin.split('if cfg!(feature = "storefront_bundle")', 1)[1].split("let navigator", 1)[0]
        self.assertIn("Opening the secure admin dashboard…", storefront)
        self.assertIn("The admin dashboard did not open.", storefront)
        self.assertIn("Try again", storefront)
        self.assertEqual(storefront.count("redirect_storefront_admin(false).await"), 1)
        self.assertEqual(storefront.count("redirect_storefront_admin(true).await"), 1)
        self.assertIn("if (!{force}", transition)


if __name__ == "__main__":
    unittest.main()
