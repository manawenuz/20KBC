# 02 — Asset Loading Pipeline

## Dev System Asset Locations

```
/Volumes/samGames/WC3/          ← external drive (WD or Samsung, mounted at boot)
  War3.mpq                       base game (RoC assets)
  War3Local.mpq                  localized strings/audio (English)
  War3x.mpq                      Frozen Throne expansion assets
  War3xLocal.mpq                 TFT localized strings/audio
  Deprecated.mpq                 deprecated assets (still needed for some models)
  Maps/                          custom map folder

/Users/manwe/CascadeProjects/WarsmashModEngine/
  core/assets/warsmash.ini       config file (edit this to change paths)
  resources/                     Warsmash-specific override assets
    (UI tweaks, missing textures, custom FDF overlays)
```

**Priority order:** index 00 is searched first. Index 05 (`resources/`) and index 07 (`.`) are plain folders and serve as override layers. Last-defined wins for any given path.

---

## warsmash.ini DataSources Block

```ini
[DataSources]
Count=9
Type00=MPQ    Path00="/Volumes/samGames/WC3/War3.mpq"
Type01=MPQ    Path01="/Volumes/samGames/WC3/War3Local.mpq"
Type02=MPQ    Path02="/Volumes/samGames/WC3/War3x.mpq"
Type03=MPQ    Path03="/Volumes/samGames/WC3/War3xLocal.mpq"
Type04=MPQ    Path04="/Volumes/samGames/WC3/Deprecated.mpq"
Type05=Folder Path05="../../resources"
Type06=Folder Path06="/Volumes/samGames/WC3/Maps"
Type07=Folder Path07="."
Type08=Folder Path08="/Volumes/samGames/WC3/"
```

Relative paths are resolved from the working directory, which is `desktop/assets/` when running via Gradle.

**Type values:** `MPQ`, `Folder`, `CASC`

---

## DataSource Interface

```java
// core/src/com/etheller/warsmash/datasources/DataSource.java
interface DataSource {
    boolean has(String filepath);
    InputStream getResourceAsStream(String filepath);
    File getFile(String filepath);          // extracts to temp file
    ByteBuffer read(String path);           // fully buffered
    Collection<String> getListfile();       // all known paths
}
```

All engine code uses only this interface. Never calls `new File(...)` or `FileInputStream` directly.

---

## Implementations

### CompoundDataSource
```
core/src/com/etheller/warsmash/datasources/CompoundDataSource.java
```
- Holds `List<DataSource>` in insertion order
- `has()`: iterates list, returns true if any source has the file
- `getResourceAsStream()`: returns stream from first source that has the file
- Merges listfiles: `Set<String> getMergedListfile()`
- Maintains `Map<String, File> tempFileCache` so extracted files aren't re-extracted

### MpqDataSource
```
core/src/com/etheller/warsmash/datasources/MpqDataSource.java
core/src/mpq/  ← DrSuperGood's MPQ library
```
- Opens MPQ via `MPQArchive(File)` — memory-maps the archive
- File lookup: `HashLookup.getFileIndex(filename)` → block table index
- Decompresses block chain into `byte[]`
- Compression types (identified by first byte of each block):
  - `0x02` — zlib (Deflate)
  - `0x08` — BZip2
  - `0x40` — ADPCM mono audio
  - `0x80` — ADPCM stereo audio
  - `0x10` — PKWARE Data Compression Library (explode)
  - `0x01` — Huffman

```
core/src/mpq/compression/
  pkware/   PKWare explode
  huffman/  Huffman decode
  adpcm/    ADPCM audio decode
```

**MPQ hash function (for reimplementing lookup):**
```
hash = 0xEEEEEEEE
for each char c in filename (uppercase):
    hash = (hash + cryptTable[0x100 + c]) XOR (hash << 5) XOR c
```
Three hash types (0, 1, 2) used to verify file identity.

### FolderDataSource
```
core/src/com/etheller/warsmash/datasources/FolderDataSource.java
```
- Wraps a `java.nio.file.Path`
- `has(path)`: `Files.exists(root.resolve(path))`
- Path separator: always forward slash internally, normalized to OS separator

### CascDataSource
```
core/src/com/etheller/warsmash/datasources/CascDataSource.java
```
Used for Patch 1.32 (Reforged). Opens Blizzard's CASC (Content Addressable Storage Container) format. Not needed for Patch 1.27 MPQ setup.

---

## File Resolution Rules

1. Paths are **case-insensitive** on lookup (MPQ filenames are uppercase internally)
2. Path separator is always `\` inside MPQ; Warsmash normalizes to `/` before lookup
3. `ModelViewer.load()` applies a fallback: if `file.blp` not found, tries `file.dds`
4. Missing files print to stderr but do not crash: `System.err.println("Attempting to load non-existent file: " + finalSrc)`

```java
// ModelViewer.java — DDS fallback
if (!this.dataSource.has(finalSrc)) {
    final String ddsPath = finalSrc.substring(0, finalSrc.lastIndexOf('.')) + ".dds";
    if (this.dataSource.has(ddsPath)) {
        finalSrc = ddsPath;
    }
}
```

---

## Resource Cache

`ModelViewer` maintains a `Map<String, Resource> fetchCache` keyed by resolved path. All loaded resources (textures, models, generic data) live here.

```java
// Load or return cached
Resource cached = this.fetchCache.get(finalSrc);
if (cached != null) return cached;

Resource resource = handler.construct(...);
this.fetchCache.put(finalSrc, resource);
resource.loadData(dataSource.getResourceAsStream(finalSrc), null);
```

There is no async loading — `loadData()` blocks until complete.

---

## Relevant WC3 Asset Paths Inside MPQ

These are the paths used by the engine, relative to the MPQ root:

```
UI/
  FrameDef/UI/                  FDF files (main menu, in-game HUD)
  Skins/                        UI skin data
Units/
  UnitData.slk                  All unit stats
  ItemData.slk                  Item stats
  AbilityData.slk               Ability data
  UpgradeData.slk               Upgrade data
  UnitUI.slk                    Unit name/model path
  UnitArt.slk                   Model/portrait paths
  UnitBalance.slk               Combat stats
  UnitWeapons.slk               Attack data
Destructibles/
  DestructableData.slk          Trees, cliffs, buildings
Doodads/
  DoodadData.slk                Decorative objects
Terraindata/
  TerrainArt/                   Ground texture definitions
Units/
  Critters/                     Animal models
  Human/                        Human race models
  ...
war3mapImported/                Map-specific imported files
```

---

## CASC / Patch 1.32 (Reference Only)

For Reforged (Patch 1.32.10), the archive format changes to CASC:

```ini
Type00=CASC
Path00="/path/to/WC3/installation"
Prefixes00=war3.w3mod,war3.w3mod\_locales\enus.w3mod
```

Each `Prefix` corresponds to a virtual `.w3mod` directory inside the CASC store. The engine appends the prefix when resolving paths, allowing layered mod loading (base + locale + HD skins).
