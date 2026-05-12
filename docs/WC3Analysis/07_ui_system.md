# 07 — FDF UI System

## Overview

The entire WC3 UI — every menu, button, text field, dropdown — is defined in `.fdf` (Frame Definition Files) text files stored in the MPQ. Warsmash parses them and builds a frame hierarchy that it renders using LibGDX's `SpriteBatch`.

**Key principle:** the Java/engine code never hard-codes UI layout. It creates generic frame objects; the FDF file controls exactly where they appear, what they look like, and how they're anchored.

---

## FDF File Locations (inside MPQ)

```
UI/FrameDef/UI/
  MainMenu.fdf              Main menu buttons
  EscMenu.fdf               In-game pause menu
  GameInterface.fdf         HUD: minimap, resources, command card
  glues/                    Lobby, map selection, score screens
  InfoCard.fdf              Unit info panel
  Tooltips.fdf              Ability tooltips
  ...
UI/Skins/
  HumanSkin.txt             Texture path mappings (skin)
  NightElfSkin.txt
  ...
```

Warsmash loads FDF files via:
```java
DataSourceFDFParserBuilder.buildParser(dataSource, "UI/FrameDef/UI/GameInterface.fdf")
```

---

## FDF Syntax

```
// Inheritance: inherit all properties from a template
Frame "BUTTON" "MyButton" INHERITS "GLUEBUTTON" {

    // Position relative to parent frame
    SetPoint TOPLEFT, "MyButton", TOPLEFT, 0.0, 0.0

    // Or fill parent entirely
    SetAllPoints

    // Size
    Width 0.1
    Height 0.03

    // A child frame (a backdrop/texture)
    Frame "BACKDROP" "MyButtonBackground" {
        SetAllPoints
        DecorateFileNames                     // auto-add UpNormal/PushedDown suffixes
        BackdropBackground "UI\\Widgets\\..."
        BackdropEdgeFile   "UI\\Widgets\\..."
        BackdropEdgeSize   0.006
        BackdropInsets     0.002, 0.002, 0.002, 0.002
    }

    // A text child
    Frame "TEXT" "MyButtonText" {
        SetPoint CENTER, "MyButton", CENTER, 0.0, 0.0
        FontSize 0.010
        FontColor 1.0, 0.82, 0.0, 1.0     // R G B A
    }
}
```

---

## Coordinate System

| Property | Value |
|----------|-------|
| Canvas width | 0.8 |
| Canvas height | 0.6 |
| Origin | Bottom-left = (0, 0) |
| Top-right | (0.8, 0.6) |
| Screen center | (0.4, 0.3) |

These units are independent of screen resolution. The viewport converts them to pixels at render time.

**Why 0.8 × 0.6?** WC3 was designed for 4:3 (800×600) screens. The normalized coordinate space preserves that ratio.

---

## Frame Types and Java Classes

| FDF type | Java class | Description |
|----------|-----------|-------------|
| `SIMPLEFRAME` / `FRAME` | `SimpleFrame` | Container, no visual |
| `BACKDROP` | `BackdropFrame` | Textured background with border |
| `HIGHLIGHT` | `HighlightFrame` | Mouse-over highlight layer |
| `TEXT` | `StringFrame` | Renders a string via BitmapFont |
| `TEXTAREA` | `TextAreaFrame` | Multi-line scrollable text |
| `EDITBOX` | `EditBoxFrame` | Text input (handles key events) |
| `BUTTON` | `SimpleButtonFrame` | Clickable, triggers onClick Runnable |
| `GLUEBUTTON` | `GlueButtonFrame` | Menu button with sound |
| `GLUETEXTBUTTON` | `GlueTextButtonFrame` | Menu button with text label |
| `MENU` | `MenuFrame` | Dropdown menu items |
| `POPUPMENU` | `PopupMenuFrame` | Button + dropdown |
| `CHECKBOX` | `CheckBoxFrame` | Toggle checkbox |
| `SCROLLBAR` | `ScrollBarFrame` | Scroll control |
| `LISTBOX` | `ListBoxFrame` | Scrollable list |
| `SPRITE` | `SpriteFrame` | Animated MDX model in UI |
| `SLASHCHATBOX` | `SlashChatBoxFrame` | Chat input field |

---

## GameUI — Runtime Manager

```
core/src/com/etheller/warsmash/parsers/fdf/GameUI.java
```

**Key responsibilities:**
- Parse FDF files and instantiate frame objects
- Maintain `Map<String, UIFrame> nameToFrame` for lookup by name
- Handle mouse clicks: hit-test frame tree, fire onClick callbacks
- Handle keyboard: route to `focusUIFrame`
- Render all frames via `SpriteBatch`

