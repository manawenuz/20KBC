use glam::Vec2;
use std::f32::consts::TAU;

/// Place `count` units in concentric rings around `center`.
/// Returns positions ordered such that index 0 ≈ closest to center.
///
/// Layout: 1 unit at center, then 6 in first ring at radius `spacing`,
/// then 12 in second ring at radius 2*spacing, etc.
pub fn formation_positions(center: Vec2, count: usize, spacing: f32) -> Vec<Vec2> {
    let mut positions = Vec::with_capacity(count);
    if count == 0 {
        return positions;
    }

    // Ring 0: center slot
    positions.push(center);
    if positions.len() >= count {
        return positions;
    }

    let mut ring = 1usize;
    while positions.len() < count {
        let slots_in_ring = 6 * ring;
        let radius = ring as f32 * spacing;
        for i in 0..slots_in_ring {
            let angle = TAU * (i as f32) / (slots_in_ring as f32);
            let offset = Vec2::new(angle.cos() * radius, angle.sin() * radius);
            positions.push(center + offset);
            if positions.len() >= count {
                break;
            }
        }
        ring += 1;
    }

    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formation_centered_first() {
        let p = formation_positions(Vec2::new(10.0, 10.0), 1, 2.0);
        assert_eq!(p, vec![Vec2::new(10.0, 10.0)]);
    }

    #[test]
    fn formation_rings_outward() {
        let p = formation_positions(Vec2::new(0.0, 0.0), 7, 2.0);
        assert_eq!(p.len(), 7);
        assert_eq!(p[0], Vec2::ZERO);
        for q in &p[1..] {
            let d = q.length();
            assert!((d - 2.0).abs() < 0.01);
        }
    }
}
