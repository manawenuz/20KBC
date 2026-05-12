# 03 — File Format Specifications

All formats are little-endian unless noted.

---

## BLP — Texture Format

**Parser:** `core/src/com/etheller/warsmash/viewer5/handlers/blp/`  
**External library:** `blp-iio-plugin` by DrSuperGood

```
Magic:      "BLP2" (4 bytes) or "BLP1" (older)
Version:    uint32
Compression: uint8  (1=JPEG, 2=palettized, 3=DXT)
AlphaDepth: uint8   (0, 1, 4, 8 bits)
AlphaType:  uint8
HasMips:    uint8
Width:      uint32
Height:     uint32
MipOffsets: uint32[16]
MipSizes:   uint32[16]
Palette:    uint32[256]  (BGRA, only if palettized)
MipData[0..15]: byte[]   (compressed pixel data per mip level)
```

**Compression types:**
- `1` (JPEG): JPEG data after a shared JPEG header (stored once)
- `2` (Palettized): 8-bit indices into Palette, alpha from separate channel
- `3` (DXT/S3TC): DXT1, DXT3, or DXT5 depending on AlphaDepth

**Loading in Warsmash:**  
`BlpGdxTexture` wraps a `com.badlogic.gdx.graphics.Texture`. The blp-iio plugin decodes BLP to RGBA via Java's `ImageIO` API, then uploads to GL via `Pixmap`.

**Known issue:** alpha channel not decoded correctly for some campaign art textures (e.g. `NightElfCampaign3D_exp.blp`). Causes ragged edges on transparent areas.

**DDS files (Patch 1.32+):**  
Direct3D DDS format, loaded natively. sRGB gamma correction applied for BLP (not for DDS). Using DDS with sRGB correction causes textures to appear very dark.

---

## MDX / MDL — 3D Model Format

**Parser:** `core/src/com/hiveworkshop/rms/parsers/mdlx/MdlxModel.java`

MDX is binary (chunked), MDL is plain-text. Both describe the same data model.

### MDX Binary Layout

```
Header: "MDLX"  (4 bytes magic)

Chunks (repeated until EOF):
  Tag:   char[4]   (chunk type identifier)
  Size:  uint32    (byte size of chunk payload)
  Data:  byte[Size]
```

**Chunk Tags and Contents:**

| Tag | Name | Contents |
|-----|------|----------|
| `VERS` | Version | uint32 version (800=WC3, 1000=Reforged) |
| `MODL` | Model Info | name[80], animationFile[260], extent, blend time |
| `SEQS` | Sequences | Array of: name[80], interval[2], moveSpeed, flags, rarity, syncPoint, extent |
| `GLBS` | Global Sequences | Array of: uint32 duration (ms) |
| `MTLS` | Materials | Array of layers with textures, blend modes |
| `TEXS` | Textures | Array of: replaceableId(uint32), path[260], flags |
| `GEOS` | Geosets | Mesh geometry (see below) |
| `GEOA` | Geoset Animations | Visibility/color animations per geoset |
| `BONE` | Bones | Hierarchy nodes with pivot points + animated transforms |
| `LITE` | Lights | Point/directional/ambient lights |
| `HELP` | Helpers | Invisible nodes for attachment/attachment animations |
| `ATCH` | Attachments | Named attachment points (e.g. "overhead", "chest") |
| `PIVT` | Pivot Points | float[3] per node, indexed by node array position |
| `PREM` | Particle Emitter 1 | Old-style particle systems |
| `PRE2` | Particle Emitter 2 | New-style particles (more common) |
| `RIBB` | Ribbon Emitters | Ribbon/trail effects |
| `EVTS` | Event Objects | Frame-triggered sounds/footsteps |
| `CAMS` | Cameras | Predefined camera animations |
| `CLID` | Collision Shapes | Box/sphere/cylinder for click detection |

### Geoset Layout (inside GEOS)

