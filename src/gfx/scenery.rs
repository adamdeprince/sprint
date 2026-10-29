//! The field, drawn as vector scenes from a level.
//!
//! Two scenes are produced: the ground (verges, roads, the river, the
//! railway, the henhouse wall of nest boxes) drawn beneath everything that
//! moves, and the canopy (bunting) drawn over what walks but under what
//! flies. Things that change during play (an egg appearing in a nest box,
//! the crossing lamps flashing) are "decals": a few elements with stable
//! ids, so a change is a patch of a few records on Silkie and a placement
//! from a small atlas elsewhere.
//!
//! Units: one tile is [`TILE`] user units.

use super::vector::{Affine, Color, Element, Fill, PathData, Scene, StrokeStyle};
use crate::world::level::{LaneKind, Level};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub const TILE: f32 = 64.0;

// Groups of the ground scene, created in this order (which is drawing order).
pub const G_TERRAIN: u32 = 1;
pub const G_MARKINGS: u32 = 2;
pub const G_DETAIL: u32 = 3;
pub const G_DECALS: u32 = 4;
// Groups of the canopy scene.
pub const G_BUNTING: u32 = 1;

const SYM_TUFT: u32 = 100;
const SYM_FLOWER: u32 = 101;
const SYM_PEBBLE: u32 = 102;
const SYM_FLAG: u32 = 103;
const GRAD_GRASS: u32 = 120;
const GRAD_WATER: u32 = 121;
const GRAD_ROAD: u32 = 122;

const STATIC_BASE: u32 = 1000;
/// Each decal slot owns this many element ids from here.
pub const DECAL_BASE: u32 = 1_000_000;
pub const DECAL_IDS: u32 = 16;

pub mod palette {
    use super::Color;
    pub const GRASS: Color = Color::hex(0x8cc169);
    pub const GRASS_LIGHT: Color = Color::hex(0xa2d07b);
    pub const GRASS_DARK: Color = Color::hex(0x6ea852);
    pub const TUFT: Color = Color::hex(0x5f9a45);
    pub const ROAD: Color = Color::hex(0x5d6168);
    pub const ROAD_LIGHT: Color = Color::hex(0x64686f);
    pub const ROAD_FLECK: Color = Color::hex(0x777c84);
    pub const LINE: Color = Color::hex(0xf2efe6);
    pub const LINE_YELLOW: Color = Color::hex(0xf2c94c);
    pub const KERB: Color = Color::hex(0xc9c4b8);
    pub const KERB_DARK: Color = Color::hex(0x9b958a);
    pub const WATER: Color = Color::hex(0x5fa9d6);
    pub const WATER_DEEP: Color = Color::hex(0x3f88bd);
    pub const WATER_SHINE: Color = Color::hex(0xc4e9f8);
    pub const BANK: Color = Color::hex(0xb08a5a);
    pub const BANK_DARK: Color = Color::hex(0x7d5f3c);
    pub const REED: Color = Color::hex(0x6f9a3e);
    pub const PAVING: Color = Color::hex(0xd8d2c6);
    pub const PAVING_LIGHT: Color = Color::hex(0xe6e1d6);
    pub const PAVING_JOINT: Color = Color::hex(0xb5ad9e);
    pub const CHALK: Color = Color::hex(0xfdfdfd);
    pub const BALLAST: Color = Color::hex(0xa59d90);
    pub const BALLAST_DARK: Color = Color::hex(0x857d71);
    pub const SLEEPER: Color = Color::hex(0x7a5838);
    pub const RAIL: Color = Color::hex(0xc7ccd3);
    pub const RAIL_DARK: Color = Color::hex(0x6d737c);
    pub const PLANK: Color = Color::hex(0xb2553f);
    pub const PLANK_DARK: Color = Color::hex(0x8a3f2f);
    pub const TRIM: Color = Color::hex(0xf6efe2);
    pub const BOX: Color = Color::hex(0x5a3a26);
    pub const STRAW: Color = Color::hex(0xe9c46a);
    pub const STRAW_DARK: Color = Color::hex(0xc79a3a);
    pub const EGG: Color = Color::hex(0xfaf3e3);
    pub const EGG_SHADE: Color = Color::hex(0xe2d3b4);
    pub const LAMP_OFF: Color = Color::hex(0x5a2a2a);
    pub const LAMP_ON: Color = Color::hex(0xff3b30);
    pub const POST: Color = Color::hex(0x3a3a40);
    pub const WOOD: Color = Color::hex(0xc48a51);
    pub const WOOD_DARK: Color = Color::hex(0x8c5d33);
    pub const FLAGS: [Color; 5] = [
        Color::hex(0xe53935),
        Color::hex(0xffd54f),
        Color::hex(0x42a5f5),
        Color::hex(0x66bb6a),
        Color::hex(0xf48fb1),
    ];
    pub const FLOWERS: [Color; 5] = [
        Color::hex(0xf48fb1),
        Color::hex(0xffd54f),
        Color::hex(0xce93d8),
        Color::hex(0xfdfdfd),
        Color::hex(0xffa94d),
    ];
}
use palette as pal;

