# PRD-04 — WC3 Asset Extraction Tool

## Goal

Build a standalone, scriptable tool that reads a WC3 MPQ archive, extracts a
specific list of BLP textures, converts them to PNG, and writes them under
`spikes/godot-gdext/assets/textures/`. This is the first slice of the asset
pipeline described in `docs/WC3Analysis/02_asset_loading.md` and
`docs/WC3Analysis/03_file_formats.md`.

For batch 1 we extract **2 specific files**:

1. `TerrainArt/LordaeronSummer/Lords_Dirt.blp` → `textures/ground_dirt.png`
2. `TerrainArt/LordaeronSummer/Lords_Grass.blp` → `textures/ground_grass.png`

If those exact paths are not present in the listfile, fall back to:

- Any `TerrainArt/**/*Dirt*.blp` → `ground_dirt.png`
- Any `TerrainArt/**/*Grass*.blp` → `ground_grass.png`

(WC3 path separator is backslash internally; normalize to forward slash for lookup.)

## Context

- MPQ files live at `/Volumes/samGames/WC3/` (War3.mpq is the base game archive).
- `docs/WC3Analysis/02_asset_loading.md` documents the MPQ format, hash table,
  and compression algorithms (zlib, BZip2, PKWARE explode, Huffman, ADPCM).
- `docs/WC3Analysis/03_file_formats.md` documents the BLP format spec (BLP1
  uses JPEG-compressed mipmaps; BLP2 uses S3TC/DXT or raw paletted).
- BLP1 (`MagicNumber == 'BLP1'`) is what WC3 1.27 uses.

## Tool choice (decide based on ecosystem)

You may choose **either**:

**Option A — Python tool** (preferred for speed of implementation):
- Create `scripts/asset-extract/` directory
- Use existing Python MPQ library: `pip install storm-py` OR `mpyq`
- Use `Pillow` + custom BLP decoder, OR find a `blp` PyPI package
- Entry point: `scripts/asset-extract/extract.py`

**Option B — Rust tool**:
- Create new crate `tools/asset-extract/` with `Cargo.toml`
- Use `mpq` crate or write a minimal MPQ reader
- BLP decode: use `image` crate + custom BLP parser (BLP1 = JPEG mipmaps)
- Entry point: `cargo run --bin asset-extract -- --mpq <path> --out <dir>`

Either is fine. Pick the one with the most existing library support.

## Files you MAY create

- Any new files under `scripts/asset-extract/` (Python) or `tools/asset-extract/` (Rust)
- Any new files under `spikes/godot-gdext/assets/textures/` (the output PNGs)

## Files you MAY modify

- `Cargo.toml` (root) — only if you chose Rust and need to register a workspace member.
  Append to `[workspace] members`, do not change other fields.

## Files you MUST NOT touch

- `spikes/godot-gdext/rust/**`
- `game-core/**`
- `spikes/godot-gdext/scenes/`, `scripts/`, `project.godot`
- `prds/`, `plans/`, `docs/`

## Interface contract

The tool must be runnable as:

```bash
# From repo root
scripts/asset-extract/extract.py \
    --mpq /Volumes/samGames/WC3/War3.mpq \
    --out spikes/godot-gdext/assets/textures \
    --files "TerrainArt/LordaeronSummer/Lords_Dirt.blp:ground_dirt.png" \
            "TerrainArt/LordaeronSummer/Lords_Grass.blp:ground_grass.png"
```

OR (if Rust):

```bash
cargo run --bin asset-extract -- --mpq /Volumes/samGames/WC3/War3.mpq \
    --out spikes/godot-gdext/assets/textures \
    --files "TerrainArt/LordaeronSummer/Lords_Dirt.blp:ground_dirt.png" \
            "TerrainArt/LordaeronSummer/Lords_Grass.blp:ground_grass.png"
```

If the requested paths don't exist, the tool must list `TerrainArt/**/*.blp`
candidates to stderr and pick a sensible fallback automatically.

## Acceptance criteria

Run from repo root:

```bash
# Whichever entry point you built — must exit 0:
scripts/asset-extract/extract.py --mpq /Volumes/samGames/WC3/War3.mpq \
    --out spikes/godot-gdext/assets/textures \
    --files "TerrainArt/LordaeronSummer/Lords_Dirt.blp:ground_dirt.png" \
            "TerrainArt/LordaeronSummer/Lords_Grass.blp:ground_grass.png"

# And then:
ls -la spikes/godot-gdext/assets/textures/ground_dirt.png \
       spikes/godot-gdext/assets/textures/ground_grass.png
file spikes/godot-gdext/assets/textures/ground_dirt.png   # must say "PNG image data"
```

- [ ] Both PNG files exist, are valid PNGs, non-zero size
- [ ] Tool prints a short summary to stdout (file, src→dst, size in bytes)
- [ ] No files modified outside the whitelist
- [ ] If `/Volumes/samGames/WC3/War3.mpq` doesn't exist on the test machine,
      the tool exits with a clear error message and code != 0 — but you should
      still be able to develop and test against it on the dev machine.

## Notes on existing libraries

- Python `mpyq` (BSD, pure Python): handles MPQ v1, can extract files by name.
  https://github.com/eagleflo/mpyq — works for WC3 1.27 archives.
- Python BLP decoders are rare; `python-blp` or hand-rolling BLP1 (JPEG mipmaps)
  is fine. BLP1 spec: 28-byte header, then offsets/sizes table, then mipmap data.
  For JPEG variant: header byte 4 = compression type (0=JPEG, 1=raw paletted).
  Mipmap 0 is the full-resolution image.
- Rust: the `stormlib-rs` crate wraps StormLib, or use the pure-Rust `mpq` crate.

## Out of scope

- Loading the PNGs in Godot — orchestrator handles after merge
- MDX model extraction — separate later PRD
- A general-purpose asset CLI — just enough to extract the 2 textures
- A runtime MPQ loader in the gdext layer — explicitly chose offline pipeline
