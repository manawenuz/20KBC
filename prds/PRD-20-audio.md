# PRD-20 — WC3 Audio Extraction + Sound Hooks

## Goal

Extract 3 short sound effects from War3.mpq, commit them as `.ogg` or
`.wav` under `spikes/godot-gdext/assets/audio/`, and add a Rust
`SoundFx` autoload node with `#[func]`s to play each:
- `play_chop()` — worker chopping wood
- `play_hit()` — generic combat hit
- `play_footstep()` — worker footstep (volume-attenuated by camera distance — but for MVP, just play at full volume)

## Context

WC3 stores audio under `Sound/` in the MPQ — typically `.wav` files,
some compressed with WC3's ADPCM variant. The existing `extract.py` +
StormLib should pull the raw bytes. Then either:
- ship as-is if `.wav`
- transcode to `.ogg` if ADPCM-compressed via `ffmpeg`

Target paths (use any equivalent if these don't exist):
- `Sound/Units/Human/Peasant/PeasantPissed3.wav` (or any short peasant grunt) → `worker_grunt.wav`
- `Sound/UI/MetalLightChop1.wav` (or any chop-like) → `chop.wav`
- `Sound/Combat/MetalHeavyChopFlesh1.wav` (or any short hit) → `hit.wav`
- `Sound/Footsteps/HumanFootstepDirt1.wav` (or any short footstep) → `footstep.wav`

If extraction fails, ship a tiny synthesized placeholder (e.g.
`pip install numpy` and write a sine-wave .wav) so the SoundFx node
has something to load.

## Files you MAY create

- `spikes/godot-gdext/rust/src/sound_fx.rs`
- `spikes/godot-gdext/assets/audio/*.wav` or `.ogg`
- `scripts/asset-extract/extract_audio.py` (or extend existing extract.py)

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod sound_fx;` only
- `scripts/asset-extract/extract.py` (optional: add audio subcommand)

## Files you MUST NOT touch

- `main.gd`, `Main.tscn`, `project.godot`
- Other Rust source

## Interface contract

```rust
// sound_fx.rs
use godot::prelude::*;
use godot::classes::{AudioStream, AudioStreamPlayer, INode, Node, ResourceLoader};

#[derive(GodotClass)]
#[class(base = Node)]
pub struct SoundFx {
    chop: Option<Gd<AudioStream>>,
    hit: Option<Gd<AudioStream>>,
    footstep: Option<Gd<AudioStream>>,
    player: Option<Gd<AudioStreamPlayer>>,
    base: Base<Node>,
}

#[godot_api]
impl INode for SoundFx {
    fn init(base: Base<Node>) -> Self {
        Self { chop: None, hit: None, footstep: None, player: None, base }
    }
    fn ready(&mut self) {
        // Try to load each asset path; tolerate missing files.
        let mut rl = ResourceLoader::singleton();
        self.chop = rl.load("res://assets/audio/chop.wav")
            .and_then(|r| r.try_cast::<AudioStream>().ok());
        self.hit = rl.load("res://assets/audio/hit.wav")
            .and_then(|r| r.try_cast::<AudioStream>().ok());
        self.footstep = rl.load("res://assets/audio/footstep.wav")
            .and_then(|r| r.try_cast::<AudioStream>().ok());

        let mut p = AudioStreamPlayer::new_alloc();
        self.base_mut().add_child(&p);
        self.player = Some(p);
    }
}

#[godot_api]
impl SoundFx {
    #[func] pub fn play_chop(&mut self)     { self.play(self.chop.clone()); }
    #[func] pub fn play_hit(&mut self)      { self.play(self.hit.clone()); }
    #[func] pub fn play_footstep(&mut self) { self.play(self.footstep.clone()); }
}

impl SoundFx {
    fn play(&mut self, stream: Option<Gd<AudioStream>>) {
        if let (Some(s), Some(p)) = (stream, self.player.as_mut()) {
            p.set_stream(&s);
            p.play();
        }
    }
}
```

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
ls spikes/godot-gdext/assets/audio/
```

- [ ] Clean build
- [ ] At least 3 of the 4 audio files exist and are non-zero size
- [ ] `sound_fx.rs` exists; `lib.rs` has the new mod
- [ ] ≤ 4 files modified outside the new assets/audio directory

## Out of scope

- Positional 3D audio (use plain AudioStreamPlayer, not AudioStreamPlayer3D)
- Music
- Voiceover lines