pub struct Scenery {
    pub ground: Scene,
    pub canopy: Scene,
    pub width: f32,
    pub height: f32,
}

/// Something on the ground that changes during play.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecalKind {
    /// A nest box: 0 empty, 1 an egg, 2 a hatched shell.
    Nest,
    /// A level-crossing lamp pair: 0 dark, 1 and 2 alternate.
    Lamps,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecalSlot {
    pub kind: DecalKind,
    /// Centre, in tiles.
    pub at: (f32, f32),
}

/// States every decal has (atlas columns).
pub const DECAL_STATES: u8 = 3;

/// The decals of a level, in a fixed order: nests left to right, then a
/// pair of crossing lamps (left post, right post) for each railway.
pub fn decal_slots(level: &Level) -> Vec<DecalSlot> {
    let mut v: Vec<DecalSlot> = level
        .nest_xs()
        .into_iter()
        .map(|x| DecalSlot {
            kind: DecalKind::Nest,
            at: (x, 0.5),
        })
        .collect();
    let w = level.width as f32;
    for (row, lane) in level.lanes.iter().enumerate() {
        if lane.kind == LaneKind::Rail {
            // The posts stand on the bank above the line, at each side.
            for x in [0.45, w - 0.45] {
                v.push(DecalSlot {
                    kind: DecalKind::Lamps,
                    at: (x, row as f32 - 0.05),
                });
            }
        }
    }
    v
}

struct Builder {
    rng: ChaCha8Rng,
    ground: Scene,
    canopy: Scene,
    next: u32,
}

impl Builder {
    fn put(&mut self, e: Element) {
        self.next += 1;
        self.ground.set(self.next, e);
    }

    fn put_canopy(&mut self, e: Element) {
        self.next += 1;
        self.canopy.set(self.next, e);
    }

    fn r(&mut self, lo: f32, hi: f32) -> f32 {
        self.rng.random_range(lo..hi)
    }
}

fn seed_for(level: &Level) -> u64 {
    level
        .name
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3))
}

pub fn build(level: &Level) -> Scenery {
    let width = level.width as f32 * TILE;
    let height = level.height() as f32 * TILE;
    let mut b = Builder {
        rng: ChaCha8Rng::seed_from_u64(seed_for(level)),
        ground: Scene::new(width, height),
        canopy: Scene::new(width, height),
        next: STATIC_BASE,
    };
    for g in [G_TERRAIN, G_MARKINGS, G_DETAIL, G_DECALS] {
        b.ground.set(g, Element::group(0));
    }
    b.canopy.set(G_BUNTING, Element::group(0));
    symbols(&mut b.ground);
    symbols(&mut b.canopy);
    let kinds: Vec<LaneKind> = level.lanes.iter().map(|l| l.kind).collect();
    for (row, lane) in level.lanes.iter().enumerate() {
        let y = row as f32 * TILE;
        match lane.kind {
            LaneKind::Nests => henhouse(&mut b, level, y),
            LaneKind::Verge | LaneKind::Start => verge(&mut b, width, y, lane.kind == LaneKind::Start),
            LaneKind::Road => road(&mut b, width, y),
            LaneKind::Sidewalk => sidewalk(&mut b, width, y),
            LaneKind::River => river(&mut b, width, y),
            LaneKind::Rail => rail(&mut b, width, y),
        }
    }
    // Edges between lanes of different kinds: kerbs, banks, lane lines.
    for row in 0..kinds.len() {
        let y = row as f32 * TILE;
        let above = row.checked_sub(1).map(|r| kinds[r]);
        let here = kinds[row];
        let below = kinds.get(row + 1).copied();
        match here {
            LaneKind::Road => {
                road_edge(&mut b, width, y, above != Some(LaneKind::Road), true);
                road_edge(&mut b, width, y + TILE, below != Some(LaneKind::Road), false);
            }
            LaneKind::Sidewalk => {
                // A grass edge where the pavement meets a verge.
                for (edge, other) in [(y, above), (y + TILE - 3.0, below)] {
                    if other.is_some_and(|k| matches!(k, LaneKind::Verge | LaneKind::Start)) {
                        b.put(Element::fill_path(G_MARKINGS, PathData::rect(0.0, edge, width, 3.0), pal::GRASS_DARK));
                    }
                }
            }
            LaneKind::River => {
                if above.is_some_and(|k| k != LaneKind::River) {
                    bank(&mut b, width, y, true, above == Some(LaneKind::Nests));
                }
                if below.is_some_and(|k| k != LaneKind::River) {
                    bank(&mut b, width, y + TILE, false, false);
                }
            }
            _ => {}
        }
    }
    for (row, lane) in level.lanes.iter().enumerate() {
        if lane.bunting {
            bunting(&mut b, width, row as f32 * TILE);
        }
    }
    let mut s = Scenery {
        ground: b.ground,
        canopy: b.canopy,
        width,
        height,
    };
    for (i, slot) in decal_slots(level).into_iter().enumerate() {
        for (id, e) in decal_elements(i, &slot, 0) {
            s.ground.set(id, e);
        }
    }
    s
}

