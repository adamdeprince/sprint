//! The children: Loki, small, on an orange space hopper, and his big
//! sister Skaði, with her braids and her running boots. Both want nothing
//! more than to cuddle a Silkie.
//!
//! They are always somewhere on the field. Each bounces or skips about for
//! a while, then decides it is cuddle time and chases the hen wherever she
//! goes (cartoon physics: the traffic passes them by and they cross the
//! river as if it were not there). Both are slower than a hopping hen, and
//! a flying hen is out of their reach. A child who catches her on the
//! ground picks her up and pets her (the crossing clock stops: who could
//! hurry a cuddle?), then carries her off and puts her down somewhere
//! random, and goes back to playing for a while.
//!
//! If the other child is close by while one is holding her, they both want
//! her: a tug of war. The winner (Skaði more often: she is older) runs off
//! with the hen, and the loser stomps away to sulk.
//!
//! Neither is a monster: a child only ever puts the hen down where she will
//! be safe for a while even if she does nothing at all. Skaði is the more
//! careful, and makes sure of four seconds; Loki only thinks two ahead.
//! Because lane traffic is a pure function of time, that is checked exactly,
//! by running a copy of the game forward from the moment she would be set
//! down.

use super::rules::*;
use super::{Emote, Game, Phase, SimEvent, TickInput};
use crate::world::level::LaneKind;
use crate::world::{row_center, traffic};
use glam::Vec2;
use rand::RngExt;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kid {
    Loki,
    Skadi,
}

