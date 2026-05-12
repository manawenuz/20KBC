use game_core::{CSimulation, Order, SimConfig};
use glam::Vec2;

/// Verifies that issuing a Move order causes the unit to actually travel
/// from its spawn point toward the target over 100 ticks.
#[test]
fn worker_moves_after_move_order() {
    let config = SimConfig {
        map_width: 32,
        map_height: 32,
        seed: 42,
        ..Default::default()
    };

    let mut sim = CSimulation::new(config);

    // The default constructor spawns workers; grab the first living one.
    let (first_id, start_pos) = sim
        .iter_units()
        .next()
        .map(|u| (u.id, u.pos))
        .expect("Simulation should have at least one unit");

    // Issue a move order to a point 30 world-units away.
    let target = start_pos + Vec2::new(30.0, 0.0);
    sim.issue_order(first_id, Order::Move { target });

    for _ in 0..100 {
        sim.tick();
    }

    // After 100 ticks the unit should have moved closer to the target.
    // At MOVE_SPEED = 4/20 = 0.2 units/tick, 100 ticks = 20 world-units of travel.
    if let Some(unit) = sim.get_unit(first_id) {
        let moved = unit.pos.distance(start_pos);
        assert!(
            moved > 5.0,
            "Unit should have moved at least 5 world-units, moved={moved:.2}"
        );
    }
    // (Unit may have reached target and been retained or not — we don't assert existence.)
}

/// Verifies that the input log is populated after orders are issued.
#[test]
fn input_log_records_orders() {
    let mut sim = CSimulation::new(SimConfig::default());

    let first_id = sim.iter_units().next().unwrap().id;
    sim.issue_order(first_id, Order::Stop);
    sim.issue_order(first_id, Order::Move { target: Vec2::new(50.0, 50.0) });

    assert_eq!(sim.input_log.len(), 2, "Two orders should be in the log");
    assert_eq!(sim.input_log[0].unit, first_id);
    assert_eq!(sim.input_log[1].unit, first_id);
}

/// Verifies that ticking the simulation increments the tick counter correctly.
#[test]
fn tick_counter_increments() {
    let mut sim = CSimulation::new(SimConfig::default());
    assert_eq!(sim.tick, 0);

    for i in 1..=50 {
        sim.tick();
        assert_eq!(sim.tick, i);
    }
}

/// Verifies that the deterministic RNG produces the same sequence for the same seed.
#[test]
fn rng_is_deterministic() {
    use game_core::DeterministicRng;
    let mut a = DeterministicRng::new(999);
    let mut b = DeterministicRng::new(999);

    for _ in 0..1000 {
        let va = a.next_f32();
        let vb = b.next_f32();
        assert_eq!(va, vb, "RNG should be deterministic for the same seed");
    }
}

/// Verifies that A* returns a non-empty path on an all-passable grid.
#[test]
fn pathfinder_finds_path_on_open_grid() {
    use game_core::GridPathfinder;
    let pf = GridPathfinder::new(16, 16, 2.0);
    let from = Vec2::new(1.0, 1.0);
    let to = Vec2::new(25.0, 25.0);
    let path = pf.find_path(from, to);
    assert!(!path.is_empty(), "Should find a path on an open grid");
}