fn symbols(s: &mut Scene) {
    // A tuft of three blades, about 14 units tall, origin at its base.
    let mut tuft = PathData::new();
    tuft.move_to(-6.0, 0.0).quad_to(-7.0, -8.0, -9.0, -12.0).quad_to(-4.0, -7.0, -2.0, 0.0).close();
    tuft.move_to(-2.0, 0.0).quad_to(-1.0, -10.0, 1.0, -15.0).quad_to(2.5, -8.0, 2.0, 0.0).close();
    tuft.move_to(2.0, 0.0).quad_to(5.0, -7.0, 9.0, -10.0).quad_to(6.0, -5.0, 6.0, 0.0).close();
    s.set(SYM_TUFT, Element::Symbol { d: tuft });
    let mut flower = PathData::new();
    for k in 0..5 {
        let a = k as f32 * std::f32::consts::TAU / 5.0 - std::f32::consts::FRAC_PI_2;
        flower.extend(&PathData::circle(a.cos() * 4.2, a.sin() * 4.2, 3.6));
    }
    s.set(SYM_FLOWER, Element::Symbol { d: flower });
    s.set(SYM_PEBBLE, Element::Symbol { d: PathData::ellipse(0.0, 0.0, 2.6, 1.8) });
    s.set(SYM_FLAG, Element::Symbol { d: PathData::polygon(&[(-7.0, 0.0), (7.0, 0.0), (0.0, 15.0)]) });
    s.set(
        GRAD_GRASS,
        Element::Linear {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: TILE,
            stops: vec![(0.0, pal::GRASS_LIGHT), (1.0, pal::GRASS)],
        },
    );
    s.set(
        GRAD_WATER,
        Element::Linear {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: TILE,
            stops: vec![(0.0, pal::WATER), (0.5, pal::WATER_DEEP), (1.0, pal::WATER)],
        },
    );
    s.set(
        GRAD_ROAD,
        Element::Linear {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: TILE,
            stops: vec![(0.0, pal::ROAD_LIGHT), (1.0, pal::ROAD)],
        },
    );
}

/// A full-width band for one row, with its own gradient running down the
/// row (gradients are given in the scene's coordinates).
fn band(b: &mut Builder, w: f32, y: f32, grad: u32) {
    let stops = match b.ground.get(grad) {
        Some(Element::Linear { stops, .. }) => stops.clone(),
        _ => vec![(0.0, pal::GRASS)],
    };
    b.next += 1;
    let gid = b.next;
    b.ground.set(
        gid,
        Element::Linear {
            x1: 0.0,
            y1: y,
            x2: 0.0,
            y2: y + TILE,
            stops,
        },
    );
    b.put(Element::fill_path(G_TERRAIN, PathData::rect(0.0, y, w, TILE), Fill::Gradient(gid)));
}