```
Chunk header: size uint32
Vertices:   count uint32, then float[3] × count
Normals:    count uint32, then float[3] × count
FaceTypeGroups: count uint32, then uint32 × count (always 4 = triangles)
FaceGroups: count uint32, then uint32 × count (number of indices per group)
Faces:      count uint32, then uint16 × count (triangle indices)
VertexGroups: count uint32, then uint8 × count (bone matrix group per vertex)
MatrixGroups: count uint32, then uint32 × count
MatrixIndices: count uint32, then uint32 × count
MaterialId: uint32
SelectionGroup: uint32
SelectionFlags: uint32
Extent: Extent (float[3] min, float[3] max, float radius)
Extents: count uint32, then Extent × count (per-sequence extent)
UVSets: count uint32, then per-set: count uint32, float[2] × count
```

### Animated Properties (KXXX tracks)

Any property that changes over time uses a track block:

```
Tag:        char[4]  e.g. "KGTR" = Global Translation
TrackCount: uint32
InterpolationType: uint32 (0=none, 1=linear, 2=hermite, 3=bezier)
GlobalSequenceId: int32  (-1 = use animation time, ≥0 = use global sequence)

Tracks[TrackCount]:
  Time:  int32        (millisecond timestamp)
  Value: float/vec3/vec4/etc  (depends on track type)
  [InTan, OutTan: same type, only if Hermite or Bezier]
```

**Common track types:**

| Tag | Property | Value type |
|-----|----------|-----------|
| KGTR | Translation | vec3 |
| KGRT | Rotation | quaternion (vec4) |
| KGSC | Scale | vec3 |
| KLAV | Layer visibility | float (0 or 1) |
| KMTF | Texture animation | uint32 (frame index) |
| KP2S | Particle2 speed | float |
| KP2E | Particle2 emission rate | float |
| KPPA | Particle visibility | float |

**Interpolation formula (Hermite/Bezier):**
```
t = (currentTime - track[i].time) / (track[i+1].time - track[i].time)
hermite(v0, out0, in1, v1, t)  // standard cubic hermite
```

---

## W3X — Map Archive Format

A W3X map is itself a small MPQ archive containing multiple sub-files.

**Opening:** `War3Map` class opens the map path as an `MpqDataSource`, then reads each sub-file by name.

### Sub-files

| Filename | Class | Contents |
|----------|-------|----------|
| `war3map.w3i` | `War3MapW3i` | Map name, players, teams, start locations |
| `war3map.w3e` | `War3MapW3e` | Terrain heightmap, textures, tileset |
| `war3map.wpm` | `War3MapWpm` | Pathing grid (movement/building flags) |
| `war3map.doo` | `War3MapDoo` | Doodad placements (trees, props) |
| `war3mapUnits.doo` | `War3MapUnitsDoo` | Unit/building starting positions |
| `war3map.j` | raw stream | JASS map script |
| `war3map.lua` | raw stream | Lua map script (newer maps) |
| `war3map.w3r` | `War3MapW3r` | Regions |
| `war3map.w3c` | — | Camera definitions |
| `war3map.w3s` | — | Sound definitions |
| `war3map.w3u` | — | Unit object data (overrides) |
| `war3map.w3t` | — | Item overrides |
| `war3map.w3a` | — | Ability overrides |
| `war3map.w3b` | — | Destructable overrides |
| `war3map.w3d` | — | Doodad overrides |
| `war3map.w3q` | — | Upgrade overrides |
| `war3mapImported/*` | — | Imported custom assets |

### war3map.w3e — Terrain Format

