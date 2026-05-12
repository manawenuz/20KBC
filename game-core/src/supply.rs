use crate::unit::CUnit;
use crate::player::CPlayer;
use crate::config::SimConfig;

/// Called once per tick for each living unit.
/// Drains supply when near a depot, drains HP when unsupplied, auto-heals when supplied.
pub fn update_supply(unit: &mut CUnit, player: &CPlayer, config: &SimConfig) {
    if unit.is_dead {
        return;
    }

    let in_range = player
        .supply_depot
        .map_or(false, |depot| unit.pos.distance(depot) <= config.supply_range);

    if in_range && player.food > 0 {
        // Drain supply reserve; clamp to 0.
        unit.supply -= config.supply_drain_per_tick;
        if unit.supply < 0.0 {
            unit.supply = 0.0;
        }

        // Auto-heal only when supplied.
        if unit.supply > 0.0 {
            unit.hp = (unit.hp + config.auto_heal_per_tick).min(unit.max_hp);
        }
    } else {
        // No supply → HP drains at 3× drain rate.
        unit.hp -= config.hp_drain_per_tick;
    }

    if unit.hp <= 0.0 {
        unit.hp = 0.0;
        unit.is_dead = true;
    }
}