fn verge(b: &mut Builder, w: f32, y: f32, start: bool) {
    band(b, w, y, GRAD_GRASS);
    let cols = (w / TILE) as i32;
    for c in 0..cols {
        let n = b.rng.random_range(1..4);
        for _ in 0..n {
            let x = c as f32 * TILE + b.r(4.0, 60.0);
            let ty = y + b.r(14.0, 58.0);
            let s = b.r(0.7, 1.15);
            let shade = b.r(-0.12, 0.1);
            b.put(Element::Use {
                group: G_DETAIL,
                symbol: SYM_TUFT,
                t: Some(Affine([s, 0.0, 0.0, s, x, ty])),
                fill: Fill::Solid(pal::TUFT.shade(shade).with_alpha(200)),
            });
        }
        if b.rng.random_bool(0.3) {
            let x = c as f32 * TILE + b.r(8.0, 56.0);
            let fy = y + b.r(12.0, 52.0);
            let col = pal::FLOWERS[b.rng.random_range(0..pal::FLOWERS.len())];
            b.put(Element::Use {
                group: G_DETAIL,
                symbol: SYM_FLOWER,
                t: Some(Affine([0.8, 0.0, 0.0, 0.8, x, fy])),
                fill: Fill::Solid(col),
            });
            b.put(Element::fill_path(G_DETAIL, PathData::circle(x, fy, 2.0), Color::hex(0xffeb7a)));
        }
    }
    if start {
        // A signpost at the start, pointing hopefully upwards.
        let x = TILE * 0.45;
        let mut post = PathData::new();
        post.move_to(x, y + 58.0).line_to(x, y + 14.0);
        b.put(Element::stroke_path(G_DETAIL, post, StrokeStyle::new(pal::WOOD_DARK, 6.0)));
        let sign = PathData::polygon(&[(x - 12.0, y + 10.0), (x + 12.0, y + 10.0), (x + 12.0, y + 22.0), (x, y + 30.0), (x - 12.0, y + 22.0)]);
        b.put(Element::Path {
            group: G_DETAIL,
            d: sign,
            fill: Fill::Solid(pal::WOOD),
            even_odd: false,
            stroke: Some(StrokeStyle::new(pal::WOOD_DARK, 2.0)),
            t: None,
        });
        let mut arrow = PathData::new();
        arrow.move_to(x, y + 24.0).line_to(x, y + 13.0).move_to(x - 5.0, y + 18.0).line_to(x, y + 13.0).line_to(x + 5.0, y + 18.0);
        b.put(Element::stroke_path(G_DETAIL, arrow, StrokeStyle::new(Color::hex(0x4a2a1a), 2.4)));
    } else if b.rng.random_bool(0.5) {
        // A garden gnome, keeping an eye on the traffic.
        let x = b.r(TILE, w - TILE);
        let gy = y + 50.0;
        b.put(Element::fill_path(G_DETAIL, PathData::ellipse(x, gy + 2.0, 9.0, 3.0), Color::rgba(30, 50, 20, 70)));
        b.put(Element::fill_path(G_DETAIL, PathData::round_rect(x - 6.0, gy - 16.0, 12.0, 16.0, 4.0), Color::hex(0x3e64a8)));
        b.put(Element::fill_path(G_DETAIL, PathData::circle(x, gy - 20.0, 5.0), Color::hex(0xf2c6a0)));
        b.put(Element::fill_path(G_DETAIL, PathData::ellipse(x, gy - 14.0, 5.0, 6.0), Color::hex(0xfdfdfd)));
        b.put(Element::fill_path(
            G_DETAIL,
            PathData::polygon(&[(x - 6.0, gy - 22.0), (x + 6.0, gy - 22.0), (x + 1.0, gy - 38.0)]),
            Color::hex(0xe53935),
        ));
    }
}