**Frame creation** (inside GameUI's FDF inflation code):

```java
// For each Frame block in the FDF:
UIFrame frame = createFrameOfType(frameType, frameName, parent);
applyProperties(frame, frameDefinition);
nameToFrame.put(frameName, frame);
parent.addChild(frame);
```

**setText / getText:**
```java
public void setText(StringFrame frame, String text) {
    frame.setText(text);
    frame.positionBounds(this, viewport);  // reflow text
}
```

---

## Rendering Pipeline

```
GameUI.render(SpriteBatch batch, BitmapFont font, GlyphLayout layout)
  └─ rootFrame.render(batch, font, layout)
       └─ for each child: child.render(batch, font, layout)
            ├─ BackdropFrame:  batch.draw(backgroundTexture, ...)
            │                  batch.draw(edgeTextures × 8 patches, ...)
            ├─ StringFrame:    font.draw(batch, text, x, y)
            ├─ TextureFrame:   batch.draw(texture, x, y, w, h)
            └─ ButtonFrame:    draw normal/hovered/pressed texture based on state
```

`SpriteBatch` uses an orthographic projection over the 0..0.8 × 0..0.6 coordinate space.

Fonts are loaded via `FreeTypeFontGenerator` (FreeType + LibGDX extension):
```java
FreeTypeFontParameter params = new FreeTypeFontParameter();
params.size = (int)(fontSize * viewport.getWorldHeight() / 0.6f);
BitmapFont font = generator.generateFont(params);
```

---

## Frame Anchoring

Frames are positioned by `SetPoint` declarations. Each frame can have up to 2 SetPoint constraints (enough to define both position and size, or just position):

```
SetPoint TOPLEFT,  "TargetFrame", BOTTOMLEFT, xOffset, yOffset
```

Meaning: *my TOPLEFT should be at TargetFrame's BOTTOMLEFT, plus (xOffset, yOffset).*

**Anchor points:**
```
TOPLEFT     TOP     TOPRIGHT
LEFT        CENTER  RIGHT
BOTTOMLEFT  BOTTOM  BOTTOMRIGHT
```

`SetAllPoints` = two SetPoints: TOPLEFT to parent's TOPLEFT + BOTTOMRIGHT to parent's BOTTOMRIGHT. Makes the frame fill its parent exactly.

**Resolution order:** `positionBounds()` is called after all frames are created. It recursively resolves anchor chains. Circular dependencies are not handled (map/skin authors must avoid them).

---

## Event Handling

**Mouse:**
```java
// GameUI.touchDown(screenX, screenY, button)
float uiX = screenX / viewportPixelWidth * 0.8f;
float uiY = (1 - screenY / viewportPixelHeight) * 0.6f;
UIFrame hit = rootFrame.hitTest(uiX, uiY);
if (hit instanceof ClickableFrame) ((ClickableFrame)hit).onClick(button);
```

**Keyboard:**
```java
// GameUI.keyDown(keycode)
if (focusUIFrame != null) focusUIFrame.keyDown(keycode);

// TAB cycles focus to next focusable frame
// EditBoxFrame handles text entry characters
```

---

## PopupMenuFrame (Dropdown)

A composite of:
- `GlueTextButtonFrame` (the visible button, shows current selection)
- `MenuFrame` (the dropdown list, hidden until button clicked)
- Arrow texture frame

```java
// When button clicked:
popupMenuFrame.setVisible(!popupMenuFrame.isVisible());

// When item clicked (MenuFrame.onItemClick):
popupMenuFrame.onClickItem(button, itemIndex);
  → onClick(button)               // close dropdown
  → updateTitleText(itemIndex)    // update button label
  → menuClickListener.onClick(button, itemIndex)  // notify game code
```

**Setting initial selection (required for non-empty display):**
```java
glueButtonFrame.setGameUIAndViewport(gameUI, viewport);
glueButtonFrame.setSelectedIndex(0);   // show first item as default
```

This was a bug in the original code: without `setSelectedIndex(0)`, the dropdown button shows no text on first render.

---

## Adding Custom UI (for 20KBC)

To add a custom UI element in a WC3 map:

1. Create a `.fdf` file defining your frames
2. Import it into the map via World Editor (File > Import)
3. In JASS: `BlzLoadTOCFile("war3mapImported\\MyUI.fdf")`
4. In JASS: `BlzGetFrameByName("MyFrameName", 0)` to get a handle to the frame
5. Use `BlzFrameSetText`, `BlzFrameSetVisible`, `BlzFrameSetPoint`, etc. to control it

For Warsmash specifically, custom UI frames are handled by the `BlzXxx` native family, which are wired in `CommonTriggerExecutionScope.java`.
