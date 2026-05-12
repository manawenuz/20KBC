use godot::prelude::*;
use godot::classes::{Control, IControl, Label, Engine};

#[derive(GodotClass)]
#[class(base = Control)]
pub struct FrameTimeLabel {
    /// Update every N frames to avoid distracting flicker.
    #[export] pub refresh_frames: u32,
    counter: u32,
    base: Base<Control>,
}

#[godot_api]
impl IControl for FrameTimeLabel {
    fn init(base: Base<Control>) -> Self {
        Self { refresh_frames: 15, counter: 0, base }
    }

    fn process(&mut self, _delta: f64) {
        self.counter = self.counter.wrapping_add(1);
        if self.counter % self.refresh_frames.max(1) != 0 { return; }
        let fps = Engine::singleton().get_frames_per_second() as i64;
        let ms = if fps > 0 { 1000.0 / fps as f32 } else { 0.0 };
        if let Some(mut label) = self.base().get_node_or_null("Label")
            .and_then(|n| n.try_cast::<Label>().ok())
        {
            label.set_text(&format!("FPS: {}  ({:.1} ms)", fps, ms));
        }
    }
}
