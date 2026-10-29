//! Bugs: snacks that turn up all over the field and wander about.
//!
//! Eight kinds. Six live on the ground and are pecked up by a hen on her
//! feet: ladybirds, caterpillars, beetles (quick, in dashes), snails,
//! grasshoppers (which leap away when she comes close, and need a moment
//! to recover before they can leap again) and worms (which come up out of
//! a verge and soon go back down). Two fly, and are only caught by a hen in
//! the air: butterflies over the land and dragonflies over the river.
//!
//! A bug that wanders onto a road or the railway is splatted by whatever
//! comes along; on a sidewalk the bicycles, prams and wagons always miss.
//! Bugs caught one soon after another make a combo, multiplying the points.

use super::rules::*;
use super::{Game, Phase, SimEvent};
use crate::world::level::LaneKind;
use crate::world::traffic::{self, Rail};
use crate::world::{row_center, row_of};
use glam::Vec2;
use rand::RngExt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BugKind {
    Ladybird,
    Caterpillar,
    Beetle,
    Snail,
    Grasshopper,
    Worm,
    Butterfly,
    Dragonfly,
}

impl BugKind {
    pub const ALL: [BugKind; 8] = [
        BugKind::Ladybird,
        BugKind::Caterpillar,
        BugKind::Beetle,
        BugKind::Snail,
        BugKind::Grasshopper,
        BugKind::Worm,
        BugKind::Butterfly,
        BugKind::Dragonfly,
    ];

    pub fn points(self) -> u32 {
        match self {
            BugKind::Snail => 40,
            BugKind::Ladybird => 50,
            BugKind::Caterpillar => 75,
            BugKind::Worm => 80,
            BugKind::Beetle => 100,
            BugKind::Grasshopper => 150,
            BugKind::Butterfly => 200,
            BugKind::Dragonfly => 250,
        }
    }

    /// Caught only in the air.
    pub fn flies(self) -> bool {
        matches!(self, BugKind::Butterfly | BugKind::Dragonfly)
    }

