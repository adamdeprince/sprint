//! The game simulation: a fixed-timestep, deterministic model of the field.
//!
//! Everything that decides the outcome of a game lives here and runs the
//! same whichever terminal displays it. Rendering reads the state (and the
//! previous tick's positions, for interpolation) and the events, and never
//! changes them. Given the same level, seed and recorded inputs, two runs
//! produce identical states, which is what replays, tests and the
//! autopilot's look-ahead rely on.

pub mod autopilot;
pub mod bugs;
pub mod fox;
pub mod hawk;
pub mod items;
pub mod kids;
pub mod replay;
pub mod rules;

use crate::world::level::{LaneKind, Level, MoverKind};
use crate::world::traffic::{self, Rail};
use crate::world::{Dir, row_center, row_of};
use fox::Fox;
use glam::Vec2;
use hawk::Hawk;
use items::{Item, ItemKind, StrayChick};
use kids::{Child, Kid};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rules::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Player input for one tick. Both are presses, not held keys: a held key
/// arrives as repeated presses, so the game plays the same with or without
/// key release reports.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickInput {
    pub hop: Option<Dir>,
    /// Take off, or come down.
    pub flap: bool,
}

impl TickInput {
    pub fn is_idle(&self) -> bool {
        self.hop.is_none() && !self.flap
    }
}

/// Little symbols that pop over a creature's head.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Emote {
    Alert,
    Puzzled,
    Heart,
    Fright,
    Note,
    Sweat,
    Angry,
}

/// How a crossing came to a sudden end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fate {
    Squashed(MoverKind),
    Train,
    Drowned,
    SweptAway,
    Tangled(MoverKind),
    Snatched,
    Bitten,
    TimeUp,
}