fn road(b: &mut Builder, w: f32, y: f32) {
    band(b, w, y, GRAD_ROAD);
    // Flecks of grit and the odd patch of repair.
    let mut flecks = PathData::new();
    let n = (w / TILE * 6.0) as usize;
    for _ in 0..n {
        let x = b.r(0.0, w);
        let fy = y + b.r(4.0, 60.0);
        flecks.extend(&PathData::circle(x, fy, b.r(0.8, 1.8)));
    }
    b.put(Element::fill_path(G_DETAIL, flecks, pal::ROAD_FLECK));
    if b.rng.random_bool(0.6) {
        let x = b.r(TILE, w - TILE);
        let (py, pw, ph) = (y + b.r(10.0, 30.0), b.r(30.0, 70.0), b.r(14.0, 24.0));
        b.put(Element::fill_path(G_DETAIL, PathData::round_rect(x, py, pw, ph, 4.0), pal::ROAD.shade(-0.08)));
    }
    if b.rng.random_bool(0.4) {
        let x = b.r(TILE, w - TILE);
        let my = y + 32.0;
        b.put(Element::Path {
            group: G_DETAIL,
            d: PathData::circle(x, my, 9.0),
            fill: Fill::Solid(Color::hex(0x4e5157)),
            even_odd: false,
            stroke: Some(StrokeStyle::new(Color::hex(0x7d8188), 2.0)),
            t: None,
        });
    }
}

fn sidewalk(b: &mut Builder, w: f32, y: f32) {
    b.next += 1;
    let gid = b.next;
    b.ground.set(
        gid,
        Element::Linear {
            x1: 0.0,
            y1: y,
            x2: 0.0,
            y2: y + TILE,
            stops: vec![(0.0, pal::PAVING_LIGHT), (1.0, pal::PAVING)],
        },
    );
    b.put(Element::fill_path(G_TERRAIN, PathData::rect(0.0, y, w, TILE), Fill::Gradient(gid)));
    // Slabs: two courses, staggered.
    let mut joints = PathData::new();
    joints.move_to(0.0, y + TILE / 2.0).line_to(w, y + TILE / 2.0);
    let slab = TILE * 0.75;
    let mut x = 0.0;
    while x < w {
        joints.move_to(x, y).line_to(x, y + TILE / 2.0);
        joints.move_to(x + slab / 2.0, y + TILE / 2.0).line_to(x + slab / 2.0, y + TILE);
        x += slab;
    }
    b.put(Element::stroke_path(G_DETAIL, joints, StrokeStyle::new(pal::PAVING_JOINT, 1.4)));
    // A chalk hopscotch, here and there.
    if b.rng.random_bool(0.6) {
        let x0 = b.r(TILE, w - TILE * 4.0);
        let mut chalk = PathData::new();
        for i in 0..5 {
            let cx = x0 + i as f32 * 22.0;
            if i % 2 == 1 {
                chalk.extend(&PathData::rect(cx, y + 10.0, 22.0, 22.0));
                chalk.extend(&PathData::rect(cx, y + 32.0, 22.0, 22.0));
            } else {
                chalk.extend(&PathData::rect(cx, y + 21.0, 22.0, 22.0));
            }
        }
        b.put(Element::stroke_path(G_DETAIL, chalk, StrokeStyle::new(pal::CHALK.with_alpha(200), 1.8)));
    }
    if b.rng.random_bool(0.5) {
        let x = b.r(TILE, w - TILE);
        b.put(Element::Path {
            group: G_DETAIL,
            d: PathData::round_rect(x, y + 20.0, 26.0, 18.0, 2.0),
            fill: Fill::Solid(Color::hex(0x8d877c)),
            even_odd: false,
            stroke: Some(StrokeStyle::new(Color::hex(0x6e695f), 1.4)),
            t: None,
        });
    }
}

/// The line along the top (or bottom) of a road row: a kerb where the road
/// ends, dashes between two lanes.
fn road_edge(b: &mut Builder, w: f32, y: f32, kerb: bool, top: bool) {
    if kerb {
        let ky = if top { y } else { y - 5.0 };
        b.put(Element::fill_path(G_MARKINGS, PathData::rect(0.0, ky, w, 5.0), pal::KERB));
        let mut joints = PathData::new();
        let mut x = 0.0;
        while x < w {
            joints.move_to(x, ky).line_to(x, ky + 5.0);
            x += TILE * 0.5;
        }
        b.put(Element::stroke_path(G_MARKINGS, joints, StrokeStyle::new(pal::KERB_DARK, 1.2)));
        let ly = if top { y + 9.0 } else { y - 11.0 };
        b.put(Element::fill_path(G_MARKINGS, PathData::rect(0.0, ly, w, 2.4), pal::LINE_YELLOW));
    } else if top {
        let mut dashes = PathData::new();
        let mut x = TILE * 0.25;
        while x < w {
            dashes.extend(&PathData::rect(x, y - 1.5, TILE * 0.55, 3.0));
            x += TILE * 1.1;
        }
        b.put(Element::fill_path(G_MARKINGS, dashes, pal::LINE));
    }
}

