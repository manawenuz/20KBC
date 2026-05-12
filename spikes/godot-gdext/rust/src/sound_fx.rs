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

        let p = AudioStreamPlayer::new_alloc();
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
