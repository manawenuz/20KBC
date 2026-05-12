#!/usr/bin/env python3
"""
WC3 Asset Extraction Tool
Reads a WC3 MPQ archive, extracts specific BLP textures, converts them to PNG.
"""

import argparse
import ctypes
import fnmatch
import os
import struct
import sys
from io import BytesIO
from pathlib import Path

# Ensure venv packages are available when run directly
_SCRIPT_DIR = Path(__file__).resolve().parent
_VENV_SITE = _SCRIPT_DIR / "venv" / "lib"
if _VENV_SITE.exists():
    # Find the pythonX.Y/site-packages directory
    for sub in _VENV_SITE.rglob("site-packages"):
        if str(sub) not in sys.path:
            sys.path.insert(0, str(sub))

try:
    from PIL import Image
except ImportError as exc:
    print(f"ERROR: Pillow is required. {exc}", file=sys.stderr)
    print("Hint: run 'pip install Pillow' or use the venv.", file=sys.stderr)
    sys.exit(1)


# ---------------------------------------------------------------------------
# StormLib bindings
# ---------------------------------------------------------------------------

_LIB_PATHS = [
    "/opt/homebrew/lib/libstorm.dylib",
    "/usr/local/lib/libstorm.dylib",
    "libstorm.dylib",
    "libstorm.so",
]

_storm = None
for _lp in _LIB_PATHS:
    try:
        _storm = ctypes.CDLL(_lp)
        break
    except OSError:
        continue

if _storm is None:
    print(
        "ERROR: StormLib not found. Install it (e.g. 'brew install stormlib').",
        file=sys.stderr,
    )
    sys.exit(1)


class _SFILE_FIND_DATA(ctypes.Structure):
    _fields_ = [
        ("cFileName", ctypes.c_char * 1024),
        ("szPlainName", ctypes.c_char_p),
        ("dwHashIndex", ctypes.c_uint32),
        ("dwBlockIndex", ctypes.c_uint32),
        ("dwFileSize", ctypes.c_uint32),
        ("dwFileFlags", ctypes.c_uint32),
        ("dwCompSize", ctypes.c_uint32),
        ("dwFileTimeLo", ctypes.c_uint32),
        ("dwFileTimeHi", ctypes.c_uint32),
        ("lcLocale", ctypes.c_uint32),
    ]


_storm.SFileOpenArchive.argtypes = [
    ctypes.c_char_p,
    ctypes.c_uint32,
    ctypes.c_uint32,
    ctypes.POINTER(ctypes.c_void_p),
]
_storm.SFileOpenArchive.restype = ctypes.c_bool

_storm.SFileCloseArchive.argtypes = [ctypes.c_void_p]
_storm.SFileCloseArchive.restype = ctypes.c_bool

_storm.SFileHasFile.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
_storm.SFileHasFile.restype = ctypes.c_bool

_storm.SFileOpenFileEx.argtypes = [
    ctypes.c_void_p,
    ctypes.c_char_p,
    ctypes.c_uint32,
    ctypes.POINTER(ctypes.c_void_p),
]
_storm.SFileOpenFileEx.restype = ctypes.c_bool

_storm.SFileGetFileSize.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint32)]
_storm.SFileGetFileSize.restype = ctypes.c_uint32

_storm.SFileReadFile.argtypes = [
    ctypes.c_void_p,
    ctypes.c_void_p,
    ctypes.c_uint32,
    ctypes.POINTER(ctypes.c_uint32),
    ctypes.c_void_p,
]
_storm.SFileReadFile.restype = ctypes.c_bool

_storm.SFileCloseFile.argtypes = [ctypes.c_void_p]
_storm.SFileCloseFile.restype = ctypes.c_bool

_storm.SFileFindFirstFile.argtypes = [
    ctypes.c_void_p,
    ctypes.c_char_p,
    ctypes.POINTER(_SFILE_FIND_DATA),
    ctypes.c_char_p,
]
_storm.SFileFindFirstFile.restype = ctypes.c_void_p