fn river(b: &mut Builder, w: f32, y: f32) {
    band(b, w, y, GRAD_WATER);
    let cols = (w / TILE) as i32;
    let mut ripples = PathData::new();
    for c in 0..cols {
        if b.rng.random_bool(0.7) {
            let x = c as f32 * TILE + b.r(8.0, 56.0);
            let ry = y + b.r(10.0, 54.0);
            ripples.move_to(x - 9.0, ry).quad_to(x, ry - 4.0, x + 9.0, ry);
        }
    }
    b.put(Element::stroke_path(G_DETAIL, ripples, StrokeStyle::new(pal::WATER_SHINE.with_alpha(170), 2.0)));
}

/// A muddy bank with reeds where water meets anything else.
fn bank(b: &mut Builder, w: f32, y: f32, top: bool, wall: bool) {
    let (by, h) = if top { (y - 3.0, 9.0) } else { (y - 6.0, 9.0) };
    b.put(Element::fill_path(G_MARKINGS, PathData::rect(0.0, by, w, h), pal::BANK));
    b.put(Element::fill_path(
        G_MARKINGS,
        PathData::rect(0.0, if top { by + h - 3.0 } else { by }, w, 3.0),
        pal::BANK_DARK,
    ));
    if wall {
        return;
    }
    let mut reeds = PathData::new();
    let mut x = b.r(0.0, TILE);
    while x < w {
        let ry = if top { y + 4.0 } else { y - 4.0 };
        let dir = if top { 1.0 } else { -1.0 };
        for k in 0..3 {
            let rx = x + k as f32 * 4.0;
            reeds.move_to(rx, ry).quad_to(rx + 2.0, ry - dir * 8.0, rx + b.r(-3.0, 5.0), ry - dir * b.r(12.0, 18.0));
        }
        x += b.r(TILE * 0.8, TILE * 2.4);
    }
    b.put(Element::stroke_path(G_DETAIL, reeds, StrokeStyle::new(pal::REED, 2.2)));
}

fn rail(b: &mut Builder, w: f32, y: f32) {
    band(b, w, y, GRAD_GRASS);
    b.put(Element::fill_path(G_TERRAIN, PathData::rect(0.0, y + 8.0, w, TILE - 16.0), pal::BALLAST));
    let mut stones = PathData::new();
    let n = (w / TILE * 10.0) as usize;
    for _ in 0..n {
        let x = b.r(0.0, w);
        let sy = y + b.r(10.0, TILE - 10.0);
        stones.extend(&PathData::circle(x, sy, b.r(1.0, 2.4)));
    }
    b.put(Element::fill_path(G_DETAIL, stones, pal::BALLAST_DARK));
    let mut sleepers = PathData::new();
    let mut x = TILE * 0.2;
    while x < w {
        sleepers.extend(&PathData::rect(x, y + 14.0, 9.0, TILE - 26.0));
        x += TILE * 0.42;
    }
    b.put(Element::Path {
        group: G_DETAIL,
        d: sleepers,
        fill: Fill::Solid(pal::SLEEPER),
        even_odd: false,
        stroke: Some(StrokeStyle::new(pal::SLEEPER.shade(-0.3), 1.2)),
        t: None,
    });
    for ry in [y + 22.0, y + 42.0] {
        b.put(Element::fill_path(G_DETAIL, PathData::rect(0.0, ry, w, 5.0), pal::RAIL_DARK));
        b.put(Element::fill_path(G_DETAIL, PathData::rect(0.0, ry, w, 2.2), pal::RAIL));
    }
}

