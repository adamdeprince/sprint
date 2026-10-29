//! The hawk, from the yard next door: it waits up high for a hen in the
//! open. Two things draw it: a hen in the air, and a hen standing about on
//! open grass (bunting overhead counts as cover). Its shadow slides in over
//! her, and when it settles the hawk stoops.
//!
//! How to get away depends on what drew it. A hen it found in the air is
//! safe once she lands (a hen on the ground is beneath its notice); a hen
//! it found loitering must hop out from under the shadow, which settles
//! over the spot where it saw her.

use super::rules::*;
use super::{Emote, Fate, Game, Phase, SimEvent};
use glam::Vec2;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum HawkState {
    /// Out of sight, ready again in `t` seconds.
    Away { t: f32 },
    /// Its shadow over the hen; it strikes in `t`. `air`: it is after a
    /// flying hen (else a loitering one).
    Circling { t: f32, air: bool },
    Diving { t: f32, air: bool },
    /// Flying off, with or without a hen.
    Leaving { t: f32, took: bool },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hawk {
    pub id: u32,
    pub state: HawkState,
    /// Where its shadow falls.
    pub pos: Vec2,
    pub prev_pos: Vec2,
    /// Where it saw a loitering hen: the shadow settles there.
    pub mark: Vec2,
}

impl Hawk {
    pub fn new(id: u32, away: f32) -> Self {
        Hawk {
            id,
            state: HawkState::Away { t: away },
            pos: Vec2::new(-4.0, -4.0),
            prev_pos: Vec2::new(-4.0, -4.0),
            mark: Vec2::new(-4.0, -4.0),
        }
    }

    pub fn reset(&mut self, rng: &mut ChaCha8Rng) {
        self.state = HawkState::Away {
            t: rng.random_range(HAWK_AWAY_MIN..HAWK_AWAY_MAX),
        };
    }

    pub fn shadow_visible(&self) -> bool {
        matches!(self.state, HawkState::Circling { .. } | HawkState::Diving { .. })
    }
}

pub fn update(g: &mut Game) {
    let Some(mut hk) = g.hawk.clone() else { return };
    let hen = g.hen.pos;
    let playing = g.phase == Phase::Playing;
    hk.state = match hk.state {
        HawkState::Away { t } => {
            let t = t - DT;
            let flying = g.hen.airborne();
            let loitering = g.hen.loiter >= HAWK_LOITER;
            if t <= 0.0 && playing && !g.hen.held && (flying || loitering) {
                // The shadow slides in from up-wind.
                hk.pos = hen + Vec2::new(-3.0, -2.5);
                hk.prev_pos = hk.pos;
                hk.mark = hen;
                g.emit(SimEvent::HawkSpotted { pos: hen });
                g.emit(SimEvent::Emote {
                    entity: g.hen.id,
                    emote: Emote::Fright,
                });
                HawkState::Circling { t: HAWK_WARN, air: flying }
            } else {
                HawkState::Away { t: t.max(0.0) }
            }
        }
        // A child has her: the hawk will not tangle with a child.
        HawkState::Circling { .. } | HawkState::Diving { .. } if g.hen.held => HawkState::Leaving { t: 1.6, took: false },
        HawkState::Circling { t, air } => {
            // It keeps up with a flier; on the ground it settles over the
            // spot where it saw her.
            let (goal, rate) = if air { (hen, 5.0) } else { (hk.mark, 3.0) };
            hk.pos += (goal - hk.pos) * (DT * rate).min(1.0);
            if t - DT <= 0.0 { HawkState::Diving { t: HAWK_STOOP, air } } else { HawkState::Circling { t: t - DT, air } }
        }
        HawkState::Diving { t, air } => {
            let (goal, rate) = if air { (hen, 8.0) } else { (hk.mark, 8.0) };
            hk.pos += (goal - hk.pos) * (DT * rate).min(1.0);
            if t - DT <= 0.0 {
                let close = hk.pos.distance(hen) < HAWK_REACH;
                let under_cover = g.level.lane(g.hen.row()).is_some_and(|l| l.bunting);
                let exposed = if air { g.hen.airborne() } else { !g.hen.airborne() && !under_cover };
                let took = playing && !g.hen.held && close && exposed;
                g.emit(SimEvent::HawkStrike { pos: hk.pos, took });
                if took {
                    g.die(Fate::Snatched);
                }
                HawkState::Leaving { t: 1.6, took }
            } else {
                HawkState::Diving { t: t - DT, air }
            }
        }
        HawkState::Leaving { t, took } => {
            hk.pos += Vec2::new(4.0, -5.0) * DT;
            if t - DT <= 0.0 {
                HawkState::Away {
                    t: g.rng.random_range(HAWK_AWAY_MIN..HAWK_AWAY_MAX),
                }
            } else {
                HawkState::Leaving { t: t - DT, took }
            }
        }
    };
    g.hawk = Some(hk);
}