_storm.SFileFindNextFile.argtypes = [ctypes.c_void_p, ctypes.POINTER(_SFILE_FIND_DATA)]
_storm.SFileFindNextFile.restype = ctypes.c_bool

_storm.SFileFindClose.argtypes = [ctypes.c_void_p]
_storm.SFileFindClose.restype = ctypes.c_bool

STREAM_FLAG_READ_ONLY = 0x00000100


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _normalize_path(path: str) -> str:
    """Convert forward slashes to backslashes for MPQ internal paths."""
    return path.replace("/", "\\")


def _list_candidates(h_mpq: ctypes.c_void_p, pattern: str) -> list:
    """List all files in the MPQ matching a wildcard pattern."""
    results = []
    find_data = _SFILE_FIND_DATA()
    h_find = _storm.SFileFindFirstFile(h_mpq, pattern, ctypes.byref(find_data), None)
    if not h_find:
        return results
    try:
        while True:
            name = find_data.cFileName.decode("latin-1").replace("\\", "/")
            results.append(name)
            if not _storm.SFileFindNextFile(h_find, ctypes.byref(find_data)):
                break
    finally:
        _storm.SFileFindClose(h_find)
    return results


def _read_mpq_file(h_mpq: ctypes.c_void_p, mpq_path: str) -> bytes:
    """Read the full contents of a file from the MPQ archive."""
    h_file = ctypes.c_void_p()
    if not _storm.SFileOpenFileEx(h_mpq, mpq_path.encode("latin-1"), 0, ctypes.byref(h_file)):
        raise RuntimeError(f"Failed to open '{mpq_path}' inside MPQ")
    try:
        size = _storm.SFileGetFileSize(h_file, None)
        if size == 0xFFFFFFFF:
            raise RuntimeError(f"Failed to get size of '{mpq_path}'")
        buf = ctypes.create_string_buffer(size)
        read = ctypes.c_uint32(0)
        if not _storm.SFileReadFile(h_file, buf, size, ctypes.byref(read), None):
            raise RuntimeError(f"Failed to read '{mpq_path}'")
        return buf.raw[: read.value]
    finally:
        _storm.SFileCloseFile(h_file)


def _blp_to_png(blp_data: bytes) -> bytes:
    """Convert BLP bytes to PNG bytes using Pillow's BLP plugin."""
    img = Image.open(BytesIO(blp_data))
    out = BytesIO()
    img.save(out, format="PNG")
    return out.getvalue()