/// The henhouse wall along the top: red planks, white trim, and a dark
/// opening for each nest box with straw inside (the egg is a decal).
fn henhouse(b: &mut Builder, level: &Level, y: f32) {
    let w = level.width as f32 * TILE;
    b.put(Element::fill_path(G_TERRAIN, PathData::rect(0.0, y, w, TILE), pal::PLANK));
    let mut seams = PathData::new();
    for k in 1..4 {
        let sy = y + k as f32 * TILE / 4.0;
        seams.move_to(0.0, sy).line_to(w, sy);
    }
    b.put(Element::stroke_path(G_DETAIL, seams, StrokeStyle::new(pal::PLANK_DARK, 1.6)));
    b.put(Element::fill_path(G_DETAIL, PathData::rect(0.0, y + TILE - 6.0, w, 6.0), pal::TRIM));
    for x in level.nest_xs() {
        let cx = x * TILE;
        let frame = PathData::round_rect(cx - 30.0, y + 4.0, 60.0, 56.0, 10.0);
        b.put(Element::Path {
            group: G_DETAIL,
            d: frame,
            fill: Fill::Solid(pal::TRIM),
            even_odd: false,
            stroke: Some(StrokeStyle::new(pal::PLANK_DARK, 2.0)),
            t: None,
        });
        b.put(Element::fill_path(G_DETAIL, PathData::round_rect(cx - 24.0, y + 9.0, 48.0, 48.0, 8.0), pal::BOX));
        // Straw spilling over the lip.
        let mut straw = PathData::new();
        for k in 0..9 {
            let sx = cx - 22.0 + k as f32 * 5.5;
            let sy = y + 44.0 + b.r(-2.0, 2.0);
            straw.move_to(sx, sy + 12.0).quad_to(sx + b.r(-4.0, 4.0), sy, sx + b.r(-6.0, 6.0), sy - b.r(4.0, 10.0));
        }
        b.put(Element::stroke_path(G_DETAIL, straw, StrokeStyle::new(pal::STRAW, 3.0)));
        b.put(Element::fill_path(G_DETAIL, PathData::ellipse(cx, y + 52.0, 22.0, 7.0), pal::STRAW_DARK));
    }
}

fn bunting(b: &mut Builder, w: f32, y: f32) {
    // Strung along the top of the row, sagging between poles.
    let span = TILE * 4.0;
    let mut x = 0.0;
    let mut i = 0usize;
    while x < w {
        let x2 = (x + span).min(w);
        let mut string = PathData::new();
        let sag = 10.0;
        string.move_to(x, y + 2.0).quad_to((x + x2) / 2.0, y + 2.0 + sag * 2.0, x2, y + 2.0);
        b.put_canopy(Element::stroke_path(G_BUNTING, string, StrokeStyle::new(Color::hex(0x5d4037), 1.6)));
        let n = ((x2 - x) / 14.0) as usize;
        for k in 0..n {
            let t = (k as f32 + 0.5) / n as f32;
            let fx = x + (x2 - x) * t;
            // On the quadratic sag.
            let fy = y + 2.0 + 2.0 * t * (1.0 - t) * sag * 2.0;
            let col = pal::FLAGS[i % pal::FLAGS.len()];
            i += 1;
            b.put_canopy(Element::Use {
                group: G_BUNTING,
                symbol: SYM_FLAG,
                t: Some(Affine::translate(fx, fy)),
                fill: Fill::Solid(col),
            });
        }
        x = x2;
    }
}

