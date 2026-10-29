//! Presentation: what the simulation looks like.
//!
//! Turns the game state into sprites (positions interpolated between ticks,
//! animations chosen from state and speed) and keeps the short-lived
//! effects that events set off: emotes over heads, feathers, splashes,
//! sparkles and score pop-ups. Effects use their own random numbers, so the
//! simulation stays deterministic however things are drawn.
//!
//! Lane traffic is a pure function of time, so it is drawn at the exact
//! moment between ticks that the frame shows, not at the last tick.

use crate::gfx::scenery::{self, DecalKind};
use crate::render::{Layer, SheetId, Sprite};
use crate::sim::fox::FoxState;
use crate::sim::hawk::HawkState;
use crate::sim::bugs::BugKind;
use crate::sim::items::ItemKind;
use crate::sim::kids::{Kid, KidState};
use crate::sim::rules::DT;
use crate::sim::{Emote, Fate, Game, Nest, Phase, SimEvent};
use crate::world::level::{CARRIAGE_LEN, ENGINE_LEN, LaneKind, MoverKind};
use crate::world::traffic::{self, Dive, Rail};
use crate::world::{Dir, row_center};
use glam::Vec2;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::BTreeMap;

/// Things stand this far below the middle of their row (their feet).
pub const FOOT: f32 = 0.3;
/// How high a cruising hen is drawn above her shadow, in tiles.
const HEN_LIFT: f32 = 0.85;
/// Flying traffic is drawn this far above its row.
const AIR_LIFT: f32 = 0.7;

#[derive(Clone, Debug)]
struct Particle {
    id: u32,
    anim: &'static str,
    pos: Vec2,
    vel: Vec2,
    gravity: f32,
    age: f32,
    life: f32,
    frame: usize,
    /// Frames advance with age (splashes), rather than staying put.
    animate: bool,
}

#[derive(Clone, Debug)]
struct Popup {
    id: u32,
    text: String,
    pos: Vec2,
    age: f32,
}

pub struct Show {
    /// Effects are skipped on displays where animation is costly (e-ink).
    pub effects: bool,
    rng: ChaCha8Rng,
    pub time: f32,
    particles: Vec<Particle>,
    emotes: BTreeMap<u32, (Emote, f32)>,
    popups: Vec<Popup>,
    next_id: u32,
    last_side: Dir,
    /// The hen pecks while this runs down (she just ate a bug).
    peck: f32,
}

/// Sprite keys: the top four bits say what kind of thing a key is for, so
/// keys of different kinds never meet.
fn key(tag: u32, n: u32) -> u32 {
    (tag << 28) | (n & 0x0fff_ffff)
}
const K_ENTITY: u32 = 0;
const K_SHADOW: u32 = 1;
const K_TRAFFIC: u32 = 2;
const K_TRAFFIC_SHADOW: u32 = 3;
const K_FX: u32 = 4;
const K_POPUP: u32 = 5;
const K_EMOTE: u32 = 6;
const K_EXTRA: u32 = 7;

fn dir_name(d: Dir) -> &'static str {
    match d {
        Dir::Up => "up",
        Dir::Down => "down",
        Dir::Left => "left",
        Dir::Right => "right",
    }
}

fn heading(speed: f32) -> &'static str {
    if speed < 0.0 { "left" } else { "right" }
}

impl Default for Show {
    fn default() -> Self {
        Self::new()
    }
}

impl Show {
    pub fn new() -> Self {
        Show {
            effects: true,
            rng: ChaCha8Rng::seed_from_u64(0x5ca1_ab1e),
            time: 0.0,
            particles: Vec::new(),
            emotes: BTreeMap::new(),
            popups: Vec::new(),
            next_id: 0,
            last_side: Dir::Right,
            peck: 0.0,
        }
    }

    pub fn clear(&mut self) {
        self.particles.clear();
        self.emotes.clear();
        self.popups.clear();
    }

    fn id(&mut self) -> u32 {
        self.next_id = self.next_id.wrapping_add(1) & 0x0fff_ffff;
        self.next_id
    }

    fn burst(&mut self, anim: &'static str, at: Vec2, n: usize, speed: f32, gravity: f32, life: f32) {
        if !self.effects {
            return;
        }
        for _ in 0..n {
            let a: f32 = self.rng.random_range(0.0..std::f32::consts::TAU);
            let s: f32 = self.rng.random_range(0.4..1.0) * speed;
            let id = self.id();
            let frame = self.rng.random_range(0..3);
            self.particles.push(Particle {
                id,
                anim,
                pos: at,
                vel: Vec2::new(a.cos() * s, a.sin() * s - speed * 0.4),
                gravity,
                age: 0.0,
                life: life * self.rng.random_range(0.7..1.2),
                frame,
                animate: false,
            });
        }
    }

