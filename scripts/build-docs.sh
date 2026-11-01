#!/usr/bin/env bash
#
# Rebuilds the generated parts of the GitHub Pages site in docs/:
#
#   docs/pkg     the WebAssembly build of the packing engine
#   docs/assets  a copy of examples/assets with a manifest.json
#
#   ./scripts/build-docs.sh
#
# Requires wasm-pack and python3. Pages can only serve files inside docs/, so
# the images are copied. Git stores identical files only once, so the copies
# barely add to the size of the repository.

set -euo pipefail

cd "$(dirname "$0")/.."

wasm-pack build wasm --release --target web --no-pack --out-dir ../docs/pkg
# wasm-pack writes a .gitignore that would keep the build out of the repository.
rm -f docs/pkg/.gitignore

rm -rf docs/assets
mkdir -p docs/assets
cp -R examples/assets/sprite docs/assets/sprite
cp -R examples/assets/charactor-sprite/dj_fox docs/assets/dj_fox
cp -R examples/assets/charactor-sprite/guitar_rabbit docs/assets/guitar_rabbit
find docs/assets -name .DS_Store -delete

python3 - <<'PY'
import json
import struct
from pathlib import Path

ROOT = Path("docs/assets")
SETS = [("sprite", "Sprite"), ("dj_fox", "DJ Fox"), ("guitar_rabbit", "Guitar Rabbit")]


def png_size(path):
    header = path.read_bytes()[:24]
    if header[:8] != b"\x89PNG\r\n\x1a\n":
        raise SystemExit(f"not a PNG file: {path}")
    return struct.unpack(">II", header[16:24])


manifest = {"sets": []}
for directory, label in SETS:
    sprites = []
    for path in sorted((ROOT / directory).glob("*.png")):
        width, height = png_size(path)
        sprites.append({
            "name": path.name,
            "path": f"assets/{directory}/{path.name}",
            "width": width,
            "height": height,
        })
    if not sprites:
        raise SystemExit(f"no images in {ROOT / directory}")
    manifest["sets"].append({"id": directory, "label": label, "sprites": sprites})

(ROOT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
PY

echo "Built docs/. Preview it with: python3 -m http.server -d docs"
