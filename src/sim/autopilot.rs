//! A player for the title screen's demo, `--autoplay` and the headless
//! tests that every level can be crossed.
//!
//! Lane traffic is a pure function of time, so the autopilot can look
//! ahead exactly: it runs copies of the game through short sequences of
//! actions (wait, hop, take off or land) and keeps a beam of the most
//! promising, judged by how far up the field the hen got, how close she is
//! to an empty nest, how much flight she has left and how long it took.
//! Copies where she comes to grief are dropped. It then takes the first
//! action of the best sequence and thinks again at the next chance.

use super::rules::DT;
use super::{Game, Nest, Phase, TickInput};
use crate::world::Dir;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Act {
    Wait,
    Hop(Dir),
    Flap,
}

const ACTS: [Act; 6] = [Act::Wait, Act::Hop(Dir::Up), Act::Hop(Dir::Left), Act::Hop(Dir::Right), Act::Hop(Dir::Down), Act::Flap];

/// Ticks a wait lasts.
const WAIT_TICKS: u32 = 6;

#[derive(Clone, Debug)]
pub struct Autopilot {
    /// Copies kept per step of the look-ahead.
    pub beam: usize,
    /// Actions looked ahead.
    pub depth: usize,
    hold: u32,
    /// Look-ahead copies run, for measurements.
    pub sims: u64,
    /// Whether it may take off (off, it proves a level can be crossed on
    /// foot).
    pub may_fly: bool,
}

impl Default for Autopilot {
    fn default() -> Self {
        Self::new()
    }
}

struct Node {
    game: Game,
    first: Act,
    ticks: u32,
    done: bool,
}

/// Runs one action on a copy; false if she did not survive it.
fn apply(g: &mut Game, a: Act) -> (u32, bool) {
    let idle = TickInput::default();
    let mut ticks = 0;
    let mut first = match a {
        Act::Wait => idle.clone(),
        Act::Hop(d) => TickInput { hop: Some(d), flap: false },
        Act::Flap => TickInput { hop: None, flap: true },
    };
    let n = match a {
        Act::Wait => WAIT_TICKS,
        Act::Hop(_) => 40,
        Act::Flap => 4,
    };
    let rows_before = g.hen.row();
    for _ in 0..n {
        g.step(&first);
        first = idle.clone();
        ticks += 1;
        if g.phase != Phase::Playing {
            break;
        }
        if matches!(a, Act::Hop(_)) && g.hen.hop.is_none() {
            break;
        }
    }
    let _ = rows_before;
    let ok = matches!(g.phase, Phase::Playing | Phase::Nested { .. });
    (ticks, ok)
}

/// How good a position is (higher is better).
fn value(g: &Game, ticks: u32) -> f32 {
    if matches!(g.phase, Phase::Nested { .. }) {
        return 10_000.0 - ticks as f32;
    }
    let h = &g.hen;
    let start = g.level.start_row() as f32 + 0.5;
    let progress = start - h.pos.y;
    let nest_dx = g
        .level
        .nest_xs()
        .into_iter()
        .zip(&g.nests)
        .filter(|(_, n)| **n == Nest::Empty)
        .map(|(x, _)| (x - h.pos.x).abs())
        .fold(f32::MAX, f32::min);
    let nest_dx = if nest_dx == f32::MAX { 0.0 } else { nest_dx };
    // The closer to the top, the more lining up with a nest matters.
    let near_top = (1.0 - h.pos.y / start).clamp(0.0, 1.0);
    // Points count a little, so it will detour for a juicy bug.
    progress * 10.0 - nest_dx * (0.5 + 4.0 * near_top * near_top) + h.meter * 3.0 - ticks as f32 * DT * 2.0 + g.session.score as f32 * 0.03
}

/// Whether she lasts a moment longer standing still (so a plan does not end
/// in front of a car).
fn lasts(g: &Game) -> bool {
    let mut c = g.clone();
    let idle = TickInput::default();
    for _ in 0..12 {
        c.step(&idle);
        if !matches!(c.phase, Phase::Playing | Phase::Nested { .. }) {
            return false;
        }
    }
    true
}

impl Autopilot {
    pub fn new() -> Self {
        Autopilot {
            beam: 14,
            depth: 12,
            hold: 0,
            sims: 0,
            may_fly: true,
        }
    }

    pub fn decide(&mut self, g: &Game) -> TickInput {
        let idle = TickInput::default();
        if g.phase != Phase::Playing || g.hen.held || g.hen.hop.is_some() {
            self.hold = 0;
            return idle;
        }
        if self.hold > 0 {
            self.hold -= 1;
            return idle;
        }
        let act = self.plan(g);
        match act {
            Act::Wait => {
                self.hold = WAIT_TICKS - 1;
                idle
            }
            Act::Hop(d) => TickInput { hop: Some(d), flap: false },
            Act::Flap => {
                self.hold = 3;
                TickInput { hop: None, flap: true }
            }
        }
    }

    fn plan(&mut self, g: &Game) -> Act {
        let mut root = g.clone();
        root.lookahead = true;
        root.events.clear();
        let mut beam: Vec<Node> = vec![Node {
            game: root,
            first: Act::Wait,
            ticks: 0,
            done: false,
        }];
        let mut best: Option<(f32, Act)> = None;
        for depth in 0..self.depth {
            let mut next: Vec<Node> = Vec::new();
            for n in &beam {
                if n.done {
                    continue;
                }
                for &a in &ACTS {
                    if a == Act::Flap && (!self.may_fly || (!n.game.hen.flying && n.game.hen.meter < 0.5)) {
                        continue;
                    }
                    let mut c = n.game.clone();
                    self.sims += 1;
                    let (t, ok) = apply(&mut c, a);
                    if !ok {
                        continue;
                    }
                    let first = if depth == 0 { a } else { n.first };
                    let done = matches!(c.phase, Phase::Nested { .. });
                    next.push(Node {
                        game: c,
                        first,
                        ticks: n.ticks + t,
                        done,
                    });
                }
            }
            if next.is_empty() {
                break;
            }
            // One copy per place: the earliest to get there.
            let mut seen: BTreeMap<(i32, i32, bool, bool), usize> = BTreeMap::new();
            let mut kept: Vec<Node> = Vec::new();
            next.sort_by_key(|n| n.ticks);
            for n in next {
                let h = &n.game.hen;
                let key = (h.row(), (h.pos.x * 3.0).round() as i32, h.flying, h.alt > 0.0);
                if seen.contains_key(&key) {
                    continue;
                }
                seen.insert(key, kept.len());
                kept.push(n);
            }
            let mut scored: Vec<(f32, Node)> = kept.into_iter().map(|n| (value(&n.game, n.ticks), n)).collect();
            scored.sort_by(|a, b| b.0.total_cmp(&a.0));
            scored.truncate(self.beam);
            for (v, n) in &scored {
                let v = if n.done || lasts(&n.game) { *v } else { *v - 500.0 };
                if best.is_none_or(|(b, _)| v > b) {
                    best = Some((v, n.first));
                }
            }
            beam = scored.into_iter().map(|(_, n)| n).collect();
        }
        best.map_or(Act::Wait, |(_, a)| a)
    }
}
