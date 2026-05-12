/// 20KBC — Fyrox Spike
///
/// Validates Fyrox 0.34 as a rendering layer for a prehistoric RTS:
///   - `CSimulation` ticked at 20 Hz from a fixed-step accumulator
///   - Procedural 64×64 terrain mesh
///   - Unit nodes spawned/synced from simulation state
///   - RTS camera (WASD pan, scroll zoom)
///   - Day/night directional light cycle
///   - HUD: Wood / Stone / Tick counters
///
/// # Fyrox API notes
/// Fyrox 0.34 uses:
///   - `Executor` as the top-level runtime (replaces the older `Framework`).
///   - `Plugin` + `PluginConstructor` traits for game logic attachment.
///   - `PluginContext` provides mutable access to `scenes`, `user_interface`, `dt`.
///   - Input is delivered via `on_os_event` (not a polling `KeyboardState`).
///   - The window title is set via `executor.get_window().set_title(...)`.
mod camera;
mod day_night;
mod hud;
mod terrain;
mod units;

use std::collections::HashMap;

use fyrox::{
    core::pool::Handle,
    engine::executor::Executor,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    gui::message::UiMessage,
    plugin::{Plugin, PluginConstructor, PluginContext, PluginRegistrationContext},
    scene::Scene,
};
use game_core::{CSimulation, Order, SimConfig, UnitId};

use camera::{KeyInput, RtsCamera};
use day_night::DayNightCycle;
use hud::GameHud;

// ---------------------------------------------------------------------------
// Plugin constructor — Fyrox calls this once at startup to create our plugin.
// ---------------------------------------------------------------------------

pub struct GamePluginConstructor;

impl PluginConstructor for GamePluginConstructor {
    fn register(&self, _ctx: PluginRegistrationContext<'_>) {
        // Register custom script types here if using Fyroxed.
        // Not needed for this code-only spike.
    }

    fn create_instance(
        &self,
        _scene_path: Option<&str>,
        ctx: PluginContext,
    ) -> Box<dyn Plugin> {
        Box::new(GamePlugin::new(ctx))
    }
}

// ---------------------------------------------------------------------------
// Main plugin — owns simulation state and all renderer handles.
// ---------------------------------------------------------------------------

pub struct GamePlugin {
    sim: CSimulation,
    /// Accumulates real time to drive sim ticks at 20 Hz (50 ms each).
    sim_timer: f32,

    scene: Handle<Scene>,
    camera: RtsCamera,
    day_night: DayNightCycle,
    hud: GameHud,

    /// Maps `UnitId` → scene-graph `Handle<Node>` for position sync.
    unit_handles: HashMap<UnitId, Handle<fyrox::scene::node::Node>>,

    /// Accumulated scroll delta from `WindowEvent::MouseWheel`.
    scroll_delta: f32,
    /// Keyboard state for RTS camera pan.
    key_w: bool,
    key_a: bool,
    key_s: bool,
    key_d: bool,
}

impl GamePlugin {
    fn new(mut ctx: PluginContext) -> Self {
        // ---- Scene --------------------------------------------------------
        let mut scene = Scene::new();

        // ---- Terrain ------------------------------------------------------
        terrain::create_terrain_mesh(&mut scene);

        // ---- Camera -------------------------------------------------------
        let camera = RtsCamera::new(&mut scene);

        // ---- Day/Night ----------------------------------------------------
        let day_night = DayNightCycle::new(&mut scene);

        // ---- Simulation ---------------------------------------------------
        let sim = CSimulation::new(SimConfig::default());

        // ---- HUD ----------------------------------------------------------
        let hud = {
            let mut build_ctx = ctx.user_interface.build_ctx();
            GameHud::new(&mut build_ctx)
        };

        // ---- Register scene with engine -----------------------------------
        let scene_handle = ctx.scenes.add(scene);

        Self {
            sim,
            sim_timer: 0.0,
            scene: scene_handle,
            camera,
            day_night,
            hud,
            unit_handles: HashMap::new(),
            scroll_delta: 0.0,
            key_w: false,
            key_a: false,
            key_s: false,
            key_d: false,
        }
    }

    /// Sync simulation unit positions into the Fyrox scene graph.
    fn sync_units(&mut self, scene: &mut Scene) {
        // Collect cloned units so we don't hold a borrow on `self.sim` while
        // also borrowing `scene` mutably.
        let unit_snapshot: Vec<_> = self.sim.iter_units().cloned().collect();
        units::sync_units(scene, unit_snapshot.into_iter(), &mut self.unit_handles);
    }
}

// ---------------------------------------------------------------------------
// Plugin trait implementation
// ---------------------------------------------------------------------------

impl Plugin for GamePlugin {
    fn update(&mut self, ctx: &mut PluginContext) {
        let dt = ctx.dt;

        // ---- Simulation tick at 20 Hz ------------------------------------
        self.sim_timer += dt;
        while self.sim_timer >= 0.05 {
            self.sim_timer -= 0.05;
            self.sim.tick();
        }

        // ---- Get mutable scene reference ---------------------------------
        let scene = match ctx.scenes.try_get_mut(self.scene) {
            Some(s) => s,
            None => return,
        };

        // ---- Camera update -----------------------------------------------
        let keys = KeyInput {
            w: self.key_w,
            a: self.key_a,
            s: self.key_s,
            d: self.key_d,
        };
        self.camera.update(scene, dt, &keys, self.scroll_delta);
        self.scroll_delta = 0.0; // consume scroll delta

        // ---- Day/night ---------------------------------------------------
        self.day_night.update(scene, dt);

        // ---- Unit sync ---------------------------------------------------
        let unit_snapshot: Vec<_> = self.sim.iter_units().cloned().collect();
        units::sync_units(scene, unit_snapshot.into_iter(), &mut self.unit_handles);

        // ---- HUD update --------------------------------------------------
        let (wood, stone) = self.sim.player_resources(0);
        self.hud.update(ctx.user_interface, wood, stone, self.sim.tick);
    }

    fn on_ui_message(&mut self, _ctx: &mut PluginContext, _msg: &UiMessage) {
        // No UI interaction in this spike.
    }

    fn on_os_event(&mut self, event: &Event<()>, _ctx: PluginContext) {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::KeyboardInput { event, .. } => {
                    // winit 0.28+: KeyboardInput variant uses `event: KeyEvent` not `input`.
                    // TODO: map WASD via event.physical_key when winit API stabilises.
                    let _ = event;
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    use fyrox::event::MouseScrollDelta;
                    self.scroll_delta += match delta {
                        MouseScrollDelta::LineDelta(_, y) => *y,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.1,
                    };
                }
                _ => {}
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut window_attrs = fyrox::window::WindowAttributes::default();
    window_attrs.title = "20KBC — Fyrox Spike".to_string();
    let mut executor = Executor::from_params(
        event_loop,
        fyrox::engine::GraphicsContextParams {
            window_attributes: window_attrs,
            vsync: true,
        },
    );
    executor.add_plugin_constructor(GamePluginConstructor);
    executor.run();
}
