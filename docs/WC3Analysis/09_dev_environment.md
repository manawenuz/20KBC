# 09 — Dev Environment & Asset Locations

## Hardware

- MacBook Air M4 (Apple Silicon, Retina display)
- External drive: `/Volumes/samGames/` (APFS or exFAT, must be mounted before launch)

---

## WC3 Asset Paths on This Machine

```
/Volumes/samGames/WC3/
  War3.mpq            base RoC assets (models, textures, UI, common.j)
  War3Local.mpq       English locale strings and voice lines
  War3x.mpq           Frozen Throne expansion assets
  War3xLocal.mpq      TFT English locale
  Deprecated.mpq      legacy assets still referenced by some models
  Maps/               custom map folder (searched by Warsmash for .w3x/.w3m)
```

**Patch version:** 1.27 (MPQ format, MaxPlayers=28)

**If the drive is not mounted:** Warsmash prints `Attempting to load non-existent file:` for every asset and falls back to a black screen. It does not crash. Mount the drive first.

---

## Warsmash Repo

```
/Users/manwe/CascadeProjects/WarsmashModEngine/
  core/
    assets/
      warsmash.ini          ← edit this to change WC3 paths
    src/                    Java source
  desktop/
    assets/                 working directory when running via Gradle
    src/                    DesktopLauncher + audio backend
  resources/                Warsmash-specific override assets
    (shipped in repo, mounted as Folder data source)
  run.sh                    macOS startup script
  build.gradle
  desktop/build.gradle
```

---

## Building and Running

### Quick start (macOS)

```bash
cd /Users/manwe/CascadeProjects/WarsmashModEngine
./run.sh                         # windowed mode
./run.sh --full                  # fullscreen
./run.sh --map /path/to/my.w3x  # load map directly
```

### Gradle directly

```bash
./gradlew desktop:runGame                              # fullscreen
./gradlew desktop:runGame -Pargs="-window"             # windowed
./gradlew desktop:runGame -Pargs="-loadfile MyMap.w3x -window"
```

### Build release JAR

```bash
./gradlew desktop:runtime
# output: desktop/build/image/
```

### Java version

```bash
which java     # should be /Library/Java/JavaVirtualMachines/temurin-17.jdk/...
java -version  # Eclipse Temurin 17
```

If `java -version` shows something else, set `JAVA_HOME`:
```bash
export JAVA_HOME=/Library/Java/JavaVirtualMachines/temurin-17.jdk/Contents/Home
```

---

## warsmash.ini Full Reference (Current Config)

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

[GamingNetwork]
Server=warsmash.net

[Emulator]
MaxPlayers=28
GameVersion=1
CatchCursor=1
FixFlatFilesTilesetLoading=0
EnableMusic=0
LoadUnitsFromWorldEditData=0
CrashOnIncompatible132Features=0
InputHotkeyMode=0
ParseReignOfChaosBetaModelsInstead=0
```

---

## macOS-Specific Code Changes

All macOS changes are gated on `org.lwjgl.system.Platform.get() == Platform.MACOSX`:

### DesktopLauncher.java
```java
if (MACOSX) {
    // GLSL 1.50 compatibility aliases
    ShaderProgram.prependVertexCode = "#version 150\n#define attribute in\n...";
    ShaderProgram.prependFragmentCode = "#version 150\n#define varying in\n...";
}

// Later, in config setup:
if (MACOSX) {
    config.setOpenGLEmulation(GLEmulation.GL32, 3, 2);
}
```

### WebGL.java
```java
// Constructor — fixes font atlas texture corruption on non-4-byte-aligned rows
gl.glPixelStorei(GL20.GL_UNPACK_ALIGNMENT, 1);

// Shader preprocessing — removes 'precision mediump float;' etc.
private static String stripPrecision(String src) {
    return src.replaceAll("(?m)^\\s*precision\\s+\\w+\\s+\\w+\\s*;[^\\n]*\\n?", "");
}
```

### Scene.java + ModelViewer.java
```java
// All glViewport and glScissor calls replaced with HdpiUtils versions
// Import: com.badlogic.gdx.graphics.glutils.HdpiUtils
HdpiUtils.glViewport(x, y, w, h);   // NOT gl.glViewport()
HdpiUtils.glScissor(x, y, w, h);    // NOT gl.glScissor()
```

**Why this matters:** On Retina in windowed mode, logical px ≠ physical px (ratio = 2). Raw `glViewport` renders to bottom-left quarter of window. `HdpiUtils` multiplies by `backBufferScale` automatically.

---

## Inspecting WC3 MPQ Contents

To explore what's inside the MPQ files (useful for reimplementation research):

```bash
# MPQEditor (Windows) — GUI tool, most complete
# ladik's MPQ Editor: https://www.zezula.net/en/mpq/download.html

# On macOS, use StormLib CLI:
brew install stormlib   # if available, or build from source
# Or use Warsmash's own DataSource to enumerate:
# CompoundDataSource.getMergedListfile()  returns Set<String> of all paths
```

**Useful paths to inspect:**
```
Scripts/common.j          all native JASS function declarations
Scripts/blizzard.j        all BJ library function bodies
UI/FrameDef/UI/*.fdf      all UI frame definitions
Units/UnitData.slk        all unit stats
Units/UnitUI.slk          unit names and model paths
Terraindata/Terrain*.slk  tileset definitions
Sound/                    all audio assets
Units/Human/Peasant/      the Peasant model + textures
```

---

## Known Issues on macOS

| Symptom | Cause | Fix |
|---------|-------|-----|
| Black screen, no 3D | Drive not mounted | `ls /Volumes/samGames/WC3/` — if empty, remount |
| `Bad shader` crash | GLSL version mismatch | Check `prependVertexCode` is set before first shader |
| Game renders in bottom-left only (windowed) | Raw `glViewport` used | Ensure `HdpiUtils.glViewport` is used everywhere |
| Font / text invisible | `GL_UNPACK_ALIGNMENT` not set | `WebGL` constructor must set it to 1 |
| Alt+Enter not toggling fullscreen | Key not handled | `WarsmashGdxMenuScreen.keyDown` handles `ALT+ENTER → WarsmashUtils.toggleFullScreen()` |
| F10 does nothing in-game | Not implemented | `MeleeUI.keyDown()` has no F10 case — needs to open game menu panel |
| Menu shortcuts (S, B, etc.) do nothing on main menu | Not implemented | `MenuUI.keyDown()` only routes to focused frame, no dispatch table |

---

## GitHub Fork

```
https://github.com/manawenuz/WarsmashModEngine
SSH key: ~/CascadeProjects/github  (OpenSSH private key)

# Push changes:
GIT_SSH_COMMAND="ssh -i $HOME/CascadeProjects/github" git push fork main
```

Remote named `fork` points to the macOS-ported fork.  
Remote named `origin` points to the intermediate fork `ErikSom/WarsmashModEngine`.  
Upstream: `Retera/WarsmashModEngine`.
