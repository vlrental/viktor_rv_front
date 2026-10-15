#!/usr/bin/env python3
"""Build full-admin and lean-storefront Dioxus artifacts from one source tree."""

from __future__ import annotations

import argparse
import shutil
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DX_ROOT = ROOT / "target/dx/viktor_rv_front/release/web"
PUBLIC = DX_ROOT / "public"
ADMIN = DX_ROOT / "admin-public"


def run_build(base_path: str, storefront: bool) -> None:
    command = ["dx", "build", "--release", "--web", "--debug-symbols", "false"]
    if storefront:
        command += ["--features", "storefront_bundle"]
    if base_path:
        command += ["--base-path", base_path]
    subprocess.run(command, cwd=ROOT, check=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base-path", default="")
    args = parser.parse_args()
    for path in (PUBLIC, ADMIN):
        if path.exists():
            shutil.rmtree(path)
    run_build(args.base_path, storefront=False)
    shutil.move(PUBLIC, ADMIN)
    run_build(args.base_path, storefront=True)
    if not PUBLIC.is_dir() or not ADMIN.is_dir():
        raise RuntimeError("both Pages build variants are required")
    print(f"Built storefront {PUBLIC} and full admin {ADMIN}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
