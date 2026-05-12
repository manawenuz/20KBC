use godot::prelude::*;
use godot::classes::{DirectionalLight3D, INode, Node};

/// Drives a DirectionalLight3D through a 10-minute day/night cycle.
///
/// The `sun_path` export must point at the scene's DirectionalLight3D.
/// Energy and colour are lerped across five keyframes: night → dawn → noon → dusk → night.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct DayNightController {
    /// NodePath (set in editor) pointing at the DirectionalLight3D to drive.
    #[export]
    pub sun_path: NodePath,
    /// Cycle length in seconds (default 600 = 10 minutes).
    #[export]
    pub cycle_seconds: f32,
    /// Current time-of-day in [0, 1). 0.25 = noon, 0.75 = midnight.
    pub time_of_day: f32,
    base: Base<Node>,
}

#[godot_api]
impl INode for DayNightController {
    fn init(base: Base<Node>) -> Self {
        Self {
            sun_path: NodePath::default(),
            cycle_seconds: 600.0,
            time_of_day: 0.30, // start a little after dawn
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        // Advance time_of_day, wrap to [0, 1).
        self.time_of_day += delta as f32 / self.cycle_seconds;
        self.time_of_day = self.time_of_day.rem_euclid(1.0);

        let Some(mut sun) = self.lookup_sun() else {
            return;
        };

        let (energy, color) = sample_energy_and_color(self.time_of_day);
        sun.set("light_energy", &energy.to_variant());
        sun.set_color(color);
    }
}

#[godot_api]
impl DayNightController {
    /// Force-set time_of_day (useful for tests or skip-to-night debug).
    #[func]
    pub fn set_time_of_day(&mut self, t: f32) {
        self.time_of_day = t.rem_euclid(1.0);
    }
}

impl DayNightController {
    fn lookup_sun(&self) -> Option<Gd<DirectionalLight3D>> {
        self.base()
            .get_node_or_null(&self.sun_path)?
            .try_cast::<DirectionalLight3D>()
            .ok()
    }
}

/// Piecewise-linear interpolation of energy and colour across the day.
///
/// Keyframes (time → energy, colour):
/// | 0.00 | night | 0.10 | (0.30, 0.40, 0.60) |
/// | 0.20 | dawn  | 0.30 | (1.00, 0.60, 0.30) |
/// | 0.25 | noon  | 1.40 | (1.00, 0.95, 0.85) |
/// | 0.70 | dusk  | 0.30 | (1.00, 0.55, 0.25) |
/// | 0.80 | night | 0.10 | (0.30, 0.40, 0.60) |
fn sample_energy_and_color(t: f32) -> (f32, Color) {
    // (time, energy, r, g, b)
    const K: [(f32, f32, f32, f32, f32); 5] = [
        (0.00, 0.10, 0.30, 0.40, 0.60),
        (0.20, 0.30, 1.00, 0.60, 0.30),
        (0.25, 1.40, 1.00, 0.95, 0.85),
        (0.70, 0.30, 1.00, 0.55, 0.25),
        (0.80, 0.10, 0.30, 0.40, 0.60),
    ];

    // Find the interval that contains t.
    for i in 0..K.len() - 1 {
        let (t0, e0, r0, g0, b0) = K[i];
        let (t1, e1, r1, g1, b1) = K[i + 1];
        if t >= t0 && t < t1 {
            let f = (t - t0) / (t1 - t0);
            return (
                lerp(e0, e1, f),
                Color::from_rgb(lerp(r0, r1, f), lerp(g0, g1, f), lerp(b0, b1, f)),
            );
        }
    }

    // Wrap: t is in [0.80, 1.00).  Next keyframe is 1.00 which maps to 0.00.
    let f = (t - 0.80) / 0.20;
    let (_, e0, r0, g0, b0) = K[4];
    let (_, e1, r1, g1, b1) = K[0];
    (
        lerp(e0, e1, f),
        Color::from_rgb(lerp(r0, r1, f), lerp(g0, g1, f), lerp(b0, b1, f)),
    )
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
