use godot::prelude::*;
use godot::classes::{Camera3D, ICamera3D, Input, InputEvent, InputEventMouseButton};
use godot::global::{Key, MouseButton};

/// RTS bird's-eye camera.
///
/// Controls:
/// - WASD or arrow keys → pan in camera-local XZ plane (rotation-aware).
/// - Q / E → rotate the camera yaw around the look target (also bound to
///   mouse-wheel up/down for convenience).
/// - `+` / `-` (and `=` / `_`) → zoom in / out by lifting / lowering the
///   camera along its own view direction.
/// - PageUp / PageDown → coarse zoom.
///
/// The camera orbits a virtual focus point on the ground plane. Panning
/// translates the focus along the world XZ axes (rotated by current yaw),
/// rotating spins around that focus, zooming brings the camera closer or
/// further from it along the view axis. This is the classic RTS feel.
#[derive(GodotClass)]
#[class(base = Camera3D)]
pub struct RtsCameraController {
    /// World-units per second lateral pan speed.
    pan_speed: f32,
    /// Radians per second yaw rate when Q/E held.
    yaw_speed: f32,
    /// Radians yaw delta per mouse-wheel notch.
    wheel_yaw_step: f32,
    /// Distance from camera to focus point along the view ray.
    distance: f32,
    /// World-units-per-second zoom rate when +/- held.
    zoom_speed: f32,
    /// Wheel-keyboard fallback zoom step (PageUp/PageDown).
    zoom_step: f32,
    /// Min / max camera-to-focus distance.
    zoom_min: f32,
    zoom_max: f32,
    /// Yaw rotation in radians (around world Y).
    yaw: f32,
    /// Constant pitch (radians). Negative = looking down.
    pitch: f32,
    /// Ground-plane focus point.
    focus: Vector3,
    base: Base<Camera3D>,
}

#[godot_api]
impl ICamera3D for RtsCameraController {
    fn init(base: Base<Camera3D>) -> Self {
        Self {
            pan_speed: 30.0,
            yaw_speed: 2.0,
            wheel_yaw_step: std::f32::consts::FRAC_PI_8, // ~22.5° per wheel tick
            distance: 35.0,
            zoom_speed: 30.0,
            zoom_step: 5.0,
            zoom_min: 8.0,
            zoom_max: 100.0,
            yaw: 0.0,
            pitch: -std::f32::consts::FRAC_PI_3, // -60° looking down
            focus: Vector3::new(64.0, 0.0, 64.0), // map centre on ground plane
            base,
        }
    }

    fn ready(&mut self) {
        self.apply_transform();
    }

    fn process(&mut self, delta: f64) {
        let delta = delta as f32;
        let input = Input::singleton();

        // --- Pan input (WASD + arrow keys) -------------------------------
        let mut pan_local = Vector3::ZERO; // (right, 0, forward) in camera local axes
        if input.is_key_pressed(Key::W) || input.is_action_pressed("ui_up") {
            pan_local.z -= 1.0;
        }
        if input.is_key_pressed(Key::S) || input.is_action_pressed("ui_down") {
            pan_local.z += 1.0;
        }
        if input.is_key_pressed(Key::A) || input.is_action_pressed("ui_left") {
            pan_local.x -= 1.0;
        }
        if input.is_key_pressed(Key::D) || input.is_action_pressed("ui_right") {
            pan_local.x += 1.0;
        }

        if pan_local != Vector3::ZERO {
            // Rotate pan into world space using current yaw.
            let (sy, cy) = self.yaw.sin_cos();
            let world_dx = pan_local.x * cy + pan_local.z * sy;
            let world_dz = -pan_local.x * sy + pan_local.z * cy;
            let speed = self.pan_speed * delta;
            self.focus.x += world_dx * speed;
            self.focus.z += world_dz * speed;
        }

        // --- Rotate input (Q / E) ----------------------------------------
        if input.is_key_pressed(Key::Q) {
            self.yaw -= self.yaw_speed * delta;
        }
        if input.is_key_pressed(Key::E) {
            self.yaw += self.yaw_speed * delta;
        }

        // --- Zoom input (+ / -) -------------------------------------------
        if input.is_key_pressed(Key::EQUAL) || input.is_key_pressed(Key::PLUS) {
            self.distance = (self.distance - self.zoom_speed * delta).max(self.zoom_min);
        }
        if input.is_key_pressed(Key::MINUS) {
            self.distance = (self.distance + self.zoom_speed * delta).min(self.zoom_max);
        }
        if input.is_key_pressed(Key::PAGEUP) {
            self.distance = (self.distance - self.zoom_step * delta * 10.0).max(self.zoom_min);
        }
        if input.is_key_pressed(Key::PAGEDOWN) {
            self.distance = (self.distance + self.zoom_step * delta * 10.0).min(self.zoom_max);
        }

        self.apply_transform();
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        // Mouse-wheel now rotates yaw instead of zooming (per design).
        if let Ok(mouse_btn) = event.clone().try_cast::<InputEventMouseButton>() {
            if mouse_btn.is_pressed() {
                match mouse_btn.get_button_index() {
                    MouseButton::WHEEL_UP => self.yaw -= self.wheel_yaw_step,
                    MouseButton::WHEEL_DOWN => self.yaw += self.wheel_yaw_step,
                    _ => {}
                }
                self.apply_transform();
            }
        }
    }
}

#[godot_api]
impl RtsCameraController {
    /// Console / debug accessor — set zoom distance directly.
    #[func]
    pub fn set_zoom(&mut self, distance: f32) {
        self.distance = distance.clamp(self.zoom_min, self.zoom_max);
        self.apply_transform();
    }

    /// Console / debug accessor — set yaw in radians.
    #[func]
    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
        self.apply_transform();
    }

    /// Console / debug accessor — center the camera on a world XZ point.
    #[func]
    pub fn focus_on(&mut self, x: f32, z: f32) {
        self.focus.x = x;
        self.focus.z = z;
        self.apply_transform();
    }
}

impl RtsCameraController {
    /// Recompute camera Transform3D from yaw/pitch/distance/focus.
    fn apply_transform(&mut self) {
        // Spherical-coords offset from focus.
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let dir_x = sy * cp;
        let dir_y = sp;
        let dir_z = cy * cp;
        let offset = Vector3::new(dir_x, dir_y, dir_z) * self.distance;
        let cam_pos = self.focus - offset;
        let rot = Vector3::new(self.pitch, self.yaw, 0.0);

        let mut base = self.base_mut();
        base.set_position(cam_pos);
        base.set_rotation(rot);
    }
}