def _pick_fallback(candidates: list, keyword: str) -> str | None:
    """Pick the first candidate whose basename contains the keyword (case-insensitive)."""
    keyword_lower = keyword.lower()
    for c in candidates:
        basename = os.path.basename(c).lower()
        if keyword_lower in basename:
            return c
    return candidates[0] if candidates else None


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> int:
    parser = argparse.ArgumentParser(description="Extract BLP textures or MDX models from a WC3 MPQ.")
    parser.add_argument("--mpq", required=True, help="Path to the MPQ archive.")
    parser.add_argument("--out", required=True, help="Output directory for PNG/GLB files.")
    parser.add_argument(
        "--files",
        nargs="+",
        required=False,
        help='File mappings in the form "mpq/path.blp:output.png"',
    )
    parser.add_argument(
        "--mdx",
        nargs="*",
        required=False,
        help='MDX file mappings in the form "mpq/path.mdx:output.glb". If given without values, extracts default models.',
    )
    args = parser.parse_args()

    if not args.files and args.mdx is None:
        parser.error("One of --files or --mdx is required.")

    mpq_path = Path(args.mpq)
    if not mpq_path.exists():
        print(f"ERROR: MPQ file not found: {mpq_path}", file=sys.stderr)
        return 1

    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    h_mpq = ctypes.c_void_p()
    if not _storm.SFileOpenArchive(str(mpq_path).encode("utf-8"), 0, STREAM_FLAG_READ_ONLY, ctypes.byref(h_mpq)):
        print(f"ERROR: Failed to open MPQ archive: {mpq_path}", file=sys.stderr)
        return 1

    extracted = []
    try:
        if args.mdx is not None:
            # Lazy import to avoid hard dependency on mdx_to_gltf
            try:
                from mdx_to_gltf import parse_mdx, write_glb
            except ImportError:
                print("ERROR: mdx_to_gltf module not found.", file=sys.stderr)
                return 1

            # Default mappings when --mdx is used without explicit values
            mappings = args.mdx if args.mdx else [
                "units/human/peasant/peasant.mdx:peasant.glb",
                # TimberWolf is the hostile creep variant — BrownWolf is a
                # passive critter and doesn't fit a GAIA threat that hunts
                # workers.
                "units/creeps/timberwolf/timberwolf.mdx:wolf.glb",
                "doodads/terrain/ashentree/ashentree0.mdx:ashentree.glb",
                "doodads/terrain/rockchunks/rockchunks0.mdx:rockchunks.glb",
            ]

            for mapping in mappings:
                if ":" not in mapping:
                    print(f"ERROR: Invalid MDX mapping (missing colon): {mapping}", file=sys.stderr)
                    continue
                src, dst_name = mapping.split(":", 1)
                mpq_internal = _normalize_path(src)
                dst_path = out_dir / dst_name
                try:
                    mdx_bytes = _read_mpq_file(h_mpq, mpq_internal)
                    mdx_parsed = parse_mdx(mdx_bytes)
                    write_glb(mdx_parsed, dst_path, h_mpq=h_mpq)
                    extracted.append((mpq_internal.replace("\\", "/"), dst_name, dst_path.stat().st_size))
                    print(f"  {mpq_internal.replace('\\', '/')} -> {dst_name} ({dst_path.stat().st_size} bytes)")
                except Exception as exc:
                    print(f"  WARNING: Failed to extract {mpq_internal}: {exc}", file=sys.stderr)

        for mapping in (args.files or []):
            if ":" not in mapping:
                print(f"ERROR: Invalid mapping (missing colon): {mapping}", file=sys.stderr)
                return 1

            src, dst_name = mapping.split(":", 1)
            mpq_internal = _normalize_path(src)
            dst_path = out_dir / dst_name

            # 1. Try exact path
            found = _storm.SFileHasFile(h_mpq, mpq_internal.encode("latin-1"))
            chosen = mpq_internal if found else None

            # 2. Fallback logic
            if not found:
                # Determine fallback keyword from the original requested filename
                basename = os.path.basename(src)
                keyword = ""
                if "Dirt" in basename:
                    keyword = "Dirt"
                elif "Grass" in basename:
                    keyword = "Grass"
                else:
                    keyword = os.path.splitext(basename)[0]

                candidates = _list_candidates(h_mpq, b"TerrainArt\\*.blp")
                candidates_norm = [c.replace("\\", "/") for c in candidates]
                matches = [c for c in candidates_norm if fnmatch.fnmatch(c.lower(), "terrainart/**/*.blp")]

                print(f"Requested '{src}' not found. Listing TerrainArt/**/*.blp candidates:", file=sys.stderr)
                for c in matches[:20]:
                    print(f"  {c}", file=sys.stderr)
                if len(matches) > 20:
                    print(f"  ... and {len(matches) - 20} more", file=sys.stderr)

                fallback = _pick_fallback(matches, keyword)
                if fallback:
                    chosen = _normalize_path(fallback)
                    print(f"Fallback selected: {fallback}", file=sys.stderr)
                else:
                    print(f"ERROR: No fallback found for '{src}'", file=sys.stderr)
                    return 1

            # 3. Extract and convert
            blp_data = _read_mpq_file(h_mpq, chosen)
            png_data = _blp_to_png(blp_data)
            dst_path.write_bytes(png_data)

            extracted.append((chosen.replace("\\", "/"), dst_name, len(png_data)))
            print(f"  {chosen.replace('\\', '/')} -> {dst_name} ({len(png_data)} bytes)")
    finally:
        _storm.SFileCloseArchive(h_mpq)

    # Summary
    print(f"\nExtracted {len(extracted)} file(s):")
    for src, dst, size in extracted:
        print(f"  {src} -> {dst} ({size} bytes)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
