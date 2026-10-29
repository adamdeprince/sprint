//! Where lane traffic is at a given time.
//!
//! Each lane's movers travel around a loop longer than the field is wide,
//! so they disappear off one side and come back on the other. The loop is
//! centred on the field: its hidden part is split between both sides.

use super::level::{Lane, LaneKind, MoverKind, MoverSpec, TrainSpec};

/// Euclidean remainder that is always in `[0, m)`.
pub fn wrap(v: f32, m: f32) -> f32 {
    if m <= 0.0 {
        return v;
    }
    let r = v.rem_euclid(m);
    if r >= m { 0.0 } else { r }
}

/// A mover's span along its row at some moment: `x0..x1` in field tiles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub index: usize,
    pub kind: MoverKind,
    pub n: u32,
    pub x0: f32,
    pub x1: f32,
}

impl Span {
    pub fn center(&self) -> f32 {
        (self.x0 + self.x1) * 0.5
    }

    /// Whether a body of half-width `r` centred at `x` overlaps the span,
    /// shrunk by `inset` at each end (art is a little bigger than what
    /// hurts).
    pub fn overlaps(&self, x: f32, r: f32, inset: f32) -> bool {
        x + r > self.x0 + inset && x - r < self.x1 - inset
    }
}

fn span(m: &MoverSpec, index: usize, speed: f32, loop_len: f32, width: i32, t: f32) -> Span {
    let pad = (loop_len - width as f32) * 0.5;
    let x0 = wrap(m.at + speed * t, loop_len) - pad;
    Span {
        index,
        kind: m.kind,
        n: m.n,
        x0,
        x1: x0 + m.len,
    }
}

/// Ground traffic (vehicles or floats) of a lane at time `t`.
pub fn ground(lane: &Lane, width: i32, t: f32) -> impl Iterator<Item = Span> + '_ {
    lane.movers
        .iter()
        .enumerate()
        .map(move |(i, m)| span(m, i, lane.speed, lane.loop_len, width, t))
}

/// Flying traffic of a lane at time `t`.
pub fn air(lane: &Lane, width: i32, t: f32) -> impl Iterator<Item = Span> + '_ {
    lane.air
        .iter()
        .enumerate()
        .map(move |(i, m)| span(m, i, lane.air_speed, lane.air_loop, width, t))
}

/// How far through its cycle a diving turtle group is, and what that means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dive {
    Up,
    /// Bubbles: about to go under.
    Sinking,
    Under,
    Rising,
}

pub fn dive_state(lane: &Lane, index: usize, t: f32) -> Dive {
    let p = lane.dive_period.max(1.0);
    // Groups in one lane keep their own rhythm.
    let phase = wrap(t / p + index as f32 * 0.37, 1.0);
    if phase < 0.58 {
        Dive::Up
    } else if phase < 0.72 {
        Dive::Sinking
    } else if phase < 0.9 {
        Dive::Under
    } else {
        Dive::Rising
    }
}

/// Whether a float can be stood on right now.
pub fn float_holds(lane: &Lane, s: &Span, t: f32) -> bool {
    match s.kind {
        MoverKind::Divers => dive_state(lane, s.index, t) != Dive::Under,
        k => k.is_float(),
    }
}

/// How far in from each end of a float the hen still keeps her footing.
pub const FOOTING: f32 = 0.22;

/// The float carrying a hen standing at `x` in a river lane, if any.
pub fn float_under(lane: &Lane, width: i32, x: f32, t: f32) -> Option<Span> {
    if lane.kind != LaneKind::River {
        return None;
    }
    ground(lane, width, t).find(|s| x > s.x0 - FOOTING && x < s.x1 + FOOTING && float_holds(lane, s, t))
}

/// The state of a rail lane at time `t`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rail {
    Quiet,
    /// Lights flashing, train on its way.
    Warning,
    /// The train covers `x0..x1` (it may be partly off the field).
    Passing { x0: f32, x1: f32 },
}

pub fn rail(t_spec: &TrainSpec, width: i32, t: f32) -> Rail {
    let phase = wrap(t + t_spec.offset, t_spec.every);
    if phase < t_spec.warn {
        return Rail::Warning;
    }
    let len = t_spec.length();
    let d = (phase - t_spec.warn) * t_spec.speed.abs();
    let w = width as f32;
    let (x0, x1) = if t_spec.speed >= 0.0 {
        let x1 = d - 1.0;
        (x1 - len, x1)
    } else {
        let x0 = w + 1.0 - d;
        (x0, x0 + len)
    };
    if x1 < -0.5 || x0 > w + 0.5 {
        Rail::Quiet
    } else {
        Rail::Passing { x0, x1 }
    }
}

/// Whether the crossing lights of a rail lane are flashing.
pub fn rail_lights(t_spec: &TrainSpec, width: i32, t: f32) -> bool {
    !matches!(rail(t_spec, width, t), Rail::Quiet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::level::parse;

    #[test]
    fn movers_wrap_around_the_loop() {
        let l = parse("width: 10\nlanes:\nnests\nroad speed=2 hatchback@0\nstart\n").unwrap();
        let lane = &l.lanes[1];
        let a = ground(lane, 10, 0.0).next().unwrap();
        let b = ground(lane, 10, lane.loop_len / 2.0).next().unwrap();
        assert!((a.x0 - b.x0).abs() < 1e-3, "a full loop later it is back where it was");
        // Negative speeds wrap too.
        let l = parse("width: 10\nlanes:\nnests\nroad speed=-3 sofa@4\nstart\n").unwrap();
        let s = ground(&l.lanes[1], 10, 1.0).next().unwrap();
        let s2 = ground(&l.lanes[1], 10, 1.0 + l.lanes[1].loop_len / 3.0).next().unwrap();
        assert!((s.x0 - s2.x0).abs() < 1e-3);
    }

    #[test]
    fn divers_spend_some_time_under() {
        let l = parse("lanes:\nnests\nriver speed=1 divers3@0 dive=5\nstart\n").unwrap();
        let lane = &l.lanes[1];
        let mut under = 0;
        for k in 0..500 {
            if dive_state(lane, 0, k as f32 * 0.01) == Dive::Under {
                under += 1;
            }
        }
        assert!(under > 50 && under < 150, "{under}");
    }

    #[test]
    fn trains_warn_then_pass_then_go() {
        let l = parse("width: 20\nlanes:\nnests\nrail speed=10 train=2 every=10 warn=2\nstart\n").unwrap();
        let t = l.lanes[1].train.unwrap();
        assert_eq!(rail(&t, 20, 0.5), Rail::Warning);
        let mut passed = false;
        let mut quiet_after = false;
        let mut k = 2.0;
        while k < t.every {
            match rail(&t, 20, k) {
                Rail::Passing { .. } => passed = true,
                Rail::Quiet if passed => quiet_after = true,
                _ => {}
            }
            k += 0.05;
        }
        assert!(passed && quiet_after);
    }
}
