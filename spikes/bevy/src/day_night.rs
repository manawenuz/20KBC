use bevy::prelude::*;

/// Marks the directional light that drives the day/night cycle.
#[derive(Component)]
pub struct DayNightLight;

/// Cycle period in seconds (600 s = 10 min per full day).
#[derive(Resource)]
pub struct DayNightCycle {
    pub period_secs: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self { period_secs: 600.0 }
    }
}

pub struct DayNightPlugin;

impl Plugin for DayNightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DayNightCycle>()
            .add_systems(Startup, spawn_sun)
            .add_systems(Update, update_day_night);
    }
}

fn spawn_sun(mut commands: Commands) {
    commands.spawn((
        DayNightLight,
        DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        // Angle the light like afternoon sun.
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.5, 0.0)),
    ));

    // Ambient light for the night minimum.
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.2, 0.2, 0.35),
        brightness: 100.0,
    });
}

fn update_day_night(
    time: Res<Time>,
    cycle: Res<DayNightCycle>,
    mut query: Query<(&mut DirectionalLight, &mut Transform), With<DayNightLight>>,
) {
    let t = (time.elapsed_secs() / cycle.period_secs).fract();

    // Illuminance: noon = 15000, midnight = 500.
    let illuminance = lerp(500.0, 15_000.0, noon_curve(t));

    // Rotate the sun around the X axis over the full cycle.
    let sun_angle = t * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;

    for (mut light, mut transform) in &mut query {
        light.illuminance = illuminance;
        *transform =
            Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, sun_angle, 0.5, 0.0));
    }
}

/// Smooth noon curve: peaks at t=0.5 (midday), falls to 0 at t=0 and t=1.
#[inline]
fn noon_curve(t: f32) -> f32 {
    // Sinusoidal: sin(π·t) gives 0→1→0 over [0,1].
    (t * std::f32::consts::PI).sin().max(0.0)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