```
Magic:     "W3E!" (4 bytes)
Version:   uint32  (typically 11)
Tileset:   char    ('A'=Ashenvale, 'B'=Barrens, 'C'=Felwood, …)
CustomTileset: uint32 (1=uses custom tiles)
GroundTileCount: uint32
GroundTileIds:   War3ID[GroundTileCount]   (4-byte IDs e.g. "Lgrs")
CliffTileCount:  uint32
CliffTileIds:    War3ID[CliffTileCount]
MapWidth:   uint32   (in corners = tiles+1)
MapHeight:  uint32
CenterOffsetX: float
CenterOffsetY: float

Corners[MapHeight][MapWidth]:
  GroundHeight:   uint16  (actual height = (raw - 0x2000) / 4.0)
  WaterLevel:     uint16  (actual = (raw - 0x2000) / 4.0)
  Flags:          uint8
    bits 0-3:  Cliff texture index (into CliffTileIds)
    bit 4:     Boundary (1=edge of map)
    bit 5-6:   Ramp type
    bit 7:     Has water
  GroundTexture:  uint8   lower nibble = tile index (into GroundTileIds)
                          upper nibble = texture variation (0-3)
  TextureAndCliff:uint8
    bits 0-3:  ground texture layer detail
    bits 4-7:  cliff variation
  CliffLevel:    uint8   (height of cliff layer, 0=no cliff)
  LayerHeight:   uint8   (for multi-level terrain)
```

**Coordinate system:**  
- 1 tile = 128 world units  
- Corner (0,0) = bottom-left  
- Height: 1 unit = 0.25 world height units  
- Map dimensions: width × height corners

### war3map.wpm — Pathing Map

```
Magic:   "MP3W" (4 bytes)
Version: uint32  (0)
Width:   uint32  (4× terrain width in cells)
Height:  uint32  (4× terrain height in cells)
Cells[Height][Width]: uint8  (bitfield)
  bit 0: NOT walkable (land)
  bit 1: NOT flyable
  bit 2: NOT buildable
  bit 3: blight
  bit 4: NOT water walkable
  bit 5: unknown
  bit 6: unknown
  bit 7: unknown
```

Pathing resolution is 4× terrain resolution (32 units per cell vs 128 units per tile).

### war3mapUnits.doo — Unit Placements

```
Magic:   "W3do" (4 bytes) or "W3do" for TFT
Version: uint32  (7 or 8)
SubVersion: uint32

UnitCount: uint32
Units[UnitCount]:
  TypeId:   War3ID (4 bytes, e.g. "hpea" = Human Peasant)
  Variation: uint32
  X:        float  (world position)
  Y:        float
  Z:        float
  Facing:   float  (radians)
  ScaleX:   float
  ScaleY:   float
  ScaleZ:   float
  Flags:    uint8
  Player:   uint32 (0-27, 27=neutral passive)
  Padding:  uint16
  HP:       int32  (-1 = use unit default)
  MP:       int32  (-1 = use unit default)
  ItemSets: variable (item drop tables)
  GoldAmount: uint32  (for gold mines)
  TargetAcquisition: float  (-1=normal, -2=camp)
  HeroLevel: uint32
  HeroStr, HeroAgi, HeroInt: uint32  (0=use defaults)
  ItemCount:  uint32
  Items[ItemCount]: slot(uint32) + itemId(War3ID)
  CustomName: CString
  CreationNumber: uint32
```

---

## SLK — Data Table Format

Plain text spreadsheet. Used for all game data (units, abilities, upgrades, etc.).

```
ID;PWXL;N;E        ← header line (always present)
P;F5P;            ← print/format metadata (ignore)
F;W65535;...      ← column widths (ignore)
B;Y<rows>;X<cols> ← table dimensions

C;Y<row>;X<col>;K"<string value>"   ← string cell
C;Y<row>;X<col>;N<number>           ← numeric cell
C;Y<row>;X<col>;            ← empty (inherit column from previous)
E                 ← end of file
```

**Row 1** is always the header row (column names).  
**Column 1** is always the row ID (unit typeId, e.g. `hpea`).

**Example:**
```
C;Y1;X1;K"unitID"
C;Y1;X2;K"HP"
C;Y1;X3;K"mana"
C;Y2;X1;K"hpea"
C;Y2;X2;N420
C;Y2;X3;N0
```