impl Fate {
    pub fn describe(self) -> &'static str {
        match self {
            Fate::Squashed(MoverKind::Hatchback) => "Flattened by a hatchback!",
            Fate::Squashed(MoverKind::Tractor) => "Ploughed under by a tractor!",
            Fate::Squashed(MoverKind::IceCream) => "Run down by the ice-cream van! (it was playing Greensleeves)",
            Fate::Squashed(MoverKind::Sofa) => "Sat on by a motorised sofa!",
            Fate::Squashed(MoverKind::Limo) => "Struck by a very, very long limousine!",
            Fate::Squashed(MoverKind::Scooter) => "Bumped by a mobility scooter!",
            Fate::Squashed(MoverKind::Bathtub) => "Splashed by a bathtub racer!",
            Fate::Squashed(MoverKind::Mower) => "Trimmed by a ride-on mower!",
            Fate::Squashed(MoverKind::Bus) => "Run down by the number 9 bus! (the sheep didn't even look up)",
            Fate::Squashed(MoverKind::Tandem) => "Knocked flying by a tandem! Twice.",
            Fate::Squashed(MoverKind::Camper) => "Flattened by a camper van on its way to the coast!",
            Fate::Squashed(MoverKind::MilkFloat) => "Caught by the milk float! (it does 12 miles an hour, flat out)",
            Fate::Squashed(MoverKind::Unicycle) => "Bowled over by a clown on a unicycle!",
            Fate::Squashed(MoverKind::Bicycle) => "Run over by a bicycle! (ding ding)",
            Fate::Squashed(MoverKind::Stroller) => "Flattened by a pram! The baby thought it was hilarious.",
            Fate::Squashed(MoverKind::Wagon) => "Run down by a red wagon full of toddler!",
            Fate::Squashed(_) => "Run over!",
            Fate::Train => "The Custard Express waits for no hen!",
            Fate::Drowned => "Splash! Silkie feathers are not waterproof.",
            Fate::SweptAway => "Swept off the edge of the world!",
            Fate::Tangled(MoverKind::Kite) => "Tangled in a kite string!",
            Fate::Tangled(MoverKind::Balloon) => "Bounced off a hot-air balloon!",
            Fate::Tangled(MoverKind::Drone) => "Mistaken for a parcel by a delivery drone!",
            Fate::Tangled(MoverKind::Geese) => "Flew into a skein of geese! They had words.",
            Fate::Tangled(MoverKind::Blimp) => "Bounced off a blimp!",
            Fate::Tangled(MoverKind::Plane) => "Hit by a paper aeroplane! Unlikely, but there it is.",
            Fate::Tangled(MoverKind::Umbrella) => "Caught up in a runaway umbrella!",
            Fate::Tangled(_) => "Mid-air collision!",
            Fate::Snatched => "Snatched by the hawk! Under its shadow: land if flying, hop away if not.",
            Fate::Bitten => "The fox had her! Keep moving on the verges.",
            Fate::TimeUp => "Out of time: she stopped for a nap.",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SimEvent {
    Emote { entity: u32, emote: Emote },
    Hop { from: Vec2, to: Vec2, air: bool },
    Bump { pos: Vec2 },
    TakeOff { pos: Vec2 },
    Landed { pos: Vec2 },
    /// The meter ran out in the air.
    Exhausted { pos: Vec2 },
    /// Wanted to fly but could not.
    NoFlight { pos: Vec2 },
    Recharged,
    RowPoints { points: u32, pos: Vec2 },
    Died { fate: Fate, pos: Vec2 },
    Nested { nest: usize, points: u32, pos: Vec2, chick: Option<u8> },
    Hatched,
    LevelCleared { bonus: u32 },
    GameOver,
    ExtraLife,
    Respawned { pos: Vec2 },
    ItemSpawned { item: u32, pos: Vec2 },
    ItemCollected { item: u32, kind: ItemKind, points: u32, pos: Vec2 },
    ItemExpired { item: u32 },
    ChickAppeared { pos: Vec2 },
    ChickPicked { pos: Vec2 },
    ChickLost { pos: Vec2 },
    FoxAlert { fox: u32, pos: Vec2 },
    HawkSpotted { pos: Vec2 },
    HawkStrike { pos: Vec2, took: bool },
    /// A child has decided it is cuddle time.
    KidChasing { kid: Kid, pos: Vec2 },
    KidCaught { kid: Kid, pos: Vec2 },
    KidPet { kid: Kid, pos: Vec2 },
    KidDrop { kid: Kid, pos: Vec2 },
    KidBored { kid: Kid, pos: Vec2 },
    TugOfWar { pos: Vec2 },
    TugWon { kid: Kid, pos: Vec2 },
    BugAppeared { kind: bugs::BugKind, pos: Vec2 },
    BugCaught { kind: bugs::BugKind, points: u32, combo: u32, pos: Vec2 },
    BugSplat { kind: bugs::BugKind, pos: Vec2 },
    BugGone { bug: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Phase {
    /// Countdown before play starts.
    Ready { t: f32 },
    Playing,
    /// Something went wrong: a pause, then the next hen or game over.
    Dying { t: f32, fate: Fate },
    /// Settling into a nest box.
    Nested { t: f32, nest: usize },
    /// Every nest has an egg.
    Cleared { t: f32 },
    GameOver,
}

/// What the app should do once a level's closing animation is over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    NextLevel,
    GameOver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Nest {
    Empty,
    Egg,
    /// The level is cleared and the egg has hatched.
    Hatched,
}

/// Score and lives carried from level to level.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub score: u32,
    pub lives: u32,
    pub next_extra_life: u32,
    pub levels_cleared: u32,
    pub eggs: u32,
    pub chicks_saved: u32,
}

impl Default for Session {
    fn default() -> Self {
        Session {
            score: 0,
            lives: START_LIVES,
            next_extra_life: EXTRA_LIFE_EVERY,
            levels_cleared: 0,
            eggs: 0,
            chicks_saved: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hop {
    pub from: Vec2,
    pub to: Vec2,
    pub t: f32,
    pub dur: f32,
    pub dir: Dir,
    pub air: bool,
}

impl Hop {
    pub fn progress(&self) -> f32 {
        (self.t / self.dur).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hen {
    pub id: u32,
    pub pos: Vec2,
    pub prev_pos: Vec2,
    /// 0 on the ground, 1 flying.
    pub alt: f32,
    pub prev_alt: f32,
    /// Wants to be in the air.
    pub flying: bool,
    /// The flight meter, 0 to 1.
    pub meter: f32,
    /// Seconds on the ground since she last flew.
    pub rest: f32,
    pub hop: Option<Hop>,
    pub queued: Option<Dir>,
    pub facing: Dir,
    /// A stray chick riding on her back (its plumage).
    pub carrying: Option<u8>,
    /// In a child's arms.
    pub held: bool,
    /// The furthest row reached on this crossing (lower is further).
    pub best_row: i32,
    /// Hops made, for animation.
    pub hops: u32,
    /// Seconds standing still out on open grass (the hawk notices).
    pub loiter: f32,
}

impl Hen {
    fn new(id: u32, level: &Level) -> Self {
        let pos = Hen::start_pos(level);
        Hen {
            id,
            pos,
            prev_pos: pos,
            alt: 0.0,
            prev_alt: 0.0,
            flying: false,
            meter: 1.0,
            rest: 0.0,
            hop: None,
            queued: None,
            facing: Dir::Up,
            carrying: None,
            held: false,
            best_row: level.start_row(),
            hops: 0,
            loiter: 0.0,
        }
    }

    pub fn start_pos(level: &Level) -> Vec2 {
        Vec2::new((level.width as f32 / 2.0).floor() + 0.5, row_center(level.start_row()))
    }

    pub fn row(&self) -> i32 {
        row_of(self.pos.y)
    }

    /// High enough to clear traffic (and be in reach of what flies).
    pub fn airborne(&self) -> bool {
        self.alt >= AIR_ALT
    }

    /// Standing (or sitting) on the ground, not mid-hop.
    pub fn planted(&self) -> bool {
        self.alt <= 0.0 && self.hop.is_none() && !self.held
    }
}

/// Measured cost of simulation work, for diagnostics. Not part of the
/// deterministic state.
#[derive(Clone, Copy, Debug, Default)]
pub struct TickStats {
    pub last_tick: Duration,
    pub max_tick: Duration,
    /// Time a child spent finding a safe place to put the hen down.
    pub last_drop_search: Duration,
    pub drop_candidates: u32,
}

#[derive(Clone)]
pub struct Game {
    pub level: Arc<Level>,
    pub level_index: usize,
    pub rng: ChaCha8Rng,
    pub seed: u64,
    pub tick: u64,
    /// The traffic clock: every lane's movers are a function of it.
    pub time: f32,
    /// Seconds of play on this level.
    pub play_time: f32,
    pub phase: Phase,
    pub hen: Hen,
    pub nests: Vec<Nest>,
    /// Seconds left for this crossing.
    pub time_left: f32,
    pub session: Session,
    pub foxes: Vec<Fox>,
    pub hawk: Option<Hawk>,
    /// Loki and Skaði, on levels that have them.
    pub kids: Vec<Child>,
    pub items: Vec<Item>,
    pub bugs: Vec<bugs::Bug>,
    /// Bugs caught in a row, and seconds left to catch the next.
    pub combo: u32,
    pub combo_left: f32,
    pub stray: Option<StrayChick>,
    pub events: Vec<SimEvent>,
    pub stats: TickStats,
    pub(crate) next_id: u32,
    pub(crate) item_timer: f32,
    pub(crate) stray_timer: f32,
    pub(crate) bug_timer: f32,
    /// Set in look-ahead copies: no searching for drop spots inside a
    /// search.
    pub(crate) lookahead: bool,
}

impl Game {
    pub fn new(level: Level, level_index: usize, session: Session, seed: u64) -> Self {
        let level = Arc::new(level);
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ (level_index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let mut next_id = 1u32;
        let mut alloc = || {
            let id = next_id;
            next_id += 1;
            id
        };
        let hen = Hen::new(alloc(), &level);
        let mut foxes = Vec::new();
        for (row, lane) in level.lanes.iter().enumerate() {
            if lane.fox {
                let x = rng.random_range(1.0..(level.width as f32 - 1.0).max(1.5));
                foxes.push(Fox::new(alloc(), row as i32, x));
            }
        }
        let hawk = level.hawk.then(|| Hawk::new(alloc(), rng.random_range(HAWK_AWAY_MIN..HAWK_AWAY_MAX)));
        // The children start out somewhere on the field, well away from the
        // hen, and do not both get ideas at once.
        let mut kids = Vec::new();
        for (kid, on, first) in [(Kid::Loki, level.loki, 0.5f32), (Kid::Skadi, level.skadi, 0.9)] {
            if !on {
                continue;
            }
            let (w, h) = (level.width as f32, level.height() as f32);
            let mut at = kids::somewhere(&mut rng, w, h);
            for _ in 0..64 {
                let clear = at.distance(hen.pos) > (w.min(h) * 0.4).max(3.0);
                if clear && kids.iter().all(|c: &Child| c.pos.distance(at) > 3.0) {
                    break;
                }
                at = kids::somewhere(&mut rng, w, h);
            }
            let rest = rng.random_range(KID_REST_MIN * first..KID_REST_MAX * first);
            kids.push(Child::new(alloc(), kid, at, rest));
        }
        let nests = vec![Nest::Empty; level.nests as usize];
        let time_left = level.time;
        let item_timer = rng.random_range(ITEM_EVERY_MIN..ITEM_EVERY_MAX) * 0.5;
        let stray_timer = rng.random_range(CHICK_EVERY_MIN..CHICK_EVERY_MAX);
        Game {
            level,
            level_index,
            rng,
            seed,
            tick: 0,
            time: 0.0,
            play_time: 0.0,
            // Longer on the first level, which has instructions to read.
            phase: Phase::Ready {
                t: if level_index == 0 { READY_TIME_FIRST } else { READY_TIME },
            },
            hen,
            nests,
            time_left,
            session,
            foxes,
            hawk,
            kids,
            items: Vec::new(),
            bugs: Vec::new(),
            combo: 0,
            combo_left: 0.0,
            stray: None,
            events: Vec::new(),
            stats: TickStats::default(),
            next_id,
            item_timer,
            stray_timer,
            bug_timer: 0.5,
            lookahead: false,
        }
    }

    pub fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        id
    }

    pub fn emit(&mut self, e: SimEvent) {
        if !self.lookahead {
            self.events.push(e);
        }
    }

    pub fn drain_events(&mut self) -> Vec<SimEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn width(&self) -> f32 {
        self.level.width as f32
    }

    pub fn nests_left(&self) -> usize {
        self.nests.iter().filter(|n| **n == Nest::Empty).count()
    }

    pub fn outcome(&self) -> Option<Outcome> {
        match self.phase {
            Phase::Cleared { t } if t <= 0.0 => Some(Outcome::NextLevel),
            Phase::GameOver => Some(Outcome::GameOver),
            _ => None,
        }
    }

    pub fn add_score(&mut self, points: u32) {
        self.session.score = self.session.score.saturating_add(points);
        while self.session.score >= self.session.next_extra_life {
            self.session.lives = self.session.lives.saturating_add(1);
            self.session.next_extra_life = self.session.next_extra_life.saturating_add(EXTRA_LIFE_EVERY);
            self.emit(SimEvent::ExtraLife);
        }
    }

    /// Advances the simulation by one fixed tick.
    pub fn step(&mut self, input: &TickInput) {
        let started = Instant::now();
        self.tick += 1;
        self.hen.prev_pos = self.hen.pos;
        self.hen.prev_alt = self.hen.alt;
        for f in &mut self.foxes {
            f.prev_x = f.x;
        }
        if let Some(h) = &mut self.hawk {
            h.prev_pos = h.pos;
        }
        for c in &mut self.kids {
            c.prev_pos = c.pos;
        }
        if self.phase != Phase::GameOver {
            self.time += DT;
        }
        match self.phase {
            Phase::Ready { t } => {
                // Any hop or flap starts at once (and counts).
                if !input.is_idle() {
                    self.phase = Phase::Playing;
                    self.update_playing(input);
                } else {
                    let t = t - DT;
                    self.phase = if t <= 0.0 { Phase::Playing } else { Phase::Ready { t } };
                }
            }
            Phase::Playing => self.update_playing(input),
            Phase::Dying { t, fate } => {
                let t = t - DT;
                // The hawk carries her off; everyone else carries on.
                hawk::update(self);
                fox::update_all(self);
                bugs::update(self);
                if t <= 0.0 {
                    self.after_death();
                } else {
                    self.phase = Phase::Dying { t, fate };
                }
            }
            Phase::Nested { t, nest } => {
                let t = t - DT;
                fox::update_all(self);
                bugs::update(self);
                if t <= 0.0 {
                    self.after_nest(nest);
                } else {
                    self.phase = Phase::Nested { t, nest };
                }
            }
            Phase::Cleared { t } => self.phase = Phase::Cleared { t: t - DT },
            Phase::GameOver => {}
        }
        let elapsed = started.elapsed();
        self.stats.last_tick = elapsed;
        self.stats.max_tick = self.stats.max_tick.max(elapsed);
    }

    fn update_playing(&mut self, input: &TickInput) {
        self.play_time += DT;
        if !self.hen.held {
            self.time_left -= DT;
            if input.flap {
                self.toggle_flight();
            }
            if let Some(d) = input.hop {
                if self.hen.hop.is_none() {
                    self.try_hop(d);
                } else {
                    self.hen.queued = Some(d);
                }
            }
        }
        self.move_hen();
        if matches!(self.phase, Phase::Playing) {
            self.check_hazards();
        }
        if matches!(self.phase, Phase::Playing) {
            fox::update_all(self);
            hawk::update(self);
            kids::update(self);
            items::update(self);
            bugs::update(self);
        }
        if matches!(self.phase, Phase::Playing) && self.time_left <= 0.0 {
            self.die(Fate::TimeUp);
        }
    }

    fn toggle_flight(&mut self) {
        let pos = self.hen.pos;
        if self.hen.flying {
            self.hen.flying = false;
            return;
        }
        let row = self.hen.row();
        let under_bunting = self.level.lane(row).is_some_and(|l| l.bunting);
        if self.hen.meter < MIN_TAKEOFF || under_bunting {
            self.emit(SimEvent::NoFlight { pos });
            self.emit(SimEvent::Emote {
                entity: self.hen.id,
                emote: Emote::Sweat,
            });
            return;
        }
        self.hen.flying = true;
        self.hen.rest = 0.0;
        self.emit(SimEvent::TakeOff { pos });
    }

    /// Starts a hop (or a glide, in the air) if the way is open.
    pub(crate) fn try_hop(&mut self, d: Dir) {
        self.hen.facing = d;
        let h = &self.hen;
        let air = h.flying || h.alt > 0.0;
        let mut to = h.pos + d.vec();
        let w = self.width();
        let row = h.row();
        let bump = |g: &mut Game| {
            let pos = g.hen.pos;
            g.emit(SimEvent::Bump { pos });
        };
        match d {
            Dir::Left | Dir::Right => {
                to.x = to.x.clamp(EDGE_MARGIN, w - EDGE_MARGIN);
                if (to.x - h.pos.x).abs() < 0.05 {
                    return bump(self);
                }
            }
            Dir::Down => {
                if row >= self.level.start_row() {
                    return bump(self);
                }
            }
            Dir::Up => {
                if row <= 0 {
                    return bump(self);
                }
            }
        }
        let target_row = row_of(to.y);
        if let Some(lane) = self.level.lane(target_row) {
            if lane.bunting && air {
                self.emit(SimEvent::Emote {
                    entity: self.hen.id,
                    emote: Emote::Puzzled,
                });
                return bump(self);
            }
            if lane.kind == LaneKind::Nests {
                let x = self.hen.pos.x;
                let near = self
                    .level
                    .nest_xs()
                    .into_iter()
                    .enumerate()
                    .min_by(|a, b| (a.1 - x).abs().total_cmp(&(b.1 - x).abs()));
                match near {
                    Some((i, nx)) if (nx - x).abs() <= NEST_SNAP && self.nests[i] == Nest::Empty => to.x = nx,
                    Some((i, nx)) if (nx - x).abs() <= NEST_SNAP => {
                        let _ = i;
                        self.emit(SimEvent::Emote {
                            entity: self.hen.id,
                            emote: Emote::Puzzled,
                        });
                        return bump(self);
                    }
                    _ => return bump(self),
                }
            }
        }
        let from = self.hen.pos;
        self.hen.hop = Some(Hop {
            from,
            to,
            t: 0.0,
            dur: if air { GLIDE_TIME } else { HOP_TIME },
            dir: d,
            air,
        });
        self.hen.hops = self.hen.hops.wrapping_add(1);
        self.emit(SimEvent::Hop { from, to, air });
    }

    fn move_hen(&mut self) {
        if self.hen.held {
            return;
        }
        let level = self.level.clone();
        let w = self.width();
        // Altitude and the meter.
        let was_up = self.hen.alt > 0.0;
        if self.hen.flying {
            self.hen.alt = (self.hen.alt + DT / TAKEOFF_TIME).min(1.0);
            self.hen.meter -= DT / level.flight.max(0.1);
            if self.hen.meter <= 0.0 {
                self.hen.meter = 0.0;
                self.hen.flying = false;
                let pos = self.hen.pos;
                self.emit(SimEvent::Exhausted { pos });
                self.emit(SimEvent::Emote {
                    entity: self.hen.id,
                    emote: Emote::Sweat,
                });
            }
        } else {
            self.hen.alt = (self.hen.alt - DT / LAND_TIME).max(0.0);
        }
        if was_up && self.hen.alt <= 0.0 {
            let pos = self.hen.pos;
            self.emit(SimEvent::Landed { pos });
        }
        if self.hen.alt <= 0.0 && !self.hen.flying {
            self.hen.rest += DT;
            if self.hen.rest > RECHARGE_DELAY && self.hen.meter < 1.0 {
                self.hen.meter = (self.hen.meter + DT / level.recharge.max(0.1)).min(1.0);
                if self.hen.meter >= 1.0 {
                    self.emit(SimEvent::Recharged);
                }
            }
        }
        // Riding a float.
        let row = self.hen.row();
        if self.hen.alt <= 0.0 && self.hen.hop.is_none_or(|h| h.dir.is_horizontal()) {
            if let Some(lane) = level.lane(row) {
                if traffic::float_under(lane, level.width, self.hen.pos.x, self.time).is_some() {
                    let dx = lane.speed * DT;
                    self.hen.pos.x += dx;
                    if let Some(h) = &mut self.hen.hop {
                        h.from.x += dx;
                        h.to.x += dx;
                    }
                }
            }
        }
        // Wind pushes her about in the air.
        if self.hen.alt > 0.0 && level.wind != 0.0 {
            let dx = level.wind * DT * self.hen.alt;
            self.hen.pos.x = (self.hen.pos.x + dx).clamp(EDGE_MARGIN, w - EDGE_MARGIN);
            if let Some(h) = &mut self.hen.hop {
                h.from.x = (h.from.x + dx).clamp(EDGE_MARGIN, w - EDGE_MARGIN);
                h.to.x = (h.to.x + dx).clamp(EDGE_MARGIN, w - EDGE_MARGIN);
            }
        }
        // The hop itself.
        if let Some(mut h) = self.hen.hop {
            h.t += DT;
            if h.t >= h.dur {
                self.hen.pos = h.to;
                self.hen.hop = None;
                self.hop_landed();
                if matches!(self.phase, Phase::Playing) {
                    if let Some(q) = self.hen.queued.take() {
                        self.try_hop(q);
                    }
                }
            } else {
                // Ease out: quick off the mark, gentle on landing.
                let k = h.progress();
                let e = 1.0 - (1.0 - k) * (1.0 - k);
                self.hen.pos = h.from.lerp(h.to, e);
                self.hen.hop = Some(h);
            }
        }
        if self.hen.alt > 0.0 {
            self.hen.pos.x = self.hen.pos.x.clamp(EDGE_MARGIN, w - EDGE_MARGIN);
        }
        // Standing about in the open: grass with no bunting overhead.
        let open = level
            .lane(self.hen.row())
            .is_some_and(|l| matches!(l.kind, LaneKind::Verge | LaneKind::Start) && !l.bunting);
        if open && self.hen.planted() {
            self.hen.loiter += DT;
        } else {
            self.hen.loiter = 0.0;
        }
    }

    fn hop_landed(&mut self) {
        let row = self.hen.row();
        if row < self.hen.best_row {
            self.hen.best_row = row;
            let pos = self.hen.pos;
            self.add_score(POINTS_ROW);
            self.emit(SimEvent::RowPoints { points: POINTS_ROW, pos });
        }
        if self.level.lane(row).is_some_and(|l| l.kind == LaneKind::Nests) {
            self.reach_nest();
        }
    }

    fn reach_nest(&mut self) {
        let x = self.hen.pos.x;
        let Some((i, _)) = self
            .level
            .nest_xs()
            .into_iter()
            .enumerate()
            .find(|(i, nx)| (nx - x).abs() < 0.05 && self.nests[*i] == Nest::Empty)
        else {
            return;
        };
        self.nests[i] = Nest::Egg;
        self.session.eggs += 1;
        let mut points = POINTS_NEST + POINTS_PER_SECOND * self.time_left.max(0.0).floor() as u32;
        let chick = self.hen.carrying.take();
        if chick.is_some() {
            points += POINTS_CHICK;
            self.session.chicks_saved += 1;
        }
        self.add_score(points);
        self.hen.flying = false;
        self.hen.alt = 0.0;
        self.hen.queued = None;
        let pos = self.hen.pos;
        self.emit(SimEvent::Nested { nest: i, points, pos, chick });
        self.emit(SimEvent::Emote {
            entity: self.hen.id,
            emote: Emote::Heart,
        });
        self.phase = Phase::Nested { t: NESTED_TIME, nest: i };
    }

    fn after_nest(&mut self, _nest: usize) {
        if self.nests_left() == 0 {
            self.nests.fill(Nest::Hatched);
            let bonus = POINTS_ALL_NESTS;
            self.add_score(bonus);
            self.session.levels_cleared += 1;
            self.emit(SimEvent::Hatched);
            self.emit(SimEvent::LevelCleared { bonus });
            self.phase = Phase::Cleared { t: CLEARED_TIME };
        } else {
            self.respawn();
        }
    }

    /// What can hurt her where she is right now.
    pub(crate) fn hazard(&self) -> Option<Fate> {
        let h = &self.hen;
        if h.held {
            return None;
        }
        let row = h.row();
        let lane = self.level.lane(row)?;
        let w = self.level.width;
        let x = h.pos.x;
        if !h.airborne() {
            match lane.kind {
                LaneKind::Road | LaneKind::Sidewalk => {
                    if let Some(s) = traffic::ground(lane, w, self.time).find(|s| s.overlaps(x, HEN_HALF, VEHICLE_INSET)) {
                        return Some(Fate::Squashed(s.kind));
                    }
                }
                LaneKind::Rail => {
                    if let Some(t) = &lane.train {
                        if let Rail::Passing { x0, x1 } = traffic::rail(t, w, self.time) {
                            if x + HEN_HALF > x0 && x - HEN_HALF < x1 {
                                return Some(Fate::Train);
                            }
                        }
                    }
                }
                LaneKind::River if h.alt <= 0.0 => {
                    let on = traffic::float_under(lane, w, x, self.time).is_some();
                    if !on && h.hop.is_none() {
                        return Some(Fate::Drowned);
                    }
                    if on && (x < SWEPT_MARGIN || x > self.width() - SWEPT_MARGIN) {
                        return Some(Fate::SweptAway);
                    }
                }
                _ => {}
            }
        } else if let Some(s) = traffic::air(lane, w, self.time).find(|s| s.overlaps(x, 0.3, 0.08)) {
            return Some(Fate::Tangled(s.kind));
        }
        None
    }

    fn check_hazards(&mut self) {
        if let Some(f) = self.hazard() {
            self.die(f);
        }
    }

    pub(crate) fn die(&mut self, fate: Fate) {
        if !matches!(self.phase, Phase::Playing) {
            return;
        }
        let pos = self.hen.pos;
        self.hen.hop = None;
        self.hen.queued = None;
        self.hen.flying = false;
        if self.hen.carrying.take().is_some() {
            self.emit(SimEvent::ChickLost { pos });
        }
        self.emit(SimEvent::Died { fate, pos });
        self.phase = Phase::Dying { t: DEATH_TIME, fate };
    }

    fn after_death(&mut self) {
        self.session.lives = self.session.lives.saturating_sub(1);
        if self.session.lives == 0 {
            self.phase = Phase::GameOver;
            self.emit(SimEvent::GameOver);
            return;
        }
        self.respawn();
    }

    fn respawn(&mut self) {
        let id = self.hen.id;
        // The wings meter is not topped up between crossings: it only
        // refills with time, so a level cannot be flown nest after nest.
        let meter = self.hen.meter;
        self.hen = Hen::new(id, &self.level);
        self.hen.meter = meter;
        self.time_left = self.level.time;
        for c in &mut self.kids {
            c.reset(&mut self.rng);
        }
        if let Some(h) = &mut self.hawk {
            h.reset(&mut self.rng);
        }
        let pos = self.hen.pos;
        self.emit(SimEvent::Respawned { pos });
        self.phase = Phase::Playing;
    }

    /// A hash of the deterministic state, for replay verification.
    pub fn state_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.tick.hash(&mut h);
        self.session.score.hash(&mut h);
        self.session.lives.hash(&mut h);
        let mut f = |v: Vec2| {
            v.x.to_bits().hash(&mut h);
            v.y.to_bits().hash(&mut h);
        };
        f(self.hen.pos);
        f(Vec2::new(self.hen.alt, self.hen.meter));
        for fx in &self.foxes {
            f(Vec2::new(fx.x, fx.row as f32));
        }
        for c in &self.kids {
            f(c.pos);
        }
        if let Some(hk) = &self.hawk {
            f(hk.pos);
        }
        for b in &self.bugs {
            f(b.pos);
        }
        for n in &self.nests {
            (*n as u8).hash(&mut h);
        }
        self.items.len().hash(&mut h);
        h.finish()
    }
}

#[cfg(test)]
mod tests;
