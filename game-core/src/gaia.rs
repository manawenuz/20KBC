use glam::Vec2;
use crate::unit::{CUnit, UnitId};
use crate::rng::DeterministicRng;

pub type GaiaId = u32;

#[derive(Clone, Debug)]
pub enum GaiaBehavior {
    Roaming {
        waypoint: Vec2,
        /// Ticks remaining at current waypoint before picking a new one.
        idle_ticks: u32,
    },
    Chasing {
        target: UnitId,
    },
    Attacking {
        target: UnitId,
        cooldown: f32,
    },
    Returning,
}

#[derive(Clone, Debug)]
pub struct CGaiaEntity {
    pub id: GaiaId,
    pub pos: Vec2,
    pub territory_center: Vec2,
    pub territory_radius: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub behavior: GaiaBehavior,
}

impl CGaiaEntity {
    pub const MOVE_SPEED: f32 = 1.5 / 20.0; // 1.5 world-units/s at 20 tps
    pub const ATTACK_RANGE: f32 = 1.5;
    pub const ATTACK_DAMAGE: f32 = 12.0;
    pub const ATTACK_COOLDOWN_TICKS: f32 = 20.0; // 1 attack/s
    pub const DETECT_RADIUS: f32 = 50.0;
    pub const LEASH_MULTIPLIER: f32 = 2.0; // abandon chase when target > territory_radius * 2
    pub const ROAM_IDLE_TICKS: u32 = 100; // 5 s at 20 tps before picking new waypoint

    pub fn new(id: GaiaId, pos: Vec2, territory_center: Vec2, territory_radius: f32) -> Self {
        let initial_waypoint = territory_center;
        Self {
            id,
            pos,
            territory_center,
            territory_radius,
            hp: 60.0,
            max_hp: 60.0,
            behavior: GaiaBehavior::Roaming {
                waypoint: initial_waypoint,
                idle_ticks: Self::ROAM_IDLE_TICKS,
            },
        }
    }
}

/// Advance one GAIA entity by one tick.
/// `units` slice is borrowed mutably so we can apply attack damage.
pub fn update_gaia(entity: &mut CGaiaEntity, units: &mut Vec<CUnit>, rng: &mut DeterministicRng) {
    if entity.hp <= 0.0 {
        return;
    }

    // --- Find closest living player unit inside territory detection radius ---
    let closest_player_unit: Option<(UnitId, f32)> = units
        .iter()
        .filter(|u| !u.is_dead && u.owner != u8::MAX) // owner u8::MAX reserved for GAIA
        .map(|u| {
            let d = entity.pos.distance(u.pos);
            (u.id, d)
        })
        .filter(|(_, d)| *d <= entity.territory_radius)
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let new_behavior = match &entity.behavior {
        GaiaBehavior::Roaming { waypoint, idle_ticks } => {
            // Check if a unit entered territory — switch to chasing.
            if let Some((uid, _)) = closest_player_unit {
                Some(GaiaBehavior::Chasing { target: uid })
            } else {
                let dist_to_wp = entity.pos.distance(*waypoint);
                if dist_to_wp < CGaiaEntity::MOVE_SPEED * 1.5 || *idle_ticks == 0 {
                    // Pick a new random waypoint inside territory.
                    let angle = rng.next_range(0.0, std::f32::consts::TAU);
                    let radius = rng.next_range(0.0, entity.territory_radius * 0.8);
                    let new_wp = entity.territory_center
                        + Vec2::new(angle.cos(), angle.sin()) * radius;
                    Some(GaiaBehavior::Roaming {
                        waypoint: new_wp,
                        idle_ticks: CGaiaEntity::ROAM_IDLE_TICKS,
                    })
                } else {
                    // Move toward waypoint.
                    move_toward(&mut entity.pos, *waypoint, CGaiaEntity::MOVE_SPEED);
                    Some(GaiaBehavior::Roaming {
                        waypoint: *waypoint,
                        idle_ticks: idle_ticks.saturating_sub(1),
                    })
                }
            }
        }

        GaiaBehavior::Chasing { target } => {
            let target_id = *target;
            let target_unit = units.iter().find(|u| u.id == target_id);
            match target_unit {
                None => Some(GaiaBehavior::Returning),
                Some(tu) if tu.is_dead => Some(GaiaBehavior::Returning),
                Some(tu) => {
                    let dist_to_target = entity.pos.distance(tu.pos);
                    let leash = entity.territory_radius * CGaiaEntity::LEASH_MULTIPLIER;
                    if dist_to_target > leash {
                        Some(GaiaBehavior::Returning)
                    } else if dist_to_target <= CGaiaEntity::ATTACK_RANGE {
                        Some(GaiaBehavior::Attacking {
                            target: target_id,
                            cooldown: 0.0,
                        })
                    } else {
                        let target_pos = tu.pos;
                        move_toward(&mut entity.pos, target_pos, CGaiaEntity::MOVE_SPEED);
                        None
                    }
                }
            }
        }

        GaiaBehavior::Attacking { target, cooldown } => {
            let target_id = *target;
            let mut new_cooldown = (cooldown - 1.0).max(0.0);
            let target_unit = units.iter().find(|u| u.id == target_id);
            match target_unit {
                None => Some(GaiaBehavior::Returning),
                Some(tu) if tu.is_dead => Some(GaiaBehavior::Returning),
                Some(tu) => {
                    let dist = entity.pos.distance(tu.pos);
                    if dist > CGaiaEntity::ATTACK_RANGE {
                        Some(GaiaBehavior::Chasing { target: target_id })
                    } else {
                        // Apply damage when cooldown expires.
                        if new_cooldown <= 0.0 {
                            if let Some(tu_mut) = units.iter_mut().find(|u| u.id == target_id) {
                                tu_mut.hp -= CGaiaEntity::ATTACK_DAMAGE;
                                if tu_mut.hp <= 0.0 {
                                    tu_mut.hp = 0.0;
                                    tu_mut.is_dead = true;
                                }
                            }
                            new_cooldown = CGaiaEntity::ATTACK_COOLDOWN_TICKS;
                        }
                        Some(GaiaBehavior::Attacking {
                            target: target_id,
                            cooldown: new_cooldown,
                        })
                    }
                }
            }
        }

        GaiaBehavior::Returning => {
            let dist = entity.pos.distance(entity.territory_center);
            if dist < CGaiaEntity::MOVE_SPEED * 1.5 {
                let wp = entity.territory_center;
                Some(GaiaBehavior::Roaming {
                    waypoint: wp,
                    idle_ticks: CGaiaEntity::ROAM_IDLE_TICKS,
                })
            } else {
                let tc = entity.territory_center;
                move_toward(&mut entity.pos, tc, CGaiaEntity::MOVE_SPEED);
                None
            }
        }
    };

    if let Some(nb) = new_behavior {
        entity.behavior = nb;
    }
}

#[inline]
fn move_toward(pos: &mut Vec2, target: Vec2, speed: f32) {
    let delta = target - *pos;
    let dist = delta.length();
    if dist > speed {
        *pos += delta / dist * speed;
    } else {
        *pos = target;
    }
}
