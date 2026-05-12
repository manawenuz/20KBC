# PRD-31 — Runtime BLP Decoder (Rust)

## Goal

Decode WC3 `.blp` (Blizzard Picture) bytes into RGBA8 pixels at runtime
so the GDExtension can build Godot `ImageTexture`s on demand. No more
offline BLP→PNG conversion.

## Files you MAY create

- `spikes/godot-gdext/rust/src/blp/mod.rs`

## Files you MAY modify

- `spikes/godot-gdext/rust/Cargo.toml` — add `blp = "0.1"` (the crates.io
  `blp` reader/writer) and `image = { version = "0.25", default-features = false, features = ["jpeg"] }`
  (for BLP1's JPEG-compressed mipmaps). Nothing else.
- `spikes/godot-gdext/rust/src/lib.rs` — add `mod blp;` only.

## Files you MUST NOT touch

- `scripts/asset-extract/**`
- `game-core/**`
- Other Rust source

## Interface contract

```rust
// blp/mod.rs
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,  // tightly packed RGBA8
}

/// Decode a BLP byte buffer (BLP1 JPEG or paletted, BLP2 DXT or paletted)
/// into RGBA8 mipmap level 0. Returns Err with a short reason on failure.
pub fn decode_blp(bytes: &[u8]) -> Result<DecodedImage, String>;
```

If the `blp` crate handles everything, great. If it only handles BLP2
and we need BLP1 (WC3 1.27 uses BLP1), fall back to a hand-rolled
parser: 28-byte header, then offsets/sizes table, then mipmap data
(JPEG bytes if `compression == 0`, raw paletted if `compression == 1`).

## Acceptance criteria

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn decode_peasant_blp() {
        // Use the offline-extracted PNG's BLP source via the on-disk MPQ
        // OR check in a small reference .blp file under tests/data/.
        let data = fs::read("/Volumes/samGames/WC3/test_peasant.blp").ok();
        if let Some(bytes) = data {
            let img = decode_blp(&bytes).unwrap();
            assert!(img.width > 0 && img.height > 0);
            assert_eq!(img.rgba.len(), (img.width * img.height * 4) as usize);
        }
    }
}
```

If pulling a BLP from disk is awkward in tests, decode it inline by
reading from War3.mpq via PRD-30 — but PRD-30 may not be merged yet,
so a static test fixture under
`spikes/godot-gdext/rust/tests/data/Peasant.blp` is acceptable.
Commit a small (≤ 256x256) BLP fixture.

- [ ] `cargo test -p spike-godot-gdext blp` passes
- [ ] `cargo build` clean
- [ ] ≤ 4 files modified

## Out of scope

- Mipmap levels > 0
- BLP encoding
- DXT5 specifically (DXT1 + paletted cover WC3 1.27)
- Animated palettes
