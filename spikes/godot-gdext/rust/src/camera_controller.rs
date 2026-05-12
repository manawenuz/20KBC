use godot::prelude::*;
use godot::classes::{Camera3D, ICamera3D, Input, InputEvent, InputEventMouseButton};
use godot::global::{Key, MouseButton};

/// RTS bird's-eye camera with WASD pan and scroll-wheel zoom.
///
/// Default orientation: 60° pitch tilt, facing south (negative Z), positioned
/// north-west of the map centre.  These values give a classic RTS perspective
/// over a 64×64 tile map (128×128 world units).
///
/// Pan speed is expressed in world-units per second so it feels consistent
/// regardless of frame rate.  Zoom clamps between 10 and 80 world-units of
/// height, matching the spec.
#[derive(GodotClass)]
#[class(base = Camera3D)]
pub struct RtsCameraController {
    /// World-units per second lateral pan speed.
    pan_speed: f32,
    /// Height delta applied per scroll tick.
    zoom_step: f32,
    /// Minimum camera height above y=0.
    zoom_min: f32,
    /// Maximum camera height above y=0.
    zoom_max: f32,
    base: Base<Camera3D>,
}

#[godot_api]
impl ICamera3D for RtsCameraController {
    fn init(base: Base<Camera3D>) -> Self {
        Self {
            pan_speed: 30.0,
            zoom_step: 4.0,
            zoom_min: 10.0,
            zoom_max: 80.0,
            base,
        }
    }

    fn ready(&mut self) {
        // Position camera north-west of map centre (64 tiles × 2 wu = 128 wu map).
        // 60° tilt gives good RTS perspective.
        self.base_mut().set_position(Vector3::new(64.0, 50.0, 100.0));
        self.base_mut().set_rotation(Vector3::new(
            -std::f32::consts::FRAC_PI_3, // -60° pitch (looking down)
            0.0,
            0.0,
        ));
    }

    fn process(&mut self, delta: f64) {
        let delta = delta as f32;
        let input = Input::singleton();

        // --- Pan (WASD / arrow keys) ---
        let mut pan = Vector3::ZERO;

        if input.is_key_pressed(Key::W) || input.is_action_pressed("ui_up") {
            pan.z -= 1.0;
        }
        if input.is_key_pressed(Key::S) || input.is_action_pressed("ui_down") {
            pan.z += 1.0;
        }
        if input.is_key_pressed(Key::A) || input.is_action_pressed("ui_left") {
            pan.x -= 1.0;
        }
        if input.is_key_pressed(Key::D) || input.is_action_pressed("ui_right") {
            pan.x += 1.0;
        }

        if pan != Vector3::ZERO {
            let pan_speed = self.pan_speed;
            let current = self.base().get_position();
            self.base_mut()
                .set_position(current + pan.normalized() * pan_speed * delta);
        }
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        // Scroll wheel zoom — adjust height, keep tilt constant.
        if let Ok(mouse_btn) = event.clone().try_cast::<InputEventMouseButton>() {
            let btn = mouse_btn.get_button_index();
            if mouse_btn.is_pressed() {
                let mut pos = self.base().get_position();
                match btn {
                    MouseButton::WHEEL_UP => {
                        pos.y = (pos.y - self.zoom_step).max(self.zoom_min);
                        // Pull Z forward proportionally to maintain look target.
                        pos.z = (pos.z - self.zoom_step * 0.6).max(0.0);
                    }
                    MouseButton::WHEEL_DOWN => {
                        pos.y = (pos.y + self.zoom_step).min(self.zoom_max);
                        pos.z = (pos.z + self.zoom_step * 0.6)
                            .min(self.zoom_max * 2.0);
                    }
                    _ => {}
                }
                self.base_mut().set_position(pos);
            }
        }
    }
}