    pub fn name(self) -> &'static str {
        match self {
            BugKind::Ladybird => "ladybird",
            BugKind::Caterpillar => "caterpillar",
            BugKind::Beetle => "beetle",
            BugKind::Snail => "snail",
            BugKind::Grasshopper => "grasshopper",
            BugKind::Worm => "worm",
            BugKind::Butterfly => "butterfly",
            BugKind::Dragonfly => "dragonfly",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bug {
    pub id: u32,
    pub kind: BugKind,
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub vel: Vec2,
    /// Seconds before it wanders off (or burrows, or flies away).
    pub life: f32,
    /// Seconds to its next change of mind.
    pub timer: f32,
    /// A grasshopper in mid-leap: seconds left, and where it lands.
    pub leap: Option<(f32, Vec2, Vec2)>,
    /// A grasshopper catching its breath after a leap: it cannot leap
    /// again yet (the chance to catch it).
    pub tired: f32,
}

impl Bug {
    /// In mid-leap: neither pecked nor splatted.
    pub fn aloft(&self) -> bool {
        self.leap.is_some()
    }
}

/// The rows a ground bug can be in: not the nests, not the water.
fn walkable(g: &Game, row: i32) -> bool {
    g.level
        .lane(row)
        .is_some_and(|l| !matches!(l.kind, LaneKind::Nests | LaneKind::River))
}

pub fn update(g: &mut Game) {
    if g.combo_left > 0.0 {
        g.combo_left -= DT;
        if g.combo_left <= 0.0 {
            g.combo = 0;
        }
    }
    let w = g.width();
    let start = g.level.start_row();
    let mut i = 0;
    while i < g.bugs.len() {
        let mut b = g.bugs[i].clone();
        b.prev_pos = b.pos;
        b.life -= DT;
        b.timer -= DT;
        b.tired = (b.tired - DT).max(0.0);
        let row = row_of(b.pos.y);
        let lane_y = row_center(row);
        match b.kind {
            BugKind::Ladybird | BugKind::Caterpillar | BugKind::Snail => {
                if b.timer <= 0.0 {
                    let speed = match b.kind {
                        BugKind::Ladybird => 0.35,
                        BugKind::Caterpillar => 0.2,
                        _ => 0.12,
                    };
                    let dir = if g.rng.random_bool(0.5) { 1.0 } else { -1.0 };
                    let wobble = if b.kind == BugKind::Ladybird { g.rng.random_range(-0.15..0.15) } else { 0.0 };
                    b.vel = Vec2::new(dir * speed, wobble);
                    b.timer = g.rng.random_range(1.0..3.0);
                }
                b.pos += b.vel * DT;
            }
            BugKind::Beetle => {
                // Dashes, then a think.
                if b.timer <= 0.0 {
                    if b.vel == Vec2::ZERO {
                        let dir = if g.rng.random_bool(0.5) { 1.0 } else { -1.0 };
                        b.vel = Vec2::new(dir * 1.3, g.rng.random_range(-0.2..0.2));
                        b.timer = g.rng.random_range(0.4..0.8);
                    } else {
                        b.vel = Vec2::ZERO;
                        b.timer = g.rng.random_range(0.3..0.9);
                    }
                }
                b.pos += b.vel * DT;
            }
            BugKind::Worm => {}
            BugKind::Grasshopper => {
                if let Some((left, from, to)) = b.leap {
                    let left = left - DT;
                    if left <= 0.0 {
                        b.pos = to;
                        b.leap = None;
                        b.tired = 1.3;
                        b.timer = g.rng.random_range(1.2..2.6);
                    } else {
                        b.pos = from.lerp(to, 1.0 - left / GRASSHOPPER_LEAP);
                        b.leap = Some((left, from, to));
                    }
                } else {
                    let hen = g.hen.pos;
                    let near = !g.hen.airborne() && !g.hen.held && hen.distance(b.pos) < GRASSHOPPER_WARY;
                    if (near && b.tired <= 0.0) || b.timer <= 0.0 {
                        // Away from the hen if she is close, else anywhere.
                        let away = if near { (b.pos.x - hen.x).signum() } else if g.rng.random_bool(0.5) { 1.0 } else { -1.0 };
                        let dx = away * g.rng.random_range(1.2..2.4);
                        let mut to_row = row;
                        if g.rng.random_bool(0.3) {
                            let r = row + if g.rng.random_bool(0.5) { 1 } else { -1 };
                            if walkable(g, r) {
                                to_row = r;
                            }
                        }
                        let to = Vec2::new((b.pos.x + dx).clamp(0.4, w - 0.4), row_center(to_row) + g.rng.random_range(-0.15..0.15));
                        b.leap = Some((GRASSHOPPER_LEAP, b.pos, to));
                    }
                }
            }
            BugKind::Butterfly => {
                // A random flutter, wandering between rows.
                if b.timer <= 0.0 {
                    let a: f32 = g.rng.random_range(0.0..std::f32::consts::TAU);
                    b.vel = Vec2::new(a.cos(), a.sin() * 0.6) * 0.7;
                    b.timer = g.rng.random_range(0.6..1.6);
                }
                b.pos += b.vel * DT;
                b.pos.y = b.pos.y.clamp(1.2, start as f32 + 0.8);
            }
            BugKind::Dragonfly => {
                // Darts along the river, then hovers.
                if b.timer <= 0.0 {
                    if b.vel == Vec2::ZERO {
                        let dir = if g.rng.random_bool(0.5) { 1.0 } else { -1.0 };
                        b.vel = Vec2::new(dir * 2.6, 0.0);
                        b.timer = g.rng.random_range(0.3..0.6);
                    } else {
                        b.vel = Vec2::ZERO;
                        b.timer = g.rng.random_range(0.6..1.4);
                    }
                }
                b.pos += b.vel * DT;
            }
        }
        // Stay on the field, and (crawlers) in their row.
        if b.pos.x < 0.3 || b.pos.x > w - 0.3 {
            b.vel.x = -b.vel.x;
            b.pos.x = b.pos.x.clamp(0.3, w - 0.3);
        }
        if !b.kind.flies() && b.leap.is_none() {
            b.pos.y = b.pos.y.clamp(lane_y - 0.3, lane_y + 0.3);
        } else if b.kind == BugKind::Dragonfly {
            b.pos.y = b.pos.y.clamp(lane_y - 0.35, lane_y + 0.35);
        }
        // Caught?
        let hen = &g.hen;
        let reach = if b.kind.flies() { hen.airborne() } else { !hen.airborne() && !b.aloft() };
        if g.phase == Phase::Playing && reach && !hen.held && b.pos.distance(hen.pos) < BUG_REACH {
            g.bugs.remove(i);
            g.combo = if g.combo_left > 0.0 { g.combo + 1 } else { 1 };
            g.combo_left = COMBO_WINDOW;
            let points = b.kind.points().saturating_mul(g.combo);
            g.add_score(points);
            g.emit(SimEvent::BugCaught {
                kind: b.kind,
                points,
                combo: g.combo,
                pos: b.pos,
            });
            continue;
        }
        // Splat?
        if !b.kind.flies() && !b.aloft() && run_over(g, &b) {
            g.bugs.remove(i);
            g.emit(SimEvent::BugSplat { kind: b.kind, pos: b.pos });
            continue;
        }
        if b.life <= 0.0 {
            g.bugs.remove(i);
            g.emit(SimEvent::BugGone { bug: b.id });
            continue;
        }
        g.bugs[i] = b;
        i += 1;
    }
    spawn(g);
}

/// Whether traffic is going over a bug on the ground. Sidewalk traffic
/// never squashes one: cyclists swerve and prams have very high wheels.
fn run_over(g: &Game, b: &Bug) -> bool {
    let row = row_of(b.pos.y);
    let Some(lane) = g.level.lane(row) else { return false };
    match lane.kind {
        LaneKind::Road => traffic::ground(lane, g.level.width, g.time).any(|s| s.overlaps(b.pos.x, 0.1, VEHICLE_INSET)),
        LaneKind::Rail => lane
            .train
            .as_ref()
            .is_some_and(|t| matches!(traffic::rail(t, g.level.width, g.time), Rail::Passing { x0, x1 } if b.pos.x > x0 && b.pos.x < x1)),
        _ => false,
    }
}

fn spawn(g: &mut Game) {
    g.bug_timer -= DT;
    if g.bug_timer > 0.0 {
        return;
    }
    g.bug_timer = g.rng.random_range(BUG_EVERY_MIN..BUG_EVERY_MAX);
    let start = g.level.start_row();
    if start < 2 {
        return;
    }
    let row = g.rng.random_range(1..=start);
    let Some(kind) = g.level.lane(row).map(|l| l.kind) else { return };
    let kind = match kind {
        LaneKind::Nests => return,
        LaneKind::River => BugKind::Dragonfly,
        k => {
            let earthy = matches!(k, LaneKind::Verge | LaneKind::Start);
            match g.rng.random_range(0..100) {
                0..20 => BugKind::Ladybird,
                20..33 => BugKind::Caterpillar,
                33..48 => BugKind::Beetle,
                48..56 => BugKind::Snail,
                56..72 => BugKind::Grasshopper,
                72..84 if earthy => BugKind::Worm,
                72..84 => BugKind::Ladybird,
                _ => BugKind::Butterfly,
            }
        }
    };
    let w = g.width();
    let pos = Vec2::new(g.rng.random_range(0.6..(w - 0.6).max(0.7)), row_center(row) + g.rng.random_range(-0.2..0.2));
    if pos.distance(g.hen.pos) < 1.5 {
        return;
    }
    let life = if kind == BugKind::Worm { g.rng.random_range(4.0..7.0) } else { g.rng.random_range(10.0..16.0) };
    let id = g.alloc_id();
    g.bugs.push(Bug {
        id,
        kind,
        pos,
        prev_pos: pos,
        vel: Vec2::ZERO,
        life,
        timer: 0.0,
        leap: None,
        tired: 0.0,
    });
    g.emit(SimEvent::BugAppeared { kind, pos });
}
