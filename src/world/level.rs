//! Levels: a stack of lanes read from a small text format.
//!
//! ```text
//! name: Country Lane
//! tagline: Why did the Silkie cross the road?
//! width: 20
//! time: 45
//! nests: 5
//! flight: 1.23
//! recharge: 16
//! lanes:
//! nests
//! river  speed=-1.1  log3@0 log3@8 log4@15
//! verge  items
//! road   speed=2.0   hatchback@0 tractor@9   air=0.8 ^kite@3
//! start
//! ```
//!
//! Each line after `lanes:` is one row of the field, top to bottom. See
//! `assets/levels/README.md` for every kind and option.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaneKind {
    /// The row of nest boxes along the top.
    Nests,
    /// Safe grass.
    Verge,
    /// The verge the hen starts each crossing on (the bottom row).
    Start,
    Road,
    /// A pavement: bicycles, prams and wagons, and they will not stop.
    Sidewalk,
    River,
    Rail,
}

/// Everything that moves along a lane. Lengths are in tiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum MoverKind {
    // Road: run the hen over.
    Hatchback,
    Tractor,
    IceCream,
    Sofa,
    /// A stretch limousine of any length.
    Limo,
    Scooter,
    Bathtub,
    Mower,
    /// A double-decker bus, its passengers all sheep.
    Bus,
    Tandem,
    Camper,
    MilkFloat,
    Unicycle,
    // Sidewalks (and roads).
    Bicycle,
    Stroller,
    Wagon,
    // River: ride on these.
    Log,
    Turtles,
    /// Turtles that dive now and then.
    Divers,
    Duck,
    Flamingo,
    Picnic,
    Lily,
    /// A swan pedalo.
    Pedalo,
    /// A raft, sailed by a scarecrow.
    Raft,
    /// A canoe full of ducklings.
    Canoe,
    /// A striped rubber ring.
    Ring,
    // Air: only a flying hen meets these.
    Kite,
    Balloon,
    Drone,
    /// A skein of geese in a V.
    Geese,
    Blimp,
    /// A paper aeroplane.
    Plane,
    /// An umbrella that got away.
    Umbrella,
}

impl MoverKind {
    pub fn parse(name: &str) -> Option<(MoverKind, Option<u32>)> {
        let split = name.find(|c: char| c.is_ascii_digit()).unwrap_or(name.len());
        let (base, digits) = name.split_at(split);
        let n = if digits.is_empty() { None } else { Some(digits.parse().ok()?) };
        let k = match base {
            "hatchback" => MoverKind::Hatchback,
            "tractor" => MoverKind::Tractor,
            "icecream" => MoverKind::IceCream,
            "sofa" => MoverKind::Sofa,
            "limo" => MoverKind::Limo,
            "scooter" => MoverKind::Scooter,
            "bathtub" => MoverKind::Bathtub,
            "mower" => MoverKind::Mower,
            "bus" => MoverKind::Bus,
            "tandem" => MoverKind::Tandem,
            "camper" => MoverKind::Camper,
            "milkfloat" => MoverKind::MilkFloat,
            "unicycle" => MoverKind::Unicycle,
            "bicycle" => MoverKind::Bicycle,
            "stroller" => MoverKind::Stroller,
            "wagon" => MoverKind::Wagon,
            "pedalo" => MoverKind::Pedalo,
            "raft" => MoverKind::Raft,
            "canoe" => MoverKind::Canoe,
            "ring" => MoverKind::Ring,
            "geese" => MoverKind::Geese,
            "blimp" => MoverKind::Blimp,
            "plane" => MoverKind::Plane,
            "umbrella" => MoverKind::Umbrella,
            "log" => MoverKind::Log,
            "turtles" => MoverKind::Turtles,
            "divers" => MoverKind::Divers,
            "duck" => MoverKind::Duck,
            "flamingo" => MoverKind::Flamingo,
            "picnic" => MoverKind::Picnic,
            "lily" => MoverKind::Lily,
            "kite" => MoverKind::Kite,
            "balloon" => MoverKind::Balloon,
            "drone" => MoverKind::Drone,
            _ => return None,
        };
        Some((k, n))
    }

