# 04 — Rendering Pipeline

## Overview

Warsmash uses two parallel rendering paths:

```
WorldScene (3D perspective)          SimpleScene (2D orthographic)
  ├─ Terrain (ground + water)          └─ GameUI (FDF frames)
  ├─ MDX model instances                    └─ SpriteBatch
  ├─ Particles / ribbons
  ├─ Shadows / UberSplats
  └─ Selection circles
```

Both scenes are rendered each frame:
```java
// ModelViewer.render()
for (Scene scene : scenes) {
    scene.startFrame();
    scene.renderOpaque();
    scene.renderTranslucent();
}
```

---

## WebGL Wrapper

```
core/src/com/etheller/warsmash/viewer5/gl/WebGL.java
```

Not WebGL — just a utility wrapper around LibGDX's `GL20` interface, ported from Ghostwolf's JavaScript codebase.

**Key responsibilities:**
- `createShaderProgram(vertSrc, fragSrc)` — compiles + caches by hash
- `useShaderProgram(shader)` — switches shader, enables/disables vertex attribs
- `bindTexture(texture, unit)` — activates texture unit, binds texture (or 2×2 black if null)
- `setTextureMode(wrapS, wrapT, magFilter, minFilter)` — wrapping + filtering
- `emptyTexture` — 2×2 solid black fallback

**Shader caching:** programs are keyed by `hashCode(vertSrc + fragSrc)`. Same shader source always returns the same `ShaderProgram` object.

**macOS GLSL translation (applied in DesktopLauncher before any shader compiles):**
```java
ShaderProgram.prependVertexCode =
    "#version 150\n" +
    "#define attribute in\n" +
    "#define varying out\n" +
    "#define texture2D texture\n" +
    "#define textureCube texture\n" +
    "#define texture2DLod textureLod\n" +
    "#define texture2DProj textureProj\n" +
    "#define textureCubeLod textureLod\n";

ShaderProgram.prependFragmentCode =
    "#version 150\n" +
    "#define varying in\n" +
    "#define texture2D texture\n" +
    "#define textureCube texture\n" +
    "#define texture2DLod textureLod\n" +
    "#define texture2DProj textureProj\n" +
    "#define textureCubeLod textureLod\n" +
    "#define gl_FragColor _fragColor_\n" +
    "out vec4 _fragColor_;\n";
```

**Precision stripping (all platforms):**
```java
// WebGL.stripPrecision()
src.replaceAll("(?m)^\\s*precision\\s+\\w+\\s+\\w+\\s*;[^\\n]*\\n?", "")
```
This removes `precision mediump float;` etc. which are invalid in desktop GLSL ≥ 1.50.

---

## MDX Model Rendering

### Data Flow

```
DataSource.read("Units/Human/Peasant/Peasant.mdx")
  → ByteBuffer
  → MdlxModel(buffer)          [parse binary chunks]
  → MdxModel(mdlxModel, viewer)  [build GL buffers]
  → MdxComplexInstance(mdxModel) [per-instance state]
  → scene.addInstance(instance)
  → update() each tick: advance animation frame
  → renderOpaque() / renderTranslucent()
```

### MdlxModel (Parser)
```
core/src/com/hiveworkshop/rms/parsers/mdlx/MdlxModel.java
```
Pure data: no GL calls. Reads binary chunks into Java objects:
- `List<MdlxSequence> sequences` — named animations + frame ranges
- `List<MdlxGeoset> geosets` — mesh data (vertices, normals, UVs, faces, bone weights)
- `List<MdlxMaterial> materials` — per-geoset materials
- `List<MdlxLayer> layers` — per-material layers (texture + blend mode)
- `List<MdlxBone> bones` + `List<MdlxHelper> helpers` — skeleton
- `List<MdlxParticleEmitter2> particleEmitters2` — particles
- `List<MdlxRibbonEmitter> ribbonEmitters`

### MdxModel (Renderer)
```
core/src/com/etheller/warsmash/viewer5/handlers/mdx/MdxModel.java
```
Converts parsed data to GL resources:
- Uploads vertex/index data to GPU buffers
- Builds `List<Batch> batches` — groups geosets by texture mapper for draw call efficiency
- `List<GenericGroup> opaqueGroups` — geosets with no alpha blending
- `List<GenericGroup> translucentGroups` — geosets with alpha blend

### MdxComplexInstance
```
core/src/com/etheller/warsmash/viewer5/handlers/mdx/MdxComplexInstance.java
```
Per-instance state:
- `float[] nodeMatrices` — flattened 4×4 bone matrices (one per skeleton node)
- `int currentSequence` — which animation is playing
- `float currentFrame` — frame counter within sequence
- `float[] worldMatrices` — final world transform matrices for rendering
- Skin/texture overrides: `TextureMapper textureMapper`

**Animation update (each tick):**
```
currentFrame += deltaTime × sequence.speed
if (currentFrame > sequence.end):
    if (looping): currentFrame = sequence.start
    else: freeze at end

// Evaluate all animated tracks at currentFrame
// Write results into nodeMatrices
// Propagate through hierarchy: parent × local = world
```

### Shaders

Located in `core/src/com/etheller/warsmash/viewer5/handlers/mdx/shaders/` (loaded as strings, compiled at runtime).

**Vertex shader (HD/SD variants):**
- Inputs: position, normal, UV, boneIndices, boneWeights
- Uniforms: boneMatrices (mat4 array), mvpMatrix
- Skin transform: weighted sum of up to 4 bone matrices

