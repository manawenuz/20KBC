# PRD-30 — Runtime MPQ Reader (Rust)

## Goal

Replace the offline Python + StormLib MPQ extraction with a **runtime**
Rust reader that the Godot GDExtension can call to load files from
`War3.mpq` (and friends) on demand. First Warsmash-style pivot piece.

## Files you MAY create

- `spikes/godot-gdext/rust/src/datasource/mod.rs`
- `spikes/godot-gdext/rust/src/datasource/mpq.rs`
- `spikes/godot-gdext/rust/src/datasource/compound.rs`

## Files you MAY modify

- `spikes/godot-gdext/rust/Cargo.toml` — add **one** dep:
  `wow-mpq = "0.6"` (or `stormlib-rs` if `wow-mpq` can't open WC3 1.27
  archives; document choice in the module). Nothing else.
- `spikes/godot-gdext/rust/src/lib.rs` — add `mod datasource;` only.

## Files you MUST NOT touch

- `scripts/asset-extract/**` — old offline path stays for now
- `game-core/**`
- `main.gd`, `Main.tscn`, `project.godot`
- Anything else under `spikes/`

## Interface contract

Mirror Warsmash's `DataSource` interface (`docs/WC3Analysis/02_asset_loading.md`):

```rust
// datasource/mod.rs
pub trait DataSource: Send + Sync {
    fn has(&self, path: &str) -> bool;
    fn read(&self, path: &str) -> Option<Vec<u8>>;
}

pub use mpq::MpqDataSource;
pub use compound::CompoundDataSource;
```

```rust
// datasource/mpq.rs
pub struct MpqDataSource { /* wraps wow-mpq archive handle */ }
impl MpqDataSource {
    pub fn open(path: &Path) -> Result<Self, String>;
}
impl DataSource for MpqDataSource { ... }
```

```rust
// datasource/compound.rs — layered like Warsmash's CompoundDataSource
pub struct CompoundDataSource { sources: Vec<Box<dyn DataSource>> }
impl CompoundDataSource {
    pub fn new() -> Self;
    pub fn add(&mut self, source: Box<dyn DataSource>);
}
impl DataSource for CompoundDataSource { ... }
```

Path normalization: accept `Units/Human/Peasant/peasant.mdx` (forward
slashes); convert internally to backslashes for MPQ lookup.
Case-insensitive lookup.

## Acceptance criteria

Add a `#[cfg(test)] mod tests` block proving:

```rust
let mpq = MpqDataSource::open(Path::new("/Volumes/samGames/WC3/War3.mpq")).unwrap();
assert!(mpq.has("Units/Human/Peasant/peasant.mdx"));
let bytes = mpq.read("Units/Human/Peasant/peasant.mdx").unwrap();
assert!(bytes.starts_with(b"MDLX"));
assert!(bytes.len() > 100_000);
```

Then `cargo test -p spike-godot-gdext datasource` from
`spikes/godot-gdext/rust/`. Skip the test with `#[cfg(target_os = "macos")]`
if the MPQ path isn't present (CI safety).

- [ ] Test passes locally
- [ ] `cargo build` clean
- [ ] No #[func] / GDExtension class exposed yet (PRD-32 wires it)
- [ ] ≤ 5 files modified

## Out of scope

- File listing / wildcards (read-by-path is enough for Warsmash flow)
- Write support
- CASC (Patch 1.32 Reforged) — MPQ only
- Async loading