    pub fn name(self) -> &'static str {
        match self {
            MoverKind::Hatchback => "hatchback",
            MoverKind::Tractor => "tractor",
            MoverKind::IceCream => "icecream",
            MoverKind::Sofa => "sofa",
            MoverKind::Limo => "limo",
            MoverKind::Scooter => "scooter",
            MoverKind::Bathtub => "bathtub",
            MoverKind::Mower => "mower",
            MoverKind::Bus => "bus",
            MoverKind::Tandem => "tandem",
            MoverKind::Camper => "camper",
            MoverKind::MilkFloat => "milkfloat",
            MoverKind::Unicycle => "unicycle",
            MoverKind::Bicycle => "bicycle",
            MoverKind::Stroller => "stroller",
            MoverKind::Wagon => "wagon",
            MoverKind::Pedalo => "pedalo",
            MoverKind::Raft => "raft",
            MoverKind::Canoe => "canoe",
            MoverKind::Ring => "ring",
            MoverKind::Geese => "geese",
            MoverKind::Blimp => "blimp",
            MoverKind::Plane => "plane",
            MoverKind::Umbrella => "umbrella",
            MoverKind::Log => "log",
            MoverKind::Turtles => "turtles",
            MoverKind::Divers => "divers",
            MoverKind::Duck => "duck",
            MoverKind::Flamingo => "flamingo",
            MoverKind::Picnic => "picnic",
            MoverKind::Lily => "lily",
            MoverKind::Kite => "kite",
            MoverKind::Balloon => "balloon",
            MoverKind::Drone => "drone",
        }
    }

    /// Whether the kind takes a count (`log3`, `limo5`), and the default.
    fn sized(self) -> Option<u32> {
        match self {
            MoverKind::Limo => Some(4),
            MoverKind::Log => Some(3),
            MoverKind::Turtles | MoverKind::Divers => Some(3),
            MoverKind::Lily => Some(1),
            _ => None,
        }
    }

    /// Length in tiles for a count of `n` (where the kind takes one).
    pub fn length(self, n: u32) -> f32 {
        match self {
            MoverKind::Hatchback => 1.6,
            MoverKind::Tractor => 2.0,
            MoverKind::IceCream => 2.4,
            MoverKind::Sofa => 1.9,
            MoverKind::Limo => n as f32,
            MoverKind::Scooter => 1.1,
            MoverKind::Bathtub => 1.7,
            MoverKind::Mower => 1.2,
            MoverKind::Bus => 2.8,
            MoverKind::Tandem => 1.6,
            MoverKind::Camper => 2.2,
            MoverKind::MilkFloat => 1.8,
            MoverKind::Unicycle => 0.7,
            MoverKind::Bicycle => 1.1,
            MoverKind::Stroller => 1.1,
            MoverKind::Wagon => 1.3,
            MoverKind::Pedalo => 2.0,
            MoverKind::Raft => 2.0,
            MoverKind::Canoe => 2.4,
            MoverKind::Ring => 1.0,
            MoverKind::Geese => 2.0,
            MoverKind::Blimp => 2.8,
            MoverKind::Plane => 0.7,
            MoverKind::Umbrella => 0.9,
            MoverKind::Log => n as f32,
            MoverKind::Turtles | MoverKind::Divers => n as f32,
            MoverKind::Duck => 2.0,
            MoverKind::Flamingo => 1.6,
            MoverKind::Picnic => 2.0,
            MoverKind::Lily => n as f32,
            MoverKind::Kite => 0.9,
            MoverKind::Balloon => 1.5,
            MoverKind::Drone => 0.9,
        }
    }

    pub fn is_air(self) -> bool {
        matches!(
            self,
            MoverKind::Kite | MoverKind::Balloon | MoverKind::Drone | MoverKind::Geese | MoverKind::Blimp | MoverKind::Plane | MoverKind::Umbrella
        )
    }

    pub fn is_float(self) -> bool {
        matches!(
            self,
            MoverKind::Log
                | MoverKind::Turtles
                | MoverKind::Divers
                | MoverKind::Duck
                | MoverKind::Flamingo
                | MoverKind::Picnic
                | MoverKind::Lily
                | MoverKind::Pedalo
                | MoverKind::Raft
                | MoverKind::Canoe
                | MoverKind::Ring
        )
    }

    pub fn is_vehicle(self) -> bool {
        !self.is_air() && !self.is_float()
    }

    /// Small enough for a sidewalk.
    pub fn is_pedestrian(self) -> bool {
        matches!(
            self,
            MoverKind::Bicycle | MoverKind::Stroller | MoverKind::Wagon | MoverKind::Unicycle | MoverKind::Scooter | MoverKind::Tandem
        )
    }
}

