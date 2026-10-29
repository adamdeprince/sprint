//! The field: levels (a stack of lanes) and the traffic in them.
//!
//! Lane traffic is a pure function of time: a mover's position is its
//! start plus speed times time, wrapped around its lane's loop. Nothing
//! about it is stored or stepped, so the simulation, the renderer and the
//! autopilot's look-ahead all agree exactly on where everything is at any
//! moment, past or future.

pub mod level;
pub mod traffic;

use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::Up, Dir::Left, Dir::Down, Dir::Right];

    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
        }
    }

    pub fn vec(self) -> Vec2 {
        let (x, y) = self.delta();
        Vec2::new(x as f32, y as f32)
    }

    pub fn is_horizontal(self) -> bool {
        matches!(self, Dir::Left | Dir::Right)
    }

    /// The direction closest to a vector, if it is not zero.
    pub fn from_vec(v: Vec2) -> Option<Dir> {
        if v.length_squared() < 1e-8 {
            return None;
        }
        Some(if v.x.abs() > v.y.abs() {
            if v.x > 0.0 { Dir::Right } else { Dir::Left }
        } else if v.y > 0.0 {
            Dir::Down
        } else {
            Dir::Up
        })
    }
}

/// The row a point is in (rows are one tile tall, row 0 at the top).
pub fn row_of(y: f32) -> i32 {
    y.floor() as i32
}

pub fn row_center(row: i32) -> f32 {
    row as f32 + 0.5
}