    fn one(&mut self, anim: &'static str, at: Vec2, vel: Vec2, life: f32, animate: bool) {
        if !self.effects {
            return;
        }
        let id = self.id();
        self.particles.push(Particle {
            id,
            anim,
            pos: at,
            vel,
            gravity: 0.0,
            age: 0.0,
            life,
            frame: 0,
            animate,
        });
    }

    fn popup(&mut self, text: String, at: Vec2) {
        let id = self.id();
        self.popups.push(Popup { id, text, pos: at, age: 0.0 });
    }

    pub fn on_event(&mut self, e: &SimEvent) {
        let foot = |p: Vec2| p + Vec2::new(0.0, FOOT);
        match e {
            SimEvent::Emote { entity, emote } => {
                self.emotes.insert(*entity, (*emote, 1.4));
            }
            SimEvent::Hop { from, air: false, .. } => self.burst("dust", foot(*from), 1, 0.6, 0.0, 0.3),
            SimEvent::TakeOff { pos } => {
                self.burst("feather", foot(*pos) - Vec2::new(0.0, 0.3), 3, 2.0, 1.2, 0.8);
                self.burst("dust", foot(*pos), 3, 1.5, 0.0, 0.4);
            }
            SimEvent::Landed { pos } => self.burst("dust", foot(*pos), 2, 1.2, 0.0, 0.35),
            SimEvent::Exhausted { pos } => self.burst("feather", foot(*pos) - Vec2::new(0.0, 0.8), 2, 1.0, 1.5, 0.8),
            SimEvent::Bump { pos } => self.one("puff", foot(*pos) - Vec2::new(0.0, 0.4), Vec2::new(0.0, -0.4), 0.35, true),
            SimEvent::RowPoints { .. } => {}
            SimEvent::Died { fate, pos } => {
                let at = foot(*pos);
                match fate {
                    Fate::Drowned | Fate::SweptAway => {
                        self.one("splash", at, Vec2::ZERO, 0.6, true);
                        self.burst("feather", at - Vec2::new(0.0, 0.3), 3, 1.2, 0.6, 1.0);
                    }
                    Fate::Squashed(_) | Fate::Train => {
                        self.burst("feather", at - Vec2::new(0.0, 0.3), 12, 3.2, 2.0, 1.5);
                        self.burst("star", at - Vec2::new(0.0, 0.8), 3, 1.0, 0.0, 1.2);
                    }
                    Fate::Tangled(_) => {
                        self.burst("feather", at - Vec2::new(0.0, HEN_LIFT), 10, 2.5, 2.5, 1.5);
                        self.burst("star", at - Vec2::new(0.0, HEN_LIFT + 0.3), 3, 1.0, 0.0, 1.0);
                    }
                    Fate::Snatched | Fate::Bitten => self.burst("feather", at - Vec2::new(0.0, 0.5), 12, 3.0, 1.5, 1.4),
                    Fate::TimeUp => {}
                }
            }
            SimEvent::Nested { points, pos, chick, .. } => {
                self.popup(format!("+{points}"), *pos - Vec2::new(0.0, 0.4));
                self.burst("sparkle", foot(*pos) - Vec2::new(0.0, 0.4), 6 + 4 * usize::from(chick.is_some()), 2.5, 0.0, 0.9);
                if chick.is_some() {
                    self.one("heart", *pos - Vec2::new(0.0, 0.2), Vec2::new(0.0, -0.8), 1.0, false);
                }
            }
            SimEvent::LevelCleared { bonus } => self.popup(format!("+{bonus}"), Vec2::new(10.0, 3.0)),
            SimEvent::Respawned { pos } => self.burst("sparkle", foot(*pos), 5, 2.0, 0.0, 0.6),
            SimEvent::ItemCollected { points, pos, kind, .. } => {
                self.popup(format!("+{points}"), *pos - Vec2::new(0.0, 0.3));
                let n = if *kind == ItemKind::Feather { 7 } else { 3 };
                self.burst("sparkle", foot(*pos), n, 1.6, 0.0, 0.6);
            }
            SimEvent::ChickAppeared { pos } => self.one("splash", foot(*pos), Vec2::ZERO, 0.5, true),
            SimEvent::ChickPicked { pos } => {
                self.one("heart", *pos - Vec2::new(0.0, 0.3), Vec2::new(0.0, -0.8), 0.9, false);
                self.burst("sparkle", foot(*pos), 3, 1.4, 0.0, 0.5);
            }
            SimEvent::ChickLost { pos } => self.burst("feather", foot(*pos), 3, 1.4, 1.2, 0.8),
            SimEvent::HawkStrike { pos, took } => {
                self.burst("dust", foot(*pos), 4, 2.0, 0.0, 0.5);
                if *took {
                    self.burst("feather", foot(*pos) - Vec2::new(0.0, 0.8), 6, 2.2, 1.0, 1.0);
                }
            }
            SimEvent::BugCaught { kind, points, combo, pos } => {
                let at = if kind.flies() { *pos - Vec2::new(0.0, HEN_LIFT) } else { *pos };
                self.popup(format!("+{points}"), at - Vec2::new(0.0, 0.5));
                if *combo > 1 {
                    self.popup(format!("x{combo}"), at + Vec2::new(0.0, 0.1));
                }
                self.burst("sparkle", at + Vec2::new(0.0, FOOT - 0.2), 2 + (*combo as usize).min(6), 1.6, 0.0, 0.5);
                if !kind.flies() {
                    self.peck = 0.36;
                }
            }
            SimEvent::BugSplat { pos, .. } => self.one("splat", foot(*pos), Vec2::ZERO, 1.6, false),
            SimEvent::KidCaught { pos, .. } => self.burst("sparkle", foot(*pos) - Vec2::new(0.0, 0.8), 4, 1.5, 0.0, 0.6),
            SimEvent::KidPet { pos, .. } => {
                let dx = self.rng.random_range(-0.3..0.3);
                self.one("heart", foot(*pos) - Vec2::new(dx, 1.4), Vec2::new(dx, -0.9), 0.9, false);
            }
            SimEvent::KidDrop { pos, .. } => {
                self.burst("sparkle", foot(*pos), 5, 1.8, 0.0, 0.6);
                self.burst("dust", foot(*pos), 2, 1.0, 0.0, 0.4);
            }
            SimEvent::TugOfWar { pos } => {
                self.burst("dust", foot(*pos), 6, 2.0, 0.0, 0.6);
                self.burst("feather", foot(*pos) - Vec2::new(0.0, 0.6), 4, 1.6, 1.2, 1.0);
            }
            SimEvent::TugWon { pos, .. } => {
                self.burst("feather", foot(*pos) - Vec2::new(0.0, 0.6), 5, 2.2, 1.4, 1.0);
                self.burst("star", foot(*pos) - Vec2::new(0.0, 1.0), 3, 1.4, 0.0, 0.8);
            }
            _ => {}
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        self.peck = (self.peck - dt).max(0.0);
        for p in &mut self.particles {
            p.age += dt;
            p.vel.y += p.gravity * dt;
            p.vel *= 1.0 - 1.5 * dt;
            p.pos += p.vel * dt;
            if p.animate {
                p.frame = ((p.age / p.life) * 3.0) as usize;
            }
        }
        self.particles.retain(|p| p.age < p.life);
        for (_, t) in self.emotes.values_mut() {
            *t -= dt;
        }
        self.emotes.retain(|_, (_, t)| *t > 0.0);
        for p in &mut self.popups {
            p.age += dt;
        }
        self.popups.retain(|p| p.age < 1.3);
    }

    /// A celebration shower, for the level-cleared screen.
    pub fn confetti(&mut self, area: Vec2) {
        if self.rng.random_bool(0.35) {
            let at = Vec2::new(self.rng.random_range(1.0..area.x - 1.0), self.rng.random_range(1.0..area.y - 1.0));
            self.burst("sparkle", at, 3, 2.0, 0.0, 0.8);
        }
    }

    /// The state of every decal slot: nests, then crossing lamps.
    pub fn decals(g: &Game, alpha: f32) -> Vec<u8> {
        let t = g.time - DT + alpha * DT;
        let slots = scenery::decal_slots(&g.level);
        let mut nests = g.nests.iter();
        let mut v = Vec::with_capacity(slots.len());
        let mut lamp_lanes = g.level.lanes.iter().filter_map(|l| l.train.as_ref());
        let mut current = None;
        let mut post = 0;
        for s in &slots {
            match s.kind {
                DecalKind::Nest => v.push(match nests.next() {
                    Some(Nest::Egg) => 1,
                    Some(Nest::Hatched) => 2,
                    _ => 0,
                }),
                DecalKind::Lamps => {
                    if post % 2 == 0 {
                        current = lamp_lanes.next();
                    }
                    post += 1;
                    let lit = current.is_some_and(|tr| traffic::rail_lights(tr, g.level.width, t));
                    let phase = ((t * 3.0) as i64 + post) % 2;
                    v.push(if lit { 1 + phase as u8 } else { 0 });
                }
            }
        }
        v
    }

    /// Everything that moves, with positions interpolated by `alpha`
    /// between the previous tick and the current one.
    pub fn sprites(&mut self, g: &Game, alpha: f32) -> Vec<Sprite> {
        let mut v = Vec::with_capacity(128);
        let alpha = alpha.clamp(0.0, 1.0);
        let lerp = |a: Vec2, b: Vec2| a.lerp(b, alpha);
        let t = self.time;
        let lane_t = g.time - DT + alpha * DT;
        let mut heads: BTreeMap<u32, Vec2> = BTreeMap::new();
        traffic_sprites(g, lane_t, &mut v);

        // Treats.
        for it in &g.items {
            let anim = match it.kind {
                ItemKind::Corn => "corn",
                ItemKind::Feather => "feather",
            };
            v.push(Sprite {
                key: key(K_ENTITY, it.id),
                sheet: SheetId::Items,
                anim: anim.into(),
                frame: (t * 3.0) as usize,
                pos: it.pos + Vec2::new(0.0, FOOT),
                layer: Layer::Body,
                z_bias: -2,
            });
        }

        // Bugs: crawlers on the ground, fliers up at a flying hen's height.
        for b in &g.bugs {
            let pos = lerp(b.prev_pos, b.pos);
            let ground = pos + Vec2::new(0.0, FOOT);
            let (anim, frame, layer, at) = match b.kind {
                BugKind::Grasshopper if b.aloft() => {
                    let k = b.leap.map_or(0.0, |l| 1.0 - l.0 / crate::sim::rules::GRASSHOPPER_LEAP);
                    let hop = (k * std::f32::consts::PI).sin() * 0.5;
                    ("grasshopper_leap", 0, Layer::Body, ground - Vec2::new(0.0, hop))
                }
                BugKind::Butterfly | BugKind::Dragonfly => {
                    let bob = (t * 5.0 + b.id as f32).sin() * 0.08;
                    (b.kind.name(), (t * 10.0) as usize, Layer::Air, ground - Vec2::new(0.0, HEN_LIFT * 0.85 + bob))
                }
                k => (k.name(), (t * 4.0 + b.id as f32 * 0.3) as usize, Layer::Body, ground),
            };
            v.push(Sprite {
                key: key(K_ENTITY, b.id),
                sheet: SheetId::Items,
                anim: anim.into(),
                frame,
                pos: at,
                layer,
                z_bias: -3,
            });
        }

        // The stray chick on its float, peeping for help.
        if let Some(s) = &g.stray {
            if let Some(p) = s.pos(g) {
                let lane = g.level.lane(s.row);
                let p = lane
                    .and_then(|l| traffic::ground(l, g.level.width, lane_t).nth(s.mover))
                    .map_or(p, |sp| Vec2::new(sp.center(), p.y));
                v.push(Sprite {
                    key: key(K_ENTITY, s.id),
                    sheet: SheetId::Chick,
                    anim: format!("{}:{}", s.color, if (t * 0.7).fract() < 0.5 { "scared" } else { "idle" }),
                    frame: (t * 6.0) as usize,
                    pos: p + Vec2::new(0.0, FOOT - 0.05),
                    layer: Layer::Body,
                    z_bias: 2,
                });
            }
        }

        // Hatched chicks in their nests.
        for (i, (x, n)) in g.level.nest_xs().into_iter().zip(&g.nests).enumerate() {
            if *n == Nest::Hatched {
                v.push(Sprite {
                    key: key(K_EXTRA, 1000 + i as u32),
                    sheet: SheetId::Chick,
                    anim: format!("{}:happy", i % 4),
                    frame: (t * 5.0 + i as f32) as usize,
                    pos: Vec2::new(x, row_center(0) + 0.38),
                    layer: Layer::Body,
                    z_bias: 1,
                });
            }
        }

        self.fox_sprites(g, alpha, &mut v, &mut heads);
        self.hen_sprites(g, alpha, &mut v, &mut heads);
        self.kid_sprites(g, alpha, &mut v, &mut heads);

        // The hawk: a shadow sliding over the hen, then the stoop.
        if let Some(hk) = &g.hawk {
            let pos = lerp(hk.prev_pos, hk.pos) + Vec2::new(0.0, FOOT);
            match hk.state {
                HawkState::Circling { .. } | HawkState::Diving { .. } => {
                    v.push(Sprite {
                        key: key(K_SHADOW, hk.id),
                        sheet: SheetId::Hawk,
                        anim: "shadow".into(),
                        frame: (t * 3.0) as usize,
                        pos,
                        layer: Layer::Decal,
                        z_bias: 5,
                    });
                    if let HawkState::Diving { t: left, .. } = hk.state {
                        v.push(Sprite {
                            key: key(K_ENTITY, hk.id),
                            sheet: SheetId::Hawk,
                            anim: "dive".into(),
                            frame: 0,
                            pos: pos - Vec2::new(0.0, HEN_LIFT + left * 4.0),
                            layer: Layer::Over,
                            z_bias: 0,
                        });
                    }
                }
                HawkState::Leaving { took, .. } => {
                    let at = pos - Vec2::new(0.0, HEN_LIFT + 0.3);
                    v.push(Sprite {
                        key: key(K_ENTITY, hk.id),
                        sheet: SheetId::Hawk,
                        anim: "fly".into(),
                        frame: (t * 8.0) as usize,
                        pos: at,
                        layer: Layer::Over,
                        z_bias: 0,
                    });
                    if took {
                        v.push(Sprite {
                            key: key(K_EXTRA, 2),
                            sheet: SheetId::Hen,
                            anim: "startled".into(),
                            frame: (t * 6.0) as usize,
                            pos: at + Vec2::new(0.0, 0.9),
                            layer: Layer::Over,
                            z_bias: -1,
                        });
                    }
                }
                HawkState::Away { .. } => {}
            }
        }

        // Emotes over heads.
        for (&entity, &(emote, left)) in &self.emotes {
            let Some(&head) = heads.get(&entity) else { continue };
            let anim = match emote {
                Emote::Alert => "emote_alert",
                Emote::Puzzled => "emote_puzzled",
                Emote::Heart => "emote_heart",
                Emote::Fright => "emote_fright",
                Emote::Note => "emote_note",
                Emote::Sweat => "emote_sweat",
                Emote::Angry => "emote_angry",
            };
            let pop = ((1.4 - left) * 10.0).min(1.0);
            v.push(Sprite {
                key: key(K_EMOTE, entity),
                sheet: SheetId::Fx,
                anim: anim.into(),
                frame: 0,
                pos: head - Vec2::new(0.0, 0.15 * pop),
                layer: Layer::Over,
                z_bias: 10,
            });
        }

        for p in &self.particles {
            v.push(Sprite {
                key: key(K_FX, p.id),
                sheet: SheetId::Fx,
                anim: p.anim.into(),
                frame: p.frame,
                pos: p.pos,
                layer: Layer::Over,
                z_bias: 5,
            });
        }
        for pop in &self.popups {
            let n = pop.text.chars().count() as f32;
            let rise = pop.pos - Vec2::new(0.0, pop.age * 0.9);
            let mut x = -(n - 1.0) * 0.18;
            for (i, ch) in pop.text.chars().enumerate() {
                v.push(Sprite {
                    key: key(K_POPUP, pop.id.wrapping_mul(32).wrapping_add(i as u32)),
                    sheet: SheetId::Glyphs,
                    anim: ch.to_string(),
                    frame: 0,
                    pos: rise + Vec2::new(x, 0.0),
                    layer: Layer::Over,
                    z_bias: 20,
                });
                x += 0.36 * crate::gfx::title::glyph_advance(ch) / 0.72;
            }
        }
        v
    }

    fn hen_sprites(&mut self, g: &Game, alpha: f32, v: &mut Vec<Sprite>, heads: &mut BTreeMap<u32, Vec2>) {
        let h = &g.hen;
        let t = self.time;
        if h.held {
            // Drawn with Loki.
            return;
        }
        if matches!(g.hawk.as_ref().map(|hk| hk.state), Some(HawkState::Leaving { took: true, .. })) {
            return;
        }
        let pos = h.prev_pos.lerp(h.pos, alpha);
        let alt = h.prev_alt + (h.alt - h.prev_alt) * alpha;
        let ground = pos + Vec2::new(0.0, FOOT);
        if h.facing.is_horizontal() {
            self.last_side = h.facing;
        }
        let facing = h.facing;
        // A little arc on every hop on foot.
        let arc = h.hop.filter(|hp| !hp.air).map_or(0.0, |hp| (hp.progress() * std::f32::consts::PI).sin() * 0.22);
        let lift = alt * HEN_LIFT + arc;
        let mut layer = if alt > 0.3 { Layer::Air } else { Layer::Body };
        let (anim, frame): (String, usize) = match g.phase {
            Phase::Dying { fate, t: left } => {
                layer = Layer::Body;
                match fate {
                    Fate::Squashed(_) | Fate::Train => ("squashed".into(), 0),
                    Fate::Drowned | Fate::SweptAway => ("soggy".into(), (t * 4.0) as usize),
                    Fate::Tangled(_) => {
                        // Tumbling down out of the sky.
                        let fall = (1.0 - left / crate::sim::rules::DEATH_TIME).clamp(0.0, 1.0);
                        v.push(Sprite {
                            key: key(K_ENTITY, h.id),
                            sheet: SheetId::Hen,
                            anim: "dizzy".into(),
                            frame: 0,
                            pos: ground - Vec2::new(0.0, HEN_LIFT * (1.0 - fall)),
                            layer: Layer::Air,
                            z_bias: 0,
                        });
                        heads.insert(h.id, ground - Vec2::new(0.0, 1.3));
                        return;
                    }
                    Fate::Bitten | Fate::Snatched => ("dizzy".into(), 0),
                    Fate::TimeUp => ("blink_down".into(), 0),
                }
            }
            Phase::Cleared { .. } => ("celebrate".into(), (t * 8.0) as usize),
            Phase::Nested { .. } => ("celebrate".into(), (t * 8.0) as usize),
            _ if alt > 0.02 => (format!("fly_{}", dir_name(facing)), (t * 14.0) as usize),
            _ if self.peck > 0.0 && h.hop.is_none() => {
                let d = if facing == Dir::Up { Dir::Down } else { facing };
                (format!("peck_{}", dir_name(d)), (((0.36 - self.peck) / 0.12) as usize).min(2))
            }
            _ if h.hop.is_some() => (format!("walk_{}", dir_name(facing)), 1 + (h.hops as usize % 2) * 2),
            _ if facing != Dir::Up && ((t + 0.37) % 3.3) < 0.13 => (format!("blink_{}", dir_name(facing)), 0),
            _ => (format!("idle_{}", dir_name(facing)), (t * 1.6) as usize),
        };
        if lift > 0.05 && !matches!(g.phase, Phase::Dying { .. }) {
            v.push(Sprite {
                key: key(K_SHADOW, h.id),
                sheet: SheetId::Hen,
                anim: "shadow".into(),
                frame: 0,
                pos: ground,
                layer: Layer::Decal,
                z_bias: 3,
            });
        }
        let at = ground - Vec2::new(0.0, lift);
        v.push(Sprite {
            key: key(K_ENTITY, h.id),
            sheet: SheetId::Hen,
            anim,
            frame,
            pos: at,
            layer,
            z_bias: 0,
        });
        heads.insert(h.id, at - Vec2::new(0.0, 1.25));
        // A rescued chick riding on her back.
        if let Some(c) = h.carrying {
            v.push(Sprite {
                key: key(K_EXTRA, 1),
                sheet: SheetId::Chick,
                anim: format!("{c}:{}", if alt > 0.02 { "scared" } else { "happy" }),
                frame: (t * 4.0) as usize,
                pos: at - Vec2::new(0.0, 0.62),
                layer,
                z_bias: 1,
            });
        }
    }

    fn kid_sprites(&mut self, g: &Game, alpha: f32, v: &mut Vec<Sprite>, heads: &mut BTreeMap<u32, Vec2>) {
        let t = self.time;
        let tugging = g.kids.iter().any(|c| matches!(c.state, KidState::Tugging { .. }));
        for c in &g.kids {
            let sheet = match c.kid {
                Kid::Loki => SheetId::Loki,
                Kid::Skadi => SheetId::Skadi,
            };
            let pos = c.prev_pos.lerp(c.pos, alpha) + Vec2::new(0.0, FOOT);
            // Where the held hen sits, and the arms round her, per child.
            let (hen_at, arms_at, head_up) = match c.kid {
                Kid::Loki => (-0.16, 0.2, 2.0),
                Kid::Skadi => (-0.24, 0.26, 2.1),
            };
            let kid_sprite = |anim: String, frame: usize, at: Vec2| Sprite {
                key: key(K_ENTITY, c.id),
                sheet,
                anim,
                frame,
                pos: at,
                layer: Layer::Air,
                z_bias: 0,
            };
            match c.state {
                KidState::Cuddling { .. } | KidState::Carrying { .. } => {
                    let k = (t * 2.0) as usize;
                    // While carrying her they trot along.
                    let hop = if matches!(c.state, KidState::Carrying { .. }) { (t * 9.0).sin().abs() * 0.15 } else { 0.0 };
                    let at = pos - Vec2::new(0.0, hop);
                    let hold = if c.kid == Kid::Loki { "cuddle" } else { "hold" };
                    v.push(kid_sprite(hold.into(), k, at));
                    // The hen in front and the arms round her: drawn over
                    // everything, in that order, since she sits higher up
                    // the screen than the child's feet and would otherwise
                    // sort behind.
                    v.push(Sprite {
                        key: key(K_ENTITY, g.hen.id),
                        sheet: SheetId::Hen,
                        anim: "held".into(),
                        frame: k,
                        pos: at + Vec2::new(0.0, hen_at),
                        layer: Layer::Over,
                        z_bias: 1,
                    });
                    v.push(Sprite {
                        key: key(K_EXTRA, 3),
                        sheet,
                        anim: "arms".into(),
                        frame: k,
                        pos: at + Vec2::new(0.0, arms_at),
                        layer: Layer::Over,
                        z_bias: 3,
                    });
                    if let Some(ch) = g.hen.carrying {
                        v.push(Sprite {
                            key: key(K_EXTRA, 1),
                            sheet: SheetId::Chick,
                            anim: format!("{ch}:happy"),
                            frame: k,
                            pos: at + Vec2::new(0.0, hen_at - 0.45),
                            layer: Layer::Over,
                            z_bias: 2,
                        });
                    }
                    heads.insert(c.id, at - Vec2::new(0.0, head_up));
                    heads.insert(g.hen.id, at + Vec2::new(0.0, hen_at - 1.15));
                }
                KidState::Tugging { .. } => {
                    // Each faces the hen in the middle and leans back.
                    let dir = if c.pos.x < g.hen.pos.x { "right" } else { "left" };
                    let k = (t * 6.0 + c.id as f32) as usize;
                    v.push(kid_sprite(format!("tug_{dir}"), k, pos));
                    heads.insert(c.id, pos - Vec2::new(0.0, head_up));
                }
                _ => {
                    let dir = if c.vel.x < -0.05 { "left" } else { "right" };
                    v.push(Sprite {
                        key: key(K_SHADOW, c.id),
                        sheet: SheetId::Hen,
                        anim: "shadow".into(),
                        frame: 0,
                        pos,
                        layer: Layer::Decal,
                        z_bias: 2,
                    });
                    match c.kid {
                        Kid::Loki => {
                            // Boing.
                            let phase = (t * 2.3 + c.id as f32 * 0.3).fract();
                            let height = (phase * std::f32::consts::PI).sin() * 0.42;
                            let frame = if phase < 0.08 || phase > 0.92 {
                                0
                            } else if height > 0.3 {
                                2
                            } else {
                                1
                            };
                            v.push(kid_sprite(format!("bounce_{dir}"), frame, pos - Vec2::new(0.0, height * 0.6)));
                            heads.insert(c.id, pos - Vec2::new(0.0, head_up + height * 0.6));
                        }
                        Kid::Skadi => {
                            // Strides with the ground she covers.
                            let stride = (c.pos.x * 3.2 + c.pos.y * 3.2) as usize;
                            let frame = if c.vel.length() > 0.1 { stride } else { 0 };
                            v.push(kid_sprite(format!("run_{dir}"), frame, pos));
                            heads.insert(c.id, pos - Vec2::new(0.0, head_up));
                        }
                    }
                }
            }
        }
        // The hen in a tug of war: in the middle at hand height, yanked one
        // way and the other.
        if tugging {
            let yank = (t * 9.0).sin() * 0.14;
            let at = g.hen.prev_pos.lerp(g.hen.pos, alpha) + Vec2::new(yank, FOOT - 0.42);
            v.push(Sprite {
                key: key(K_ENTITY, g.hen.id),
                sheet: SheetId::Hen,
                anim: "held".into(),
                frame: (t * 6.0) as usize,
                pos: at,
                layer: Layer::Over,
                z_bias: 1,
            });
            heads.insert(g.hen.id, at - Vec2::new(0.0, 0.85));
        }
    }

    fn fox_sprites(&mut self, g: &Game, alpha: f32, v: &mut Vec<Sprite>, heads: &mut BTreeMap<u32, Vec2>) {
        let t = self.time;
        for f in &g.foxes {
            let x = f.prev_x + (f.x - f.prev_x) * alpha;
            let pos = Vec2::new(x, row_center(f.row) + FOOT);
            let side = if f.heading < 0.0 { "left" } else { "right" };
            let stride = (x * 4.0).abs() as usize;
            let (anim, frame) = match f.state {
                FoxState::Alert { .. } => (format!("alert_{side}"), 0),
                FoxState::Sniff { .. } => (format!("sniff_{side}"), (t * 3.0) as usize),
                FoxState::Chase { .. } if f.speed() > 0.1 => (format!("run_{side}"), stride),
                FoxState::Chase { .. } => (format!("alert_{side}"), 0),
                FoxState::Prowl => (format!("walk_{side}"), stride),
            };
            v.push(Sprite {
                key: key(K_ENTITY, f.id),
                sheet: SheetId::Fox,
                anim,
                frame,
                pos,
                layer: Layer::Body,
                z_bias: 0,
            });
            heads.insert(f.id, pos - Vec2::new(0.0, 1.3));
        }
    }
}

/// Lane traffic at time `t`: vehicles, floats, flying things and trains.
fn traffic_sprites(g: &Game, t: f32, v: &mut Vec<Sprite>) {
    let w = g.level.width;
    // Keys count through the level's traffic in a fixed order, so every
    // segment keeps its key from frame to frame.
    let mut n = 0u32;
    let mut next = || {
        n += 1;
        n
    };
    for (row, lane) in g.level.lanes.iter().enumerate() {
        let y = row_center(row as i32) + FOOT;
        let dir = heading(lane.speed);
        let wheels = (t * lane.speed.abs() * 2.5) as usize;
        for s in traffic::ground(lane, w, t) {
            let mut put = |anim: String, x: f32, frame: usize, sheet: SheetId, bias: i32| {
                v.push(Sprite {
                    key: key(K_TRAFFIC, next()),
                    sheet,
                    anim,
                    frame,
                    pos: Vec2::new(x, y),
                    layer: Layer::Body,
                    z_bias: bias,
                });
            };
            match s.kind {
                MoverKind::Limo => {
                    for i in 0..s.n {
                        let front = if lane.speed >= 0.0 { i + 1 == s.n } else { i == 0 };
                        let back = if lane.speed >= 0.0 { i == 0 } else { i + 1 == s.n };
                        let part = if front {
                            "limo_nose"
                        } else if back {
                            "limo_tail"
                        } else {
                            "limo_mid"
                        };
                        put(format!("{part}_{dir}"), s.x0 + i as f32 + 0.5, wheels, SheetId::Cars, 0);
                    }
                }
                MoverKind::Log => {
                    for i in 0..s.n {
                        let part = if s.n == 1 {
                            "log_m"
                        } else if i == 0 {
                            "log_l"
                        } else if i + 1 == s.n {
                            "log_r"
                        } else {
                            "log_m"
                        };
                        put(part.into(), s.x0 + i as f32 + 0.5, 0, SheetId::Floats, -60);
                    }
                }
                MoverKind::Turtles | MoverKind::Divers => {
                    let state = if s.kind == MoverKind::Divers { traffic::dive_state(lane, s.index, t) } else { Dive::Up };
                    for i in 0..s.n {
                        let anim = match state {
                            Dive::Up => format!("turtle_{dir}"),
                            Dive::Sinking | Dive::Rising => format!("turtle_sink_{dir}"),
                            Dive::Under => "turtle_under".into(),
                        };
                        put(anim, s.x0 + i as f32 + 0.5, (t * 3.0) as usize + i as usize, SheetId::Floats, -60);
                    }
                }
                MoverKind::Lily => {
                    for i in 0..s.n {
                        let anim = if (s.index + i as usize) % 3 == 1 { "lily_flower" } else { "lily" };
                        put(anim.into(), s.x0 + i as f32 + 0.5, (t * 1.5) as usize, SheetId::Floats, -60);
                    }
                }
                MoverKind::Duck | MoverKind::Flamingo | MoverKind::Pedalo | MoverKind::Raft | MoverKind::Canoe => {
                    put(format!("{}_{dir}", s.kind.name()), s.center(), (t * 2.0) as usize, SheetId::Floats, -60);
                }
                MoverKind::Picnic => put("picnic".into(), s.center(), (t * 1.5) as usize, SheetId::Floats, -60),
                MoverKind::Ring => put("ring".into(), s.center(), (t * 1.5) as usize, SheetId::Floats, -60),
                k if k.is_vehicle() => put(format!("{}_{dir}", k.name()), s.center(), wheels, SheetId::Cars, 0),
                _ => {}
            }
        }
        let air_dir = heading(lane.air_speed);
        for s in traffic::air(lane, w, t) {
            let name = s.kind.name();
            let anim = match s.kind {
                MoverKind::Kite | MoverKind::Balloon | MoverKind::Umbrella => name.to_string(),
                _ => format!("{name}_{air_dir}"),
            };
            let shadow = format!("shadow_{name}");
            let k = next();
            v.push(Sprite {
                key: key(K_TRAFFIC_SHADOW, k),
                sheet: SheetId::Sky,
                anim: shadow,
                frame: 0,
                pos: Vec2::new(s.center(), y),
                layer: Layer::Decal,
                z_bias: 0,
            });
            v.push(Sprite {
                key: key(K_TRAFFIC, k),
                sheet: SheetId::Sky,
                anim,
                frame: (t * match s.kind {
                    MoverKind::Drone => 14.0,
                    MoverKind::Geese | MoverKind::Plane => 6.0,
                    _ => 3.0,
                }) as usize,
                pos: Vec2::new(s.center(), y - FOOT - AIR_LIFT),
                layer: Layer::Air,
                z_bias: 0,
            });
        }
        if lane.kind == LaneKind::Rail {
            if let Some(tr) = &lane.train {
                // Keys for the whole train are reserved even when it is not
                // passing, so the keys of the lanes below do not shift.
                let keys: Vec<u32> = (0..=tr.carriages).map(|_| next()).collect();
                if let Rail::Passing { x0, x1 } = traffic::rail(tr, w, t) {
                    let dir = heading(tr.speed);
                    let wheels = (t * tr.speed.abs() * 2.0) as usize;
                    let engine_x = if tr.speed >= 0.0 { x1 - ENGINE_LEN / 2.0 } else { x0 + ENGINE_LEN / 2.0 };
                    let mut parts = vec![("engine", engine_x)];
                    for c in 0..tr.carriages {
                        let off = ENGINE_LEN + (c as f32 + 0.5) * CARRIAGE_LEN;
                        let x = if tr.speed >= 0.0 { x1 - off } else { x0 + off };
                        parts.push(("carriage", x));
                    }
                    for ((name, x), k) in parts.into_iter().zip(keys) {
                        v.push(Sprite {
                            key: key(K_TRAFFIC, k),
                            sheet: SheetId::Cars,
                            anim: format!("{name}_{dir}"),
                            frame: wheels,
                            pos: Vec2::new(x, y),
                            layer: Layer::Body,
                            z_bias: 0,
                        });
                    }
                }
            }
        }
    }
}