/// The elements of decal `index` in `state`. Always the same ids and the
/// same number of them, so that a change is a patch.
pub fn decal_elements(index: usize, slot: &DecalSlot, state: u8) -> Vec<(u32, Element)> {
    let base = DECAL_BASE + index as u32 * DECAL_IDS;
    let cx = slot.at.0 * TILE;
    let cy = slot.at.1 * TILE;
    let empty = || Element::fill_path(G_DECALS, PathData::new(), Fill::None);
    let mut out = Vec::new();
    match slot.kind {
        DecalKind::Nest => {
            let egg = PathData::smooth_closed(
                &[(0.0, -15.0), (6.5, -10.0), (8.0, -3.0), (5.0, 2.0), (0.0, 3.5), (-5.0, 2.0), (-8.0, -3.0), (-6.5, -10.0)],
                1.0,
            )
            .transformed(Affine::translate(cx, cy + 14.0));
            match state {
                1 => {
                    out.push((
                        base,
                        Element::Path {
                            group: G_DECALS,
                            d: egg,
                            fill: Fill::Solid(pal::EGG),
                            even_odd: false,
                            stroke: Some(StrokeStyle::new(pal::EGG_SHADE.shade(-0.3), 1.6)),
                            t: None,
                        },
                    ));
                    out.push((base + 1, Element::fill_path(G_DECALS, PathData::ellipse(cx - 2.5, cy + 3.0, 1.8, 3.4), Color::rgba(255, 255, 255, 220))));
                }
                2 => {
                    // Two halves of a shell, the top one tipped off.
                    let mut low = PathData::new();
                    low.move_to(cx - 8.0, cy + 10.0)
                        .line_to(cx - 5.0, cy + 6.0)
                        .line_to(cx - 2.0, cy + 10.0)
                        .line_to(cx + 2.0, cy + 6.0)
                        .line_to(cx + 5.0, cy + 10.0)
                        .line_to(cx + 8.0, cy + 7.0)
                        .quad_to(cx + 9.0, cy + 17.0, cx, cy + 18.0)
                        .quad_to(cx - 9.0, cy + 17.0, cx - 8.0, cy + 10.0)
                        .close();
                    out.push((
                        base,
                        Element::Path {
                            group: G_DECALS,
                            d: low,
                            fill: Fill::Solid(pal::EGG),
                            even_odd: false,
                            stroke: Some(StrokeStyle::new(pal::EGG_SHADE.shade(-0.3), 1.4)),
                            t: None,
                        },
                    ));
                    let cap = PathData::ellipse(0.0, 0.0, 6.0, 4.0).transformed(Affine::rotate(0.6).then(Affine::translate(cx + 16.0, cy + 15.0)));
                    out.push((
                        base + 1,
                        Element::Path {
                            group: G_DECALS,
                            d: cap,
                            fill: Fill::Solid(pal::EGG),
                            even_odd: false,
                            stroke: Some(StrokeStyle::new(pal::EGG_SHADE.shade(-0.3), 1.4)),
                            t: None,
                        },
                    ));
                }
                _ => {
                    out.push((base, empty()));
                    out.push((base + 1, empty()));
                }
            }
        }
        DecalKind::Lamps => {
            // A post with a cross-buck and two lamps that flash in turn.
            let mut post = PathData::new();
            post.move_to(cx, cy + 6.0).line_to(cx, cy - 30.0);
            out.push((base, Element::stroke_path(G_DECALS, post, StrokeStyle::new(pal::POST, 4.0))));
            let mut buck = PathData::new();
            buck.move_to(cx - 10.0, cy - 40.0).line_to(cx + 10.0, cy - 28.0).move_to(cx + 10.0, cy - 40.0).line_to(cx - 10.0, cy - 28.0);
            out.push((base + 1, Element::stroke_path(G_DECALS, buck, StrokeStyle::new(Color::rgb(255, 255, 255), 4.0))));
            out.push((base + 2, Element::fill_path(G_DECALS, PathData::rect(cx - 13.0, cy - 22.0, 26.0, 9.0), pal::POST)));
            for (k, dx) in [(0u8, -8.0f32), (1, 8.0)] {
                let lit = state != 0 && (state - 1) == k;
                let lamp = PathData::circle(cx + dx, cy - 17.5, 5.0);
                out.push((
                    base + 3 + k as u32,
                    Element::Path {
                        group: G_DECALS,
                        d: lamp,
                        fill: Fill::Solid(if lit { pal::LAMP_ON } else { pal::LAMP_OFF }),
                        even_odd: false,
                        stroke: Some(StrokeStyle::new(Color::hex(0x1e1e22), 1.4)),
                        t: None,
                    },
                ));
                let glow = if lit { PathData::circle(cx + dx, cy - 17.5, 9.5) } else { PathData::new() };
                out.push((base + 5 + k as u32, Element::fill_path(G_DECALS, glow, Color::rgba(255, 80, 60, 90))));
            }
        }
    }
    out
}

/// The number of element ids decal `index` uses (for deleting them from a
/// raster copy of the ground).
pub fn decal_ids(index: usize) -> std::ops::Range<u32> {
    let base = DECAL_BASE + index as u32 * DECAL_IDS;
    base..base + DECAL_IDS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::level;

    #[test]
    fn every_level_builds_with_its_decals() {
        for i in 0..level::BUILTIN.len() {
            let l = level::builtin(i).unwrap();
            let s = build(&l);
            assert!(!s.ground.is_empty());
            let slots = decal_slots(&l);
            assert!(slots.len() >= l.nests as usize);
            for (k, slot) in slots.iter().enumerate() {
                let ids: Vec<u32> = decal_elements(k, slot, 0).into_iter().map(|e| e.0).collect();
                for state in 1..DECAL_STATES {
                    let again: Vec<u32> = decal_elements(k, slot, state).into_iter().map(|e| e.0).collect();
                    assert_eq!(ids, again, "a decal change must replace the same elements");
                }
                assert!(ids.iter().all(|id| decal_ids(k).contains(id)));
            }
        }
    }
}