**Fragment shader:**
- Diffuse texture lookup
- Alpha test (discard if alpha < threshold)
- Fog blending
- Team color tint

### Blend Modes (MdlxLayer)

| Value | Name | GL Blend |
|-------|------|----------|
| 0 | Opaque | None (depth write on) |
| 1 | Transparent | src_alpha / one_minus_src_alpha |
| 2 | Blend | src_alpha / one_minus_src_alpha |
| 3 | Additive | one / one |
| 4 | AddAlpha | src_alpha / one |
| 5 | Modulate | dst_color / zero |
| 6 | Modulate2x | dst_color / src_color |

---

## Terrain Rendering

```
core/src/com/etheller/warsmash/viewer5/handlers/w3x/environment/Terrain.java
```

### Ground Mesh

The terrain is a single large mesh generated from the heightmap:

```
For each 2×2 corner quad (tile):
  Generate 2 triangles
  Interpolate heights from corner.groundHeight values
  Assign texture coordinates from tileset UV atlas
  Compute normals from height differences
```

Ground textures are assembled into a **texture array** (GL_TEXTURE_2D_ARRAY):
- Each tileset tile is one layer
- The mesh UVs select which layer via a per-vertex attribute
- Blend weights allow smooth transitions between tile types at corners

**Key GL objects:**
- `int groundTextureData` — texture array (512×512 per layer, up to 16 layers)
- `int groundHeight` — 1-channel floating point texture, one texel per corner
- VAO + VBO for ground mesh geometry

### Water

Rendered separately after ground with alpha blending:
- Flat plane at water level
- Animated by cycling through water texture frames (24 frames @ 15fps)
- Vertex colors encode shallow/deep tint
- `Terrain.update()` advances the water frame index

### Cliff Meshes

```
core/src/com/etheller/warsmash/viewer5/handlers/w3x/environment/CliffMesh.java
```

Cliffs are separate pre-built mesh objects:
- Looked up from a library of cliff models (`CliffAbrb.mdx`, etc.)
- Each cliff type/variation has a corresponding MDX asset
- Placed at the correct world position based on the corner's cliff texture + level data
- Rotated by 90° vs. HiveWE orientation (Warsmash-specific fix)

### Render Order

```
1. Ground (opaque, depth write on)
2. Water (translucent, depth write off, additive blend)
3. Shadows / UberSplats (decals on ground, additive)
4. MDX model instances (opaque pass)
5. MDX model instances (translucent pass, back-to-front sorted)
6. Particles + ribbons (translucent, additive)
7. Selection circles (translucent, additive)
8. UI (2D orthographic, SpriteBatch, no depth test)
```

---

## Scene Management

```
core/src/com/etheller/warsmash/viewer5/Scene.java
```

### Viewport Setup (HiDPI Critical)

```java
// Scene.startFrame() — correct
HdpiUtils.glViewport((int)viewport.x, (int)viewport.y,
                     (int)viewport.width, (int)viewport.height);
HdpiUtils.glScissor((int)viewport.x, (int)viewport.y,
                    (int)viewport.width, (int)viewport.height);
```

**Why HdpiUtils?**  
On Retina displays with `HdpiMode.Logical` (the default), `camera.rect` holds logical pixels (e.g. 640×480 for a 1280×960 window). But `glViewport` needs physical pixels (1280×960). `HdpiUtils` multiplies by the backBuffer/logical ratio automatically.

**Wrong way** (causes rendering in bottom-left quadrant):
```java
gl.glViewport((int)viewport.x, (int)viewport.y, ...);  // DO NOT USE
```

### Instance Depth Sorting

Before rendering translucent instances, sort back-to-front:
```java
Collections.sort(instances, (a, b) -> Float.compare(b.depth, a.depth));
```
`depth` is computed during update as the distance from camera to instance origin.

---

## Camera

```
core/src/com/etheller/warsmash/viewer5/Camera.java
```

- `viewport`: Rectangle in logical pixels (where on screen to draw)
- `perspective(fov, aspect, near, far)` — builds projection matrix
- `viewProjectionMatrix` — pre-multiplied view × projection, uploaded to shaders as uniform

WC3-style RTS camera: angled ~315° (northwest facing), tilted ~60° down. Controlled by `GameCameraManager.java`.

---

## Shadows

Two shadow systems:

1. **Static ground shadow** (simple): Rendered as a textured quad on the ground plane beneath each unit. Texture: a soft dark circle.

2. **Dynamic shadow map** (optional): `DynamicShadowManager.java`  
   - Renders scene depth from a directional light into a depth texture  
   - Currently commented out in `Scene.renderOpaque(DynamicShadowManager, WebGL)` — the shadow map is generated but not composited  

**UberSplats:** Ground textures that show blood stains, building foundations, etc. Rendered as additive decals on the terrain.

---

## Particle Systems

### ParticleEmitter2
Most WC3 effects use emitter type 2 (`PRE2` chunk).

Per-particle state:
- `float[] position` — world xyz
- `float[] velocity` — units/tick
- `float life, maxLife` — normalized 0..1
- `float size` — current sprite size
- `float angle` — rotation
- `float color[4]` — RGBA, animated over lifetime

Particles are billboarded quads (always face camera).  
Rendered in a single batched draw call per emitter using a `SpriteBatch`-like approach.

### RibbonEmitter
Generates a trail of connected quads following a node's path. Used for spell effects and hero glows.
