"""Exclusive source-only copy to an explicitly chosen NEW Windows directory.

Run with Python 3 under WSL. No root traversal, secret file reading, symlink
following, Linux dependencies, output bundles or note data. No mirror/delete.
"""
import argparse
import hashlib
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parent.parent
FILES = (
    "package.json", "package-lock.json", "index.html", "tsconfig.json",
    "vite.config.ts", "vitest.config.ts", "rust-toolchain.toml",
    "src-tauri/Cargo.toml", "src-tauri/Cargo.lock", "src-tauri/build.rs",
    "src-tauri/tauri.conf.json", "tooling/windows-native.ps1",
    "tooling/empty-env/.gitkeep",
)
TREES = {
    "src": {".ts", ".css"},
    "tests": {".ts"},
    "src-tauri/src": {".rs"},
    "src-tauri/tests": {".rs"},
    "src-tauri/examples": {".rs"},
    "src-tauri/capabilities": {".json"},
    "src-tauri/icons": {".png", ".ico", ".svg"},
}


def source_files():
    paths = [ROOT / name for name in FILES]
    for tree, extensions in TREES.items():
        directory = ROOT / tree
        for path in directory.rglob("*"):
            if path.is_symlink():
                raise ValueError(f"Symlink interdit : {path.relative_to(ROOT)}")
            if path.is_file() and path.suffix in extensions:
                paths.append(path)
    for path in paths:
        relative = path.relative_to(ROOT)
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"Source régulière requise : {relative}")
        if any(part.startswith(".env") or part.lower() in {
            "target", "node_modules", "attachments", "secrets", "notes"
        } for part in relative.parts) or "key" in path.name.lower():
            raise ValueError(f"Nom exclu : {relative}")
    return sorted(set(paths))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    # Restrict the target to the actual Windows user's dedicated development area.
    parent = Path("/mnt/c/Users/lmirguet/Development")
    destination = args.destination
    if destination.parent != parent or not destination.name.startswith("vocalnotes-native-"):
        raise ValueError("Cible attendue : /mnt/c/Users/lmirguet/Development/vocalnotes-native-…")
    paths = source_files()
    parent.mkdir(exist_ok=True)
    destination.mkdir(exist_ok=False)  # never overwrite an unknown/existing copy
    for path in paths:
        relative = path.relative_to(ROOT)
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
        if hashlib.sha256(path.read_bytes()).digest() != hashlib.sha256(target.read_bytes()).digest():
            raise ValueError(f"Copie différente : {relative}")
    print(f"COPY PASS: {len(paths)} fichiers vérifiés, cible={destination}")
    print("Exclus : clé, pièces jointes, .env, notes, node_modules, target, dist.")


if __name__ == "__main__":
    main()