/// One thing in a lane's loop: its kind, size and where it starts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoverSpec {
    pub kind: MoverKind,
    /// The count for sized kinds (logs, limos, turtles), else 1.
    pub n: u32,
    pub len: f32,
    /// Start of the mover along the loop at time zero, in tiles.
    pub at: f32,
}

/// A train that crosses a rail lane every so often.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrainSpec {
    pub carriages: u32,
    /// Seconds between one train's warning and the next's.
    pub every: f32,
    /// Seconds the crossing lights flash before the train arrives.
    pub warn: f32,
    pub speed: f32,
    pub offset: f32,
}

/// Length of the engine and of each carriage, in tiles.
pub const ENGINE_LEN: f32 = 2.6;
pub const CARRIAGE_LEN: f32 = 2.4;

impl TrainSpec {
    pub fn length(&self) -> f32 {
        ENGINE_LEN + self.carriages as f32 * CARRIAGE_LEN
    }
}

/// A row of the field and its traffic. Each layer (ground, air) moves along
/// its own loop: a mover leaving one side comes back on the other after
/// travelling the loop's hidden part.
#[derive(Clone, Debug, PartialEq)]
pub struct Lane {
    pub kind: LaneKind,
    pub speed: f32,
    pub loop_len: f32,
    pub movers: Vec<MoverSpec>,
    pub air_speed: f32,
    pub air_loop: f32,
    pub air: Vec<MoverSpec>,
    /// Bunting is strung across this row: no flying into it.
    pub bunting: bool,
    /// Treats turn up here.
    pub items: bool,
    /// A fox patrols this verge.
    pub fox: bool,
    /// Seconds in a diving turtle's cycle.
    pub dive_period: f32,
    pub train: Option<TrainSpec>,
}

impl Lane {
    fn new(kind: LaneKind) -> Self {
        Lane {
            kind,
            speed: 0.0,
            loop_len: 0.0,
            movers: Vec::new(),
            air_speed: 0.0,
            air_loop: 0.0,
            air: Vec::new(),
            bunting: false,
            items: false,
            fox: false,
            dive_period: 7.0,
            train: None,
        }
    }

