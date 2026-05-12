# 08 — Audio System

## Stack

```
LibGDX Audio API (interface)
  └─ OpenAL backend (desktop)
       ├─ OpenALAudio.java     (device + source management)
       ├─ OpenALSound.java     (short sound effects, loaded fully into memory)
       └─ OpenALMusic.java     (background music, streamed)
```

Located in: `desktop/src/com/etheller/warsmash/desktop/audio/`

---

## Format Support

| Extension | Decoder class | Notes |
|-----------|--------------|-------|
| `.wav` | `Wav.java` | PCM, read fully |
| `.ogg` | `Ogg.java` | Vorbis, streamed for music |
| `.mp3` | `Mp3.java` | Decoded via JLayer |
| `.flac` | `Flac.java` | Decoded via `io/nayuki/flac/`, lossy conversion to WAV in memory |

**FLAC note:** Patch 1.32 moved all audio to FLAC. The nayuki FLAC decoder drops some precision bytes when converting to the WAV PCM format LibGDX expects. Result: audio may sound slightly "tinny" compared to playing the source FLAC directly.

---

## OpenALAudio

**Initialization:**
```java
OpenALAudio audio = new OpenALAudio(
    simultaneousSources = 16,    // max concurrent sounds
    deviceBufferCount  = 9,
    deviceBufferSize   = 512
);
```

**Source pool:** OpenAL has a fixed number of `ALSource` objects. When all 16 are in use, the oldest playing sound is stopped and its source is reused. WC3 has many simultaneous sounds (footsteps, spells, ambient) so 16 is a reasonable minimum; production games use 32-64.

**Playing a sound:**
```java
Sound sfx = audio.newSound(dataSource.getFile("Sound/Spells/...ogg"));
long id = sfx.play(volume);       // volume: 0.0 - 1.0
sfx.setPan(id, pan, volume);      // pan: -1.0 (left) to 1.0 (right)
sfx.stop(id);
```

**Playing music:**
```java
Music music = audio.newMusic(dataSource.getFile("Music/...mp3"));
music.setLooping(true);
music.setVolume(0.6f);
music.play();
```

---

## 3D Positional Audio

WC3 sound effects are positioned in world space. Warsmash uses OpenAL's 3D positioning:

```java
// AudioContext (per-Scene)
AudioContext context = Extensions.audio.createContext(isWorldScene);

// Listener position (camera)
context.listener.setPosition(camX, camY, camZ);
context.listener.setOrientation(forwardX, forwardY, forwardZ, upX, upY, upZ);

// Sound source position
AudioSource source = context.createSource();
source.setPosition(unitX, unitY, 0f);
source.setGain(volume);
source.play(soundBuffer);
```

Updated each frame in `Scene.update()`:
```java
listener.setPosition(camera.location.x, camera.location.y, camera.location.z);
listener.setOrientation(camera.directionY.x, ..., camera.directionZ.x, ...);
```

**Falloff model:** OpenAL inverse distance. WC3 sounds audible within ~1200 world units (~9 tiles), silent beyond ~3200 units.

---

## MDX Event Sounds

Sounds in animations are triggered by `EVTS` (Event Objects) in MDX models:

```
EventObject:
  name: "SND_FootstepDirt"    ← look up in sound table
  track: at frame 150ms       ← fire at this animation frame
```

When `MdxComplexInstance.update()` passes through frame 150ms, it fires:
```java
eventObject.fire(instance)
  → audioContext.playSound("Sound/FootSteps/footstepDirt.wav", position)
```

Sound tables map `SND_xxx` keys to actual file paths. Loaded from `Units/SoundData.slk`.

---

## WC3 Sound Data

Sound definitions in MPQ:

```
Units/SoundData.slk        Unit voice/sound mappings
Sound/
  Spells/                  Spell cast, impact sounds
  Units/
    Human/                 "Peasant" etc. voice lines (What, Yes, Pissed, Attack, ...)
  Interface/               UI sounds (click, error, ping)
  Music/                   Background music tracks
  Ambient/                 Environmental loops (birds, wind, etc.)
```

**Unit sounds** are looked up by SoundSet (from UnitData.slk) + event type:

| Event | Example file |
|-------|-------------|
| `What` | "PeasantWhat1.wav" |
| `Yes` | "PeasantYes1.wav" |
| `Ready` | "PeasantReady1.wav" |
| `Attack` | "PeasantAttack1.wav" |
| `Death` | "PeasantDeath1.wav" |
| `Pissed` | "PeasantPissed1.wav" |

Multiple numbered variants (1–N) are picked randomly by `CSimulation`.

---

## Audio Enable/Disable

```java
// Per-scene (for WorldScene ambient)
scene.enableAudio();    // starts AudioContext, sets audioEnabled
scene.disableAudio();   // suspends AudioContext

// Per-viewer
modelViewer.audioEnabled = true;  // global gate
```

`warsmash.ini` has `EnableMusic=0` — set to 1 to enable background music (disabled by default for faster startup and quieter development).

---

## Reimplementation Notes

For a Rust/Go rewrite, the audio stack is relatively self-contained:

1. **Use rodio (Rust) or a similar library** — handles WAV/OGG decoding natively
2. **FLAC** — `symphonia` crate handles FLAC natively without precision loss
3. **3D audio** — use `kira` or OpenAL bindings; WC3's falloff model is simple inverse distance
4. **Sound event system** — just a `HashMap<String, Vec<String>>` mapping SND keys to file paths, with a random picker

The most work is loading `SoundData.slk` and wiring up the MDX event callback system. The actual audio playback is standard.