**DataTable.java** builds:  
`Map<String rowId, Map<String colName, String value>>`

Accessing: `table.get("hpea").getField("HP")` → `"420"`

---

## FDF — Frame Definition File

Text format, loosely similar to Lua table syntax. Defines the entire WC3 UI hierarchy.

```
Frame "SIMPLEFRAME" "MyFrame" {
    SetPoint TOPLEFT, "MyParent", TOPLEFT, 0.0, 0.0
    Width 0.1
    Height 0.05

    Frame "BACKDROP" "MyBackdrop" {
        SetAllPoints
        DecorateFileNames
        BackdropTileBackground true
        BackdropBackground "UI\\Widgets\\EscMenu\\Human\\blank-background.blp"
        BackdropEdgeFile "UI\\Widgets\\EscMenu\\Human\\blank-background-border.blp"
        BackdropEdgeSize 0.007
        BackdropInsets 0.002, 0.002, 0.002, 0.002
    }

    Frame "TEXT" "MyLabel" {
        SetPoint CENTER, "MyFrame", CENTER, 0.0, 0.0
        Width 0.09
        Height 0.05
        FontSize 0.012
        FontColor 1.0, 1.0, 1.0, 1.0
    }
}
```

**Frame types (parsed by GameUI.java):**  
`SIMPLEFRAME`, `FRAME`, `BUTTON`, `GLUEBUTTON`, `GLUETEXTBUTTON`, `TEXT`, `TEXTAREA`, `EDITBOX`, `MENU`, `POPUPMENU`, `BACKDROP`, `HIGHLIGHT`, `SLASHCHATBOX`, `SPRITE`, `CHECKBOX`, `SCROLLBAR`, `LISTBOX`

**Coordinate system:**  
- Base canvas: `0.8` wide × `0.6` tall (4:3 ratio)
- Origin: bottom-left = (0, 0)
- All sizes/positions in these units (not pixels)
- SetPoint anchors a frame relative to another frame's named anchor

**Anchor names:** `TOPLEFT`, `TOP`, `TOPRIGHT`, `LEFT`, `CENTER`, `RIGHT`, `BOTTOMLEFT`, `BOTTOM`, `BOTTOMRIGHT`

**Parser location:** `fdfparser/src/` — ANTLR 4.7 grammar, generates `FDFLexer` and `FDFParser`. Entry point: `DataSourceFDFParserBuilder.java`.

---

## JASS — Script Format

JASS (Just Another Scripting Syntax) is WC3's built-in scripting language.

```jass
globals
    integer udg_MyVariable = 0
    unit array udg_MyUnits
endglobals

function MyFunction takes integer i returns string
    local string s = "hello"
    if i > 0 then
        set s = "positive"
    elseif i < 0 then
        set s = "negative"
    else
        set s = "zero"
    endif
    return s
endfunction

function InitTrig_MyTrigger takes nothing returns nothing
    local trigger t = CreateTrigger()
    call TriggerRegisterAnyUnitEventBJ(t, EVENT_UNIT_DEATH)
    call TriggerAddAction(t, function MyTriggerAction)
endfunction
```

**Types:** `integer`, `real`, `boolean`, `string`, `unit`, `player`, `trigger`, `timer`, `group`, `force`, `rect`, `region`, `item`, `destructable`, `effect`, `code` (function pointer), `handle` (base type for objects)

**Key files inside a map:**
- `war3map.j` — map-specific code (user triggers + auto-generated)
- `common.j` — native function declarations (in War3.mpq)
- `blizzard.j` — Blizzard library functions (in War3.mpq)

**Parser location:** `jassparser/src/` — ANTLR 4.7 grammar. Entry: `SmashJassParser.java`.

See [05_jass_interpreter.md](05_jass_interpreter.md) for interpreter details.