impl Kid {
    pub fn name(self) -> &'static str {
        match self {
            Kid::Loki => "Loki",
            Kid::Skadi => "Skaði",
        }
    }

    /// Chasing speed, tiles per second (a hopping hen does about 6).
    pub fn speed(self) -> f32 {
        match self {
            Kid::Loki => 1.9,
            Kid::Skadi => 2.3,
        }
    }

    /// How fast the hen is carried to where she is put down.
    pub fn carry_speed(self) -> f32 {
        match self {
            Kid::Loki => 6.5,
            // Careful steps: she is holding a chicken.
            Kid::Skadi => 5.0,
        }
    }

    pub fn cuddle_time(self) -> f32 {
        match self {
            Kid::Loki => 2.4,
            Kid::Skadi => 2.8,
        }
    }

    /// Where the hen is put down must stay safe at least this long if she
    /// does nothing at all.
    pub fn safe_time(self) -> f32 {
        match self {
            Kid::Loki => LOKI_SAFE_TIME,
            Kid::Skadi => SKADI_SAFE_TIME,
        }
    }

    /// Chance of winning a tug of war.
    pub fn strength(self) -> f64 {
        match self {
            Kid::Loki => 0.4,
            Kid::Skadi => 0.6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum KidState {
    /// Playing about the field, heading for `to`; cuddle time again in `t`
    /// seconds.
    Wandering { t: f32, to: Vec2 },
    /// After the hen; gets bored in `t` seconds.
    Chasing { t: f32 },
    /// Holding and petting her.
    Cuddling { t: f32, pet: f32 },
    /// Pulling at her against the other child; it is decided in `t`.
    Tugging { t: f32 },
    /// Taking her to where she will be put down.
    Carrying { from: Vec2, to: Vec2, tick: u32, ticks: u32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Child {
    pub id: u32,
    pub kid: Kid,
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub vel: Vec2,
    pub state: KidState,
    /// Cuddles given this level.
    pub cuddles: u32,
}

impl Child {
    /// A child at `pos`, playing for `rest` seconds before the first chase.
    pub fn new(id: u32, kid: Kid, pos: Vec2, rest: f32) -> Self {
        Child {
            id,
            kid,
            pos,
            prev_pos: pos,
            vel: Vec2::ZERO,
            state: KidState::Wandering { t: rest, to: pos },
            cuddles: 0,
        }
    }

    pub fn holding(&self) -> bool {
        matches!(self.state, KidState::Cuddling { .. } | KidState::Tugging { .. } | KidState::Carrying { .. })
    }

    /// After the hen is lost or nested: a chase in progress is called off
    /// for a breather. (A child already playing keeps counting down to
    /// the next cuddle.)
    pub fn reset(&mut self, rng: &mut ChaCha8Rng) {
        if matches!(self.state, KidState::Chasing { .. }) {
            self.state = KidState::Wandering {
                t: rng.random_range(KID_REST_MIN..KID_REST_MAX),
                to: self.pos,
            };
        }
    }
}

/// Somewhere to play: anywhere below the nests.
pub fn somewhere(rng: &mut ChaCha8Rng, w: f32, h: f32) -> Vec2 {
    Vec2::new(rng.random_range(0.8..(w - 0.8).max(0.9)), rng.random_range(1.3..(h - 0.4).max(1.4)))
}

/// Steers towards `goal` at `speed`, with a child's acceleration.
fn steer(c: &mut Child, goal: Vec2, speed: f32, h: f32) {
    let want = (goal - c.pos).normalize_or_zero() * speed;
    let dv = want - c.vel;
    let max = KID_ACCEL * DT;
    c.vel += if dv.length() > max { dv.normalize() * max } else { dv };
    c.pos += c.vel * DT;
    c.pos.y = c.pos.y.clamp(1.2, h - 0.3);
}

fn rest(g: &mut Game, w: f32, h: f32) -> KidState {
    KidState::Wandering {
        t: g.rng.random_range(KID_REST_MIN..KID_REST_MAX),
        to: somewhere(&mut g.rng, w, h),
    }
}

pub fn update(g: &mut Game) {
    for i in 0..g.kids.len() {
        update_one(g, i);
    }
    start_tug(g);
    finish_tug(g);
}

fn update_one(g: &mut Game, i: usize) {
    let mut c = g.kids[i].clone();
    let w = g.width();
    let h = g.level.height() as f32;
    let playing = g.phase == Phase::Playing;
    let hen = g.hen.pos;
    let speed = c.kid.speed();
    c.state = match c.state {
        KidState::Wandering { t, to } => {
            let to = if c.pos.distance(to) < 0.4 { somewhere(&mut g.rng, w, h) } else { to };
            steer(&mut c, to, speed * 0.7, h);
            if t - DT <= 0.0 && playing {
                g.emit(SimEvent::KidChasing { kid: c.kid, pos: c.pos });
                g.emit(SimEvent::Emote {
                    entity: c.id,
                    emote: Emote::Heart,
                });
                KidState::Chasing { t: KID_PATIENCE }
            } else {
                KidState::Wandering { t: (t - DT).max(0.0), to }
            }
        }
        KidState::Chasing { t } => {
            steer(&mut c, Vec2::new(hen.x, hen.y.max(1.0)), speed, h);
            let reach = !g.hen.flying && g.hen.alt <= 0.05 && !g.hen.held && c.pos.distance(hen) < KID_REACH;
            if reach && playing {
                g.hen.held = true;
                g.hen.hop = None;
                g.hen.queued = None;
                g.hen.flying = false;
                g.hen.alt = 0.0;
                c.cuddles += 1;
                c.vel = Vec2::ZERO;
                g.emit(SimEvent::KidCaught { kid: c.kid, pos: hen });
                g.emit(SimEvent::Emote {
                    entity: c.id,
                    emote: Emote::Heart,
                });
                KidState::Cuddling {
                    t: c.kid.cuddle_time(),
                    pet: 0.0,
                }
            } else if t - DT <= 0.0 {
                g.emit(SimEvent::KidBored { kid: c.kid, pos: c.pos });
                g.emit(SimEvent::Emote {
                    entity: c.id,
                    emote: Emote::Puzzled,
                });
                rest(g, w, h)
            } else {
                if g.hen.airborne() && g.rng.random_bool(0.01) {
                    g.emit(SimEvent::Emote {
                        entity: c.id,
                        emote: Emote::Note,
                    });
                }
                KidState::Chasing { t: t - DT }
            }
        }
        KidState::Cuddling { t, pet } => {
            g.hen.pos = c.pos;
            let mut pet = pet - DT;
            if pet <= 0.0 {
                pet = 0.5;
                g.emit(SimEvent::KidPet { kid: c.kid, pos: c.pos });
            }
            if t - DT <= 0.0 {
                begin_carry(g, &mut c)
            } else {
                KidState::Cuddling { t: t - DT, pet }
            }
        }
        // Decided in `finish_tug`, for both children at once.
        KidState::Tugging { t } => KidState::Tugging { t: t - DT },
        KidState::Carrying { from, to, tick, ticks } => {
            let tick = tick + 1;
            let k = (tick as f32 / ticks.max(1) as f32).min(1.0);
            c.pos = from.lerp(to, k);
            g.hen.pos = c.pos;
            if tick >= ticks {
                g.hen.held = false;
                g.hen.pos = to;
                g.hen.alt = 0.0;
                g.hen.rest = RECHARGE_DELAY;
                g.hen.best_row = g.hen.best_row.min(g.hen.row());
                g.emit(SimEvent::KidDrop { kid: c.kid, pos: to });
                g.emit(SimEvent::Emote {
                    entity: g.hen.id,
                    emote: Emote::Puzzled,
                });
                c.vel = Vec2::ZERO;
                // Everyone calms down for a bit: she has only just been put
                // down.
                for (j, other) in g.kids.clone().iter().enumerate() {
                    if j != i && matches!(other.state, KidState::Chasing { .. }) {
                        let s = rest(g, w, h);
                        g.kids[j].state = s;
                    }
                }
                rest(g, w, h)
            } else {
                KidState::Carrying { from, to, tick, ticks }
            }
        }
    };
    g.kids[i] = c;
}

/// Decides where a child takes the hen, and sets off.
fn begin_carry(g: &mut Game, c: &mut Child) -> KidState {
    // A look-ahead copy (the autopilot's) does not search: it only needs
    // to know she will be somewhere, unharmed.
    let (to, ticks) = if g.lookahead { (c.pos, 1) } else { find_drop(g, c.pos, c.kid) };
    KidState::Carrying {
        from: c.pos,
        to,
        tick: 0,
        ticks,
    }
}

/// One child is holding the hen and the other is near: tug of war.
fn start_tug(g: &mut Game) {
    let Some(a) = g.kids.iter().position(|c| matches!(c.state, KidState::Cuddling { .. })) else {
        return;
    };
    let Some(b) = g
        .kids
        .iter()
        .enumerate()
        .position(|(j, c)| j != a && !c.holding() && c.pos.distance(g.kids[a].pos) < TUG_RANGE)
    else {
        return;
    };
    // The hen in the middle, a child pulling at each side.
    let w = g.width();
    let centre = Vec2::new(g.hen.pos.x.clamp(1.0, w - 1.0), g.hen.pos.y);
    let (left, right) = if g.kids[a].pos.x <= g.kids[b].pos.x { (a, b) } else { (b, a) };
    g.kids[left].pos = centre - Vec2::new(TUG_GAP, 0.0);
    g.kids[right].pos = centre + Vec2::new(TUG_GAP, 0.0);
    for k in [a, b] {
        g.kids[k].vel = Vec2::ZERO;
        g.kids[k].state = KidState::Tugging { t: TUG_TIME };
        let id = g.kids[k].id;
        g.emit(SimEvent::Emote {
            entity: id,
            emote: Emote::Angry,
        });
    }
    g.hen.pos = centre;
    g.hen.held = true;
    g.emit(SimEvent::TugOfWar { pos: centre });
}

/// When a tug of war is over: the winner runs off with the hen.
fn finish_tug(g: &mut Game) {
    let tuggers: Vec<usize> = (0..g.kids.len())
        .filter(|&k| matches!(g.kids[k].state, KidState::Tugging { t } if t <= 0.0))
        .collect();
    if tuggers.len() < 2 {
        // Someone left alone pulling (it cannot happen with two children,
        // but should a third ever join): they simply keep her.
        if let [k] = tuggers[..] {
            if g.kids.iter().filter(|c| matches!(c.state, KidState::Tugging { .. })).count() == 1 {
                let mut c = g.kids[k].clone();
                c.state = begin_carry(g, &mut c);
                g.kids[k] = c;
            }
        }
        return;
    }
    let (a, b) = (tuggers[0], tuggers[1]);
    let (ka, kb) = (g.kids[a].kid, g.kids[b].kid);
    let p = ka.strength() / (ka.strength() + kb.strength());
    let (winner, loser) = if g.rng.random_bool(p) { (a, b) } else { (b, a) };
    let w = g.width();
    let h = g.level.height() as f32;
    let mut c = g.kids[winner].clone();
    c.pos = g.hen.pos;
    g.emit(SimEvent::TugWon { kid: c.kid, pos: c.pos });
    g.emit(SimEvent::Emote {
        entity: c.id,
        emote: Emote::Heart,
    });
    c.cuddles += 1;
    c.state = begin_carry(g, &mut c);
    g.kids[winner] = c;
    // The loser stomps off somewhere else for a sulk.
    let sulk = rest(g, w, h);
    let lid = g.kids[loser].id;
    g.kids[loser].state = sulk;
    g.emit(SimEvent::Emote {
        entity: lid,
        emote: Emote::Sweat,
    });
}

fn carry_ticks(from: Vec2, to: Vec2, kid: Kid) -> u32 {
    ((from.distance(to) / (kid.carry_speed() * DT)).ceil() as u32).max(1)
}

/// Where `kid` can put the hen down, and how many ticks it takes to get
/// there. Candidates are tried in a random order: a spot on every verge,
/// road, sidewalk and rail column, and every float in reach on the river.
/// The first one where she survives the child's safe time without moving
/// wins. If nothing is safe (it never happens on the built-in levels) she
/// goes back where she was picked up.
pub fn find_drop(g: &mut Game, from: Vec2, kid: Kid) -> (Vec2, u32) {
    let started = std::time::Instant::now();
    let level = g.level.clone();
    let w = level.width;
    let mut candidates: Vec<Vec2> = Vec::new();
    for (row, lane) in level.lanes.iter().enumerate() {
        let y = row_center(row as i32);
        match lane.kind {
            LaneKind::Nests => {}
            LaneKind::River => {
                for i in 0..lane.movers.len() {
                    // Aim for the middle of the float where it will be on
                    // arrival.
                    let mut x = from.x;
                    for _ in 0..4 {
                        let t = g.time + carry_ticks(from, Vec2::new(x, y), kid) as f32 * DT;
                        if let Some(s) = traffic::ground(lane, w, t).nth(i) {
                            x = s.center();
                        }
                    }
                    if x > 1.0 && x < w as f32 - 1.0 {
                        candidates.push(Vec2::new(x, y));
                    }
                }
            }
            _ => {
                for c in 0..w {
                    candidates.push(Vec2::new(c as f32 + 0.5, y));
                }
            }
        }
    }
    // Somewhere else, please: that is half the fun.
    candidates.retain(|c| c.distance(from) > 2.0);
    candidates.shuffle(&mut g.rng);
    let mut tried = 0u32;
    let mut found = None;
    for c in candidates {
        tried += 1;
        let ticks = carry_ticks(from, c, kid);
        if safe_drop(g, c, ticks, kid.safe_time()) {
            found = Some((c, ticks));
            break;
        }
    }
    g.stats.last_drop_search = started.elapsed();
    g.stats.drop_candidates = tried;
    found.unwrap_or_else(|| {
        let back = Vec2::new(from.x, row_center(crate::world::row_of(from.y)));
        (back, carry_ticks(from, back, kid))
    })
}

/// Runs a copy of the game: the hen is carried for `ticks`, put down at
/// `at`, and stands there for `safe` seconds. True if she is still fine.
pub fn safe_drop(g: &Game, at: Vec2, ticks: u32, safe: f32) -> bool {
    let mut c = g.clone();
    c.lookahead = true;
    c.events.clear();
    c.kids.clear();
    c.hen.held = true;
    c.time_left = c.time_left.max(safe + 1.0);
    let idle = TickInput::default();
    for _ in 0..ticks {
        c.step(&idle);
    }
    c.hen.held = false;
    c.hen.pos = at;
    c.hen.prev_pos = at;
    c.hen.alt = 0.0;
    c.hen.flying = false;
    c.hen.hop = None;
    c.hen.queued = None;
    if c.phase != Phase::Playing || c.hazard().is_some() {
        return false;
    }
    let n = (safe / DT).ceil() as u32;
    for _ in 0..n {
        c.step(&idle);
        if c.phase != Phase::Playing {
            return false;
        }
    }
    true
}
