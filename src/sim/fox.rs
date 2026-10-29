//! The fox: it prowls a verge, and a hen who stands about on its verge is
//! a hen it has seen. It freezes with a "!" (her warning), then runs at
//! her. It cannot keep up with a hen who keeps hopping, and it cannot
//! follow her into traffic, into the river or into the air.

use super::rules::*;
use super::{Emote, Fate, Game, SimEvent};
use rand::RngExt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum FoxState {
    Prowl,
    /// Nose down, standing still.
    Sniff { t: f32 },
    Alert { t: f32 },
    /// Chasing; `lost` counts the seconds since it last saw her.
    Chase { lost: f32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fox {
    pub id: u32,
    pub row: i32,
    pub x: f32,
    pub prev_x: f32,
    /// +1 heading right, -1 left.
    pub heading: f32,
    pub state: FoxState,
}

impl Fox {
    pub fn new(id: u32, row: i32, x: f32) -> Self {
        Fox {
            id,
            row,
            x,
            prev_x: x,
            heading: 1.0,
            state: FoxState::Prowl,
        }
    }

    pub fn speed(&self) -> f32 {
        (self.x - self.prev_x).abs() / DT
    }
}

fn sees_hen(g: &Game, f: &Fox) -> bool {
    let h = &g.hen;
    !h.held && !h.airborne() && h.row() == f.row && (h.pos.x - f.x).abs() < FOX_SIGHT
}

pub fn update_all(g: &mut Game) {
    let w = g.width();
    for i in 0..g.foxes.len() {
        let mut f = g.foxes[i].clone();
        let sees = sees_hen(g, &f);
        let hx = g.hen.pos.x;
        match f.state {
            FoxState::Prowl => {
                f.x += f.heading * FOX_PROWL_SPEED * DT;
                if f.x < 0.8 || f.x > w - 0.8 {
                    f.heading = -f.heading;
                    f.x = f.x.clamp(0.8, w - 0.8);
                }
                if sees && super::Phase::Playing == g.phase {
                    f.heading = (hx - f.x).signum();
                    f.state = FoxState::Alert { t: FOX_ALERT_TIME };
                    let pos = glam::Vec2::new(f.x, crate::world::row_center(f.row));
                    g.emit(SimEvent::FoxAlert { fox: f.id, pos });
                    g.emit(SimEvent::Emote {
                        entity: f.id,
                        emote: Emote::Alert,
                    });
                } else if g.rng.random_bool(0.004) {
                    f.state = FoxState::Sniff {
                        t: g.rng.random_range(0.8..2.0),
                    };
                }
            }
            FoxState::Sniff { t } => {
                f.state = if sees {
                    FoxState::Alert { t: FOX_ALERT_TIME }
                } else if t - DT <= 0.0 {
                    if g.rng.random_bool(0.5) {
                        f.heading = -f.heading;
                    }
                    FoxState::Prowl
                } else {
                    FoxState::Sniff { t: t - DT }
                };
            }
            FoxState::Alert { t } => {
                f.state = if t - DT <= 0.0 { FoxState::Chase { lost: 0.0 } } else { FoxState::Alert { t: t - DT } };
            }
            FoxState::Chase { lost } => {
                let lost = if sees { 0.0 } else { lost + DT };
                if lost > FOX_GIVE_UP {
                    f.state = FoxState::Sniff { t: 1.0 };
                    g.emit(SimEvent::Emote {
                        entity: f.id,
                        emote: Emote::Puzzled,
                    });
                } else {
                    if sees {
                        f.heading = (hx - f.x).signum();
                    }
                    f.x = (f.x + f.heading * FOX_CHASE_SPEED * DT).clamp(0.5, w - 0.5);
                    f.state = FoxState::Chase { lost };
                }
            }
        }
        let bite = super::Phase::Playing == g.phase && sees && (hx - f.x).abs() < FOX_BITE && g.hen.alt <= AIR_ALT * 0.5;
        g.foxes[i] = f;
        if bite {
            g.die(Fate::Bitten);
        }
    }
}