    pub fn is_safe_ground(&self) -> bool {
        matches!(self.kind, LaneKind::Verge | LaneKind::Start)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Level {
    pub name: String,
    pub tagline: String,
    pub width: i32,
    pub lanes: Vec<Lane>,
    /// Seconds allowed for each crossing.
    pub time: f32,
    pub nests: u32,
    /// Seconds of flight a full meter holds.
    pub flight: f32,
    /// Seconds for an empty meter to refill.
    pub recharge: f32,
    /// Drift of a flying hen, tiles per second (positive: rightwards).
    pub wind: f32,
    pub hawk: bool,
    /// Lost chicks ride the river now and then.
    pub chicks: bool,
    /// Loki, a small child on a space hopper, chases the hen about the
    /// field for a cuddle.
    pub loki: bool,
    /// So does his big sister Skaði.
    pub skadi: bool,
}

impl Level {
    pub fn height(&self) -> i32 {
        self.lanes.len() as i32
    }

    pub fn lane(&self, row: i32) -> Option<&Lane> {
        usize::try_from(row).ok().and_then(|r| self.lanes.get(r))
    }

    pub fn start_row(&self) -> i32 {
        self.height() - 1
    }

    /// Centre x of each nest box.
    pub fn nest_xs(&self) -> Vec<f32> {
        let w = self.width as f32;
        let n = self.nests.max(1);
        (0..n).map(|k| (k as f32 + 0.5) * w / n as f32).collect()
    }

    /// Makes every mover faster by `factor` (later rounds).
    pub fn quicken(&mut self, factor: f32) {
        for l in &mut self.lanes {
            l.speed *= factor;
            l.air_speed *= factor;
            if let Some(t) = &mut l.train {
                t.speed *= factor;
                t.every /= factor;
                t.every = t.every.max(min_train_every(t, self.width));
            }
            l.dive_period /= factor;
        }
        self.recharge *= factor;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

fn min_train_every(t: &TrainSpec, width: i32) -> f32 {
    t.warn + (width as f32 + t.length() + 2.0) / t.speed.abs().max(0.1) + 1.0
}

/// The loop must hold the visible width plus room for the longest mover to
/// be fully hidden on both sides, and every mover's starting position.
fn fit_loop(requested: f32, width: i32, movers: &[MoverSpec]) -> f32 {
    let longest = movers.iter().map(|m| m.len).fold(0.0f32, f32::max);
    let extent = movers.iter().map(|m| m.at + m.len).fold(0.0f32, f32::max);
    requested.max(width as f32 + 2.0 * longest + 2.0).max(extent + 1.0)
}

pub fn parse(text: &str) -> Result<Level, ParseError> {
    let mut lvl = Level {
        name: "Untitled".into(),
        tagline: String::new(),
        width: 20,
        lanes: Vec::new(),
        time: 45.0,
        nests: 5,
        flight: 1.23,
        recharge: 16.0,
        wind: 0.0,
        hawk: false,
        chicks: true,
        loki: false,
        skadi: false,
    };
    let mut in_lanes = false;
    let mut loops: Vec<(f32, f32)> = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line_no = i + 1;
        let err = |message: String| ParseError { line: line_no, message };
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if !in_lanes {
            let Some((key, value)) = line.split_once(':') else {
                return Err(err(format!("expected `key: value`, found `{line}`")));
            };
            let value = value.trim();
            let num = |v: &str| v.parse::<f32>().map_err(|_| err(format!("`{key}` wants a number, found `{v}`")));
            match key.trim() {
                "name" => lvl.name = value.to_string(),
                "tagline" => lvl.tagline = value.to_string(),
                "width" => {
                    lvl.width = value.parse().map_err(|_| err(format!("`width` wants a whole number, found `{value}`")))?;
                    if lvl.width < 4 {
                        return Err(err("the field must be at least 4 tiles wide".into()));
                    }
                }
                "time" => lvl.time = num(value)?.max(5.0),
                "nests" => {
                    lvl.nests = value.parse().map_err(|_| err(format!("`nests` wants a whole number, found `{value}`")))?;
                    if lvl.nests == 0 {
                        return Err(err("a level needs at least one nest".into()));
                    }
                }
                "flight" => lvl.flight = num(value)?.max(0.1),
                "recharge" => lvl.recharge = num(value)?.max(0.1),
                "wind" => lvl.wind = num(value)?,
                "hawk" => lvl.hawk = matches!(value, "yes" | "true" | "on"),
                "chicks" => lvl.chicks = matches!(value, "yes" | "true" | "on"),
                "loki" => lvl.loki = matches!(value, "yes" | "true" | "on"),
                "skadi" | "skaði" => lvl.skadi = matches!(value, "yes" | "true" | "on"),
                "lanes" => in_lanes = true,
                other => return Err(err(format!("unknown setting `{other}`"))),
            }
            continue;
        }
        let mut words = line.split_whitespace();
        let kind = match words.next().unwrap_or("") {
            "nests" => LaneKind::Nests,
            "verge" => LaneKind::Verge,
            "start" => LaneKind::Start,
            "road" => LaneKind::Road,
            "sidewalk" => LaneKind::Sidewalk,
            "river" => LaneKind::River,
            "rail" => LaneKind::Rail,
            other => return Err(err(format!("unknown lane kind `{other}`"))),
        };
        let mut lane = Lane::new(kind);
        let mut req_loop = 0.0f32;
        let mut req_air_loop = 0.0f32;
        let mut train = TrainSpec {
            carriages: 3,
            every: 14.0,
            warn: 2.5,
            speed: 14.0,
            offset: 0.0,
        };
        for w in words {
            if let Some((k, v)) = w.split_once('=') {
                let f = || v.parse::<f32>().map_err(|_| err(format!("`{k}` wants a number, found `{v}`")));
                match k {
                    "speed" => {
                        lane.speed = f()?;
                        train.speed = lane.speed;
                    }
                    "loop" => req_loop = f()?,
                    "air" => lane.air_speed = f()?,
                    "airloop" => req_air_loop = f()?,
                    "dive" => lane.dive_period = f()?.max(1.0),
                    "train" => train.carriages = v.parse().map_err(|_| err(format!("`train` wants a whole number, found `{v}`")))?,
                    "every" => train.every = f()?,
                    "warn" => train.warn = f()?.max(0.5),
                    "offset" => train.offset = f()?,
                    _ => return Err(err(format!("unknown option `{k}`"))),
                }
                continue;
            }
            match w {
                "bunting" => {
                    lane.bunting = true;
                    continue;
                }
                "items" => {
                    lane.items = true;
                    continue;
                }
                "fox" => {
                    lane.fox = true;
                    continue;
                }
                _ => {}
            }
            let (air, spec) = match w.strip_prefix('^') {
                Some(rest) => (true, rest),
                None => (false, w),
            };
            let Some((name, at)) = spec.split_once('@') else {
                return Err(err(format!("expected an option, a flag or `kind@position`, found `{w}`")));
            };
            let at: f32 = at.parse().map_err(|_| err(format!("bad position in `{w}`")))?;
            if at < 0.0 {
                return Err(err(format!("positions start at 0, found `{w}`")));
            }
            let Some((mk, n)) = MoverKind::parse(name) else {
                return Err(err(format!("unknown mover `{name}`")));
            };
            let n = match (mk.sized(), n) {
                (Some(default), None) => default,
                (Some(_), Some(0)) => return Err(err(format!("`{w}` has no length"))),
                (Some(_), Some(n)) => n,
                (None, None) => 1,
                (None, Some(_)) => return Err(err(format!("`{}` comes in one size", mk.name()))),
            };
            if mk.is_air() != air {
                return Err(err(if air {
                    format!("`{}` is not something that flies", mk.name())
                } else {
                    format!("`{}` flies: write it `^{}@…`", mk.name(), mk.name())
                }));
            }
            let ok = air
                || match kind {
                    LaneKind::Road => mk.is_vehicle(),
                    LaneKind::Sidewalk => mk.is_pedestrian(),
                    LaneKind::River => mk.is_float(),
                    _ => false,
                };
            if !ok {
                return Err(err(format!("a `{}` does not belong in this kind of lane", mk.name())));
            }
            let spec = MoverSpec {
                kind: mk,
                n,
                len: mk.length(n),
                at,
            };
            if air {
                lane.air.push(spec);
            } else {
                lane.movers.push(spec);
            }
        }
        if kind == LaneKind::Rail {
            if train.speed == 0.0 {
                train.speed = 14.0;
            }
            lane.speed = train.speed;
            lane.train = Some(train);
        }
        if matches!(kind, LaneKind::Road | LaneKind::Sidewalk | LaneKind::River) && !lane.movers.is_empty() && lane.speed == 0.0 {
            return Err(err("traffic needs a `speed`".into()));
        }
        if kind == LaneKind::River && lane.movers.is_empty() {
            return Err(err("a river with nothing floating on it cannot be crossed on foot; give it something to ride".into()));
        }
        if !lane.air.is_empty() && lane.air_speed == 0.0 {
            return Err(err("flying things need an `air` speed".into()));
        }
        loops.push((req_loop, req_air_loop));
        lvl.lanes.push(lane);
    }
    if lvl.lanes.len() < 3 {
        return Err(ParseError {
            line: text.lines().count(),
            message: "a level needs at least three lanes (nests, something to cross, a start)".into(),
        });
    }
    if lvl.lanes[0].kind != LaneKind::Nests {
        return Err(ParseError {
            line: 0,
            message: "the first lane must be `nests`".into(),
        });
    }
    if lvl.lanes.last().map(|l| l.kind) != Some(LaneKind::Start) {
        return Err(ParseError {
            line: 0,
            message: "the last lane must be `start`".into(),
        });
    }
    if let Some(i) = lvl.lanes.iter().skip(1).position(|l| l.kind == LaneKind::Nests) {
        return Err(ParseError {
            line: 0,
            message: format!("only the first lane may be `nests` (lane {} is too)", i + 2),
        });
    }
    let width = lvl.width;
    for (lane, (req, req_air)) in lvl.lanes.iter_mut().zip(loops) {
        lane.loop_len = fit_loop(req, width, &lane.movers);
        lane.air_loop = fit_loop(req_air, width, &lane.air);
        if let Some(t) = &mut lane.train {
            t.every = t.every.max(min_train_every(t, width));
        }
    }
    Ok(lvl)
}

/// The levels that ship with the game, in order.
pub const BUILTIN: [(&str, &str); 6] = [
    ("01-country-lane", include_str!("../../assets/levels/01-country-lane.txt")),
    ("02-market-day", include_str!("../../assets/levels/02-market-day.txt")),
    ("03-millrace", include_str!("../../assets/levels/03-millrace.txt")),
    ("04-bypass", include_str!("../../assets/levels/04-bypass.txt")),
    ("05-village-fete", include_str!("../../assets/levels/05-village-fete.txt")),
    ("06-harvest-moon", include_str!("../../assets/levels/06-harvest-moon.txt")),
];

pub fn builtin(index: usize) -> Option<Level> {
    let (name, text) = BUILTIN.get(index)?;
    match parse(text) {
        Ok(l) => Some(l),
        Err(e) => panic!("built-in level {name} does not parse: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_level_parses() {
        for i in 0..BUILTIN.len() {
            let l = builtin(i).unwrap();
            assert!(l.height() >= 3);
            for lane in &l.lanes {
                for m in lane.movers.iter().chain(&lane.air) {
                    assert!(m.at + m.len <= lane.loop_len.max(lane.air_loop));
                }
            }
        }
    }

    #[test]
    fn reports_mistakes_with_their_line() {
        let e = parse("name: x\nlanes:\nnests\nroad speed=1 hatchback@0 log2@5\nstart\n").unwrap_err();
        assert_eq!(e.line, 4);
        assert!(e.message.contains("log"), "{e}");
        let e = parse("name: x\nlanes:\nnests\nroad speed=1 kite@0\nstart\n").unwrap_err();
        assert!(e.message.contains("^kite"), "{e}");
        let e = parse("name: x\nlanes:\nnests\nriver speed=1\nstart\n").unwrap_err();
        assert!(e.message.contains("ride"), "{e}");
    }

    #[test]
    fn loops_make_room_for_hidden_movers() {
        let l = parse("width: 10\nlanes:\nnests\nroad speed=1 limo7@0\nstart\n").unwrap();
        assert!(l.lanes[1].loop_len >= 10.0 + 14.0);
    }

    #[test]
    fn sized_movers_take_their_count() {
        let l = parse("lanes:\nnests\nriver speed=1 log5@0 divers2@9\nroad speed=-2 limo6@0 sofa@9 air=1 ^balloon@2\nstart\n").unwrap();
        assert_eq!(l.lanes[1].movers[0].len, 5.0);
        assert_eq!(l.lanes[1].movers[1].kind, MoverKind::Divers);
        assert_eq!(l.lanes[2].movers[0].n, 6);
        assert_eq!(l.lanes[2].air.len(), 1);
        assert!(parse("lanes:\nnests\nroad speed=1 sofa3@0\nstart\n").is_err());
    }
}
