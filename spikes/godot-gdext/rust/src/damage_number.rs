use godot::prelude::*;
use godot::classes::{INode3D, Label3D, Node3D};

/// A self-deleting floating Label3D that drifts upward and fades out.
#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct DamageNumber {
    elapsed: f32,
    amount: i64,
    label: Option<Gd<Label3D>>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for DamageNumber {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            elapsed: 0.0,
            amount: 0,
            label: None,
            base,
        }
    }

    fn ready(&mut self) {
        let mut label = Label3D::new_alloc();
        label.set_text(&GString::from(format!("{}", self.amount).as_str()));
        label.set_font_size(64);
        label.set_modulate(Color::from_rgb(1.0, 1.0, 0.0));
        label.set_billboard_mode(godot::classes::base_material_3d::BillboardMode::ENABLED);
        label.set_name("Label");

        self.base_mut().add_child(&label);
        self.label = Some(label);
    }

    fn process(&mut self, delta: f64) {
        self.elapsed += delta as f32;
        let t = self.elapsed;

        // Drift up at 1.5 wu/s
        let mut pos = self.base().get_position();
        pos.y += 1.5 * delta as f32;
        self.base_mut().set_position(pos);

        // Fade out alpha = 1 - t/1.0
        if let Some(ref mut label) = self.label {
            let mut color = label.get_modulate();
            color.a = 1.0 - t;
            label.set_modulate(color);
        }

        if t >= 1.0 {
            self.base_mut().queue_free();
        }
    }
}

#[godot_api]
impl DamageNumber {
    #[func]
    pub fn set_amount(&mut self, amount: i64) {
        self.amount = amount;
    }
}
