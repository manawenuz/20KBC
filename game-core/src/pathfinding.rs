use glam::Vec2;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Grid-based A* pathfinder. Cells are `cell_size` world-units wide.
/// `grid[y * width + x] = true` means the cell is passable.
pub struct GridPathfinder {
    pub grid: Vec<bool>,
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
}

// ── Internal A* node ────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
struct AStarNode {
    f: f32, // g + h
    g: f32,
    x: u32,
    y: u32,
}

impl Eq for AStarNode {}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse so BinaryHeap is a min-heap on f.
        other.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ── GridPathfinder impl ──────────────────────────────────────────────────────

impl GridPathfinder {
    pub fn new(width: u32, height: u32, cell_size: f32) -> Self {
        let size = (width * height) as usize;
        Self {
            grid: vec![true; size],
            width,
            height,
            cell_size,
        }
    }

    fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn is_passable(&self, wx: f32, wy: f32) -> bool {
        let (cx, cy) = self.world_to_cell(wx, wy);
        if cx >= self.width || cy >= self.height {
            return false;
        }
        self.grid[self.idx(cx, cy)]
    }

    pub fn set_blocked(&mut self, wx: f32, wy: f32, blocked: bool) {
        let (cx, cy) = self.world_to_cell(wx, wy);
        if cx < self.width && cy < self.height {
            let i = self.idx(cx, cy);
            self.grid[i] = !blocked;
        }
    }

    fn world_to_cell(&self, wx: f32, wy: f32) -> (u32, u32) {
        let cx = (wx / self.cell_size).floor() as i64;
        let cy = (wy / self.cell_size).floor() as i64;
        (cx.max(0) as u32, cy.max(0) as u32)
    }

    fn cell_center(&self, cx: u32, cy: u32) -> Vec2 {
        Vec2::new(
            cx as f32 * self.cell_size + self.cell_size * 0.5,
            cy as f32 * self.cell_size + self.cell_size * 0.5,
        )
    }

    /// 8-directional A*. Returns world-space waypoints. Empty if no path found.
    pub fn find_path(&self, from: Vec2, to: Vec2) -> Vec<Vec2> {
        let (sx, sy) = self.world_to_cell(from.x, from.y);
        let (gx, gy) = self.world_to_cell(to.x, to.y);

        let w = self.width as usize;
        let h = self.height as usize;
        let total = w * h;

        if sx >= self.width || sy >= self.height || gx >= self.width || gy >= self.height {
            return vec![];
        }

        let start_idx = self.idx(sx, sy);
        let goal_idx = self.idx(gx, gy);

        if !self.grid[goal_idx] {
            return vec![];
        }
        if start_idx == goal_idx {
            return vec![to];
        }

        let mut g_score = vec![f32::INFINITY; total];
        let mut came_from: Vec<Option<usize>> = vec![None; total];
        let mut open = BinaryHeap::new();

        g_score[start_idx] = 0.0;
        let h0 = heuristic(sx, sy, gx, gy);
        open.push(AStarNode { f: h0, g: 0.0, x: sx, y: sy });

        const DIRS: [(i32, i32, f32); 8] = [
            (1, 0, 1.0), (-1, 0, 1.0), (0, 1, 1.0), (0, -1, 1.0),
            (1, 1, std::f32::consts::SQRT_2),
            (-1, 1, std::f32::consts::SQRT_2),
            (1, -1, std::f32::consts::SQRT_2),
            (-1, -1, std::f32::consts::SQRT_2),
        ];

        let mut found = false;
        while let Some(node) = open.pop() {
            let cur_idx = self.idx(node.x, node.y);
            if cur_idx == goal_idx {
                found = true;
                break;
            }
            // Skip stale entries.
            if node.g > g_score[cur_idx] {
                continue;
            }
            for &(dx, dy, cost) in &DIRS {
                let nx = node.x as i32 + dx;
                let ny = node.y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let (nx, ny) = (nx as u32, ny as u32);
                let ni = self.idx(nx, ny);
                if !self.grid[ni] {
                    continue;
                }
                let ng = node.g + cost;
                if ng < g_score[ni] {
                    g_score[ni] = ng;
                    came_from[ni] = Some(cur_idx);
                    let h = heuristic(nx, ny, gx, gy);
                    open.push(AStarNode { f: ng + h, g: ng, x: nx, y: ny });
                }
            }
        }

        if !found {
            return vec![];
        }

        // Reconstruct path.
        let mut path_cells = Vec::new();
        let mut cur = goal_idx;
        while cur != start_idx {
            let (cx, cy) = (cur % w, cur / w);
            path_cells.push(self.cell_center(cx as u32, cy as u32));
            match came_from[cur] {
                Some(prev) => cur = prev,
                None => return vec![],
            }
        }
        path_cells.reverse();
        // Replace last waypoint with exact destination.
        if let Some(last) = path_cells.last_mut() {
            *last = to;
        }
        path_cells
    }
}

#[inline]
fn heuristic(ax: u32, ay: u32, bx: u32, by: u32) -> f32 {
    // Octile distance for 8-directional movement.
    let dx = (ax as i32 - bx as i32).unsigned_abs() as f32;
    let dy = (ay as i32 - by as i32).unsigned_abs() as f32;
    let (mn, mx) = if dx < dy { (dx, dy) } else { (dy, dx) };
    mx + (std::f32::consts::SQRT_2 - 1.0) * mn
}
