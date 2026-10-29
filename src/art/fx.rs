//! Treats lying about the yard, and the little effects that pop up over
//! everyone's heads: emote bubbles, feathers, dust, sparkles.

use super::SheetBuilder;
use super::paint::{self, Cloud, Painter, rot};
use crate::gfx::sprites::SheetSpec;
use crate::gfx::vector::{Affine, Color, PathData};
use tiny_skia::Pixmap;

const INK: Color = Color::hex(0x3d3530);

fn worm(p: &mut Painter, phase: f32) {
    paint::shadow(p, 11.0, 2.5);
    let pts: Vec<(f32, f32)> = (0..7)
        .map(|i| {
            let x = -12.0 + i as f32 * 4.0;
            (x, -4.0 + ((i as f32 * 0.9 + phase) * 1.0).sin() * 2.6)
        })
        .collect();
    let body = PathData::smooth_open(&pts, 1.0);
    p.stroke(&body, Color::hex(0x9c4a5c), 8.4);
    p.stroke(&body, Color::hex(0xf09cb0), 6.0);
    let mut rings = PathData::new();
    for &(x, y) in pts.iter().skip(1).step_by(2) {
        rings.move_to(x, y - 2.5).line_to(x, y + 2.5);
    }
    p.stroke(&rings, Color::hex(0xc86d85), 1.0);
    let (hx, hy) = pts[6];
    p.fill(&PathData::circle(hx + 0.5, hy - 1.0, 1.0), INK);
}

fn snail(p: &mut Painter, up: bool) {
    paint::shadow(p, 12.0, 2.8);
    let foot = PathData::smooth_closed(&[(-13.0, -2.0), (4.0, -4.0), (13.0, -6.0), (14.0, -2.0), (0.0, 0.0)], 1.0);
    p.inked(&foot, Color::hex(0xc9d6b0), Color::hex(0x6d7a55), 0.9);
    let stalk = if up { -14.0 } else { -11.0 };
    let mut st = PathData::new();
    st.move_to(11.0, -5.0).line_to(12.0, stalk).move_to(13.5, -5.0).line_to(16.0, stalk + 1.0);
    p.stroke(&st, Color::hex(0x6d7a55), 1.4);
    p.fill(&PathData::circle(12.0, stalk, 1.4), INK);
    p.fill(&PathData::circle(16.0, stalk + 1.0, 1.4), INK);
    let shell = PathData::circle(-2.0, -10.0, 8.5);
    p.inked(&shell, Color::hex(0xe0a463), Color::hex(0x7d4f26), 1.0);
    let mut spiral = PathData::new();
    let mut first = true;
    for i in 0..30 {
        let a = i as f32 * 0.42;
        let r = 7.0 - i as f32 * 0.22;
        let (x, y) = (-2.0 + a.cos() * r, -10.0 + a.sin() * r);
        if first {
            spiral.move_to(x, y);
            first = false;
        } else {
            spiral.line_to(x, y);
        }
    }
    p.stroke(&spiral, Color::hex(0x9e6633), 1.3);
}

fn corn(p: &mut Painter) {
    paint::shadow(p, 10.0, 2.8);
    p.push(rot(-25.0));
    let cob = PathData::ellipse(0.0, -12.0, 6.5, 12.0);
    p.inked(&cob, Color::hex(0xffd34d), Color::hex(0x9a6d12), 1.0);
    for row in 0..6 {
        for col in 0..3 {
            let x = -3.5 + col as f32 * 3.5;
            let y = -20.5 + row as f32 * 3.4;
            p.fill(&PathData::circle(x, y, 1.3), Color::hex(0xffe98a));
        }
    }
    for s in [-1.0f32, 1.0] {
        let husk = PathData::smooth_closed(&[(0.0, 0.0), (s * 9.0, -8.0), (s * 5.0, -16.0), (s * 2.0, -6.0)], 1.0);
        p.inked(&husk, Color::hex(0x7fbf4d), Color::hex(0x3f6b25), 0.9);
    }
    p.pop();
}

fn golden_egg(p: &mut Painter, glint: bool) {
    paint::shadow(p, 9.0, 2.6);
    let egg = PathData::smooth_closed(&[(0.0, -21.0), (6.5, -15.0), (8.0, -7.0), (5.0, -1.0), (0.0, 0.5), (-5.0, -1.0), (-8.0, -7.0), (-6.5, -15.0)], 1.0);
    p.stroke(&egg, Color::hex(0x8a5a08), 2.0);
    p.radial(&egg, -2.5, -14.0, 16.0, &[(0.0, Color::hex(0xfff3a8)), (0.45, Color::hex(0xffd23a)), (1.0, Color::hex(0xd9940f))]);
    p.fill(&PathData::ellipse(-3.0, -14.0, 1.8, 3.6), Color::rgba(255, 255, 255, 210));
    if glint {
        sparkle_shape(p, 7.0, -18.0, 5.0, Color::rgb(255, 255, 255));
    }
}

fn butterfly(p: &mut Painter, open: f32) {
    p.fill(&PathData::ellipse(0.0, 1.0, 5.0, 1.4), Color::rgba(40, 50, 20, 50));
    p.push(Affine::translate(0.0, -16.0));
    for s in [-1.0f32, 1.0] {
        let w = |x: f32| s * x * open;
        let upper = PathData::smooth_closed(&[(0.0, -1.0), (w(9.0), -9.0), (w(11.0), -2.0), (w(3.0), 1.0)], 1.0);
        let lower = PathData::smooth_closed(&[(0.0, 0.5), (w(8.0), 3.0), (w(6.0), 8.0), (w(1.5), 4.0)], 1.0);
        p.inked(&upper, Color::hex(0xffb0d6), Color::hex(0x8a3b67), 0.8);
        p.inked(&lower, Color::hex(0xb7d9ff), Color::hex(0x3b5f8a), 0.8);
        p.fill(&PathData::circle(w(6.0), -4.0, 1.4), Color::hex(0xfff4a8));
    }
    p.fill(&PathData::ellipse(0.0, 0.0, 1.3, 5.0), INK);
    let mut ant = PathData::new();
    ant.move_to(0.0, -4.5).quad_to(-2.0, -8.0, -3.5, -8.5).move_to(0.0, -4.5).quad_to(2.0, -8.0, 3.5, -8.5);
    p.stroke(&ant, INK, 0.8);
    p.pop();
}

fn beetle(p: &mut Painter, k: u32) {
    paint::shadow(p, 8.0, 2.2);
    let mut legs = PathData::new();
    for i in 0..3 {
        let y = -8.0 + i as f32 * 3.5;
        let kick = if (i + k) % 2 == 0 { 1.5 } else { -1.5 };
        legs.move_to(-5.0, y).line_to(-9.0, y + kick).move_to(5.0, y).line_to(9.0, y - kick);
    }
    p.stroke(&legs, INK, 1.1);
    p.fill(&PathData::circle(0.0, -13.0, 3.5), INK);
    let shell = PathData::ellipse(0.0, -6.0, 7.0, 7.5);
    p.inked(&shell, Color::hex(0xe23b2e), INK, 0.9);
    let mut split = PathData::new();
    split.move_to(0.0, -13.0).line_to(0.0, 1.0);
    p.stroke(&split, INK, 1.0);
    for (x, y) in [(-3.5f32, -8.0f32), (3.5, -8.0), (-3.0, -3.0), (3.0, -3.0)] {
        p.fill(&PathData::circle(x, y, 1.5), INK);
    }
    p.fill(&PathData::ellipse(-2.5, -10.0, 1.4, 2.0), Color::rgba(255, 255, 255, 150));
}

/// A golden feather: fills the flight meter.
fn gold_feather(p: &mut Painter, glint: bool) {
    paint::shadow(p, 9.0, 2.4);
    p.push(Affine::translate(0.0, -14.0).then(rot(-35.0)));
    let vane = PathData::smooth_closed(&[(-14.0, 2.0), (-4.0, -6.0), (14.0, -3.0), (6.0, 4.5), (-8.0, 5.0)], 1.0);
    p.stroke(&vane, Color::hex(0x8a5a08), 2.0);
    p.radial(&vane, -2.0, -2.0, 16.0, &[(0.0, Color::hex(0xfff3a8)), (0.5, Color::hex(0xffd23a)), (1.0, Color::hex(0xd9940f))]);
    let mut shaft = PathData::new();
    shaft.move_to(-17.0, 4.0).quad_to(0.0, 0.0, 14.0, -3.0);
    p.stroke(&shaft, Color::hex(0xb07810), 1.4);
    let mut barbs = PathData::new();
    for k in 0..4 {
        let x = -8.0 + k as f32 * 5.0;
        barbs.move_to(x, 0.5 - k as f32 * 0.5).line_to(x + 3.0, -4.5);
    }
    p.stroke(&barbs, Color::hex(0xe6a91a), 0.9);
    p.pop();
    if glint {
        sparkle_shape(p, 8.0, -24.0, 5.0, Color::rgb(255, 255, 255));
    }
}

/// A shiny green beetle with a pair of jaws, legs a-scuttle.
fn stag_beetle(p: &mut Painter, k: u32) {
    paint::shadow(p, 9.0, 2.4);
    let mut legs = PathData::new();
    for i in 0..3 {
        let x = -6.0 + i as f32 * 6.0;
        let kick = if (i + k) % 2 == 0 { 2.0 } else { -2.0 };
        legs.move_to(x, -6.0).line_to(x + kick, 0.0);
    }
    p.stroke(&legs, INK, 1.2);
    let body = PathData::ellipse(-1.0, -8.0, 10.0, 6.0);
    p.inked_radial(&body, INK, 0.9, (-4.0, -11.0, 12.0), &[(0.0, Color::hex(0x9ccc65)), (0.6, Color::hex(0x2e7d32)), (1.0, Color::hex(0x1b5e20))]);
    let mut split = PathData::new();
    split.move_to(-10.0, -8.0).line_to(6.0, -8.0);
    p.stroke(&split, INK, 0.8);
    p.fill(&PathData::circle(10.0, -9.0, 3.6), INK);
    let mut jaws = PathData::new();
    jaws.move_to(12.0, -11.0).quad_to(17.0, -14.0, 17.0, -9.0).move_to(12.0, -7.0).quad_to(17.0, -4.0, 17.0, -9.0 + if k == 0 { 2.0 } else { 0.5 });
    p.stroke(&jaws, Color::hex(0x5d4037), 1.6);
    p.fill(&PathData::ellipse(-4.0, -11.0, 3.0, 1.4), Color::rgba(255, 255, 255, 170));
}

fn caterpillar(p: &mut Painter, k: u32) {
    paint::shadow(p, 12.0, 2.4);
    // An inchworm hump that travels along.
    for i in 0..6 {
        let x = -12.0 + i as f32 * 4.6;
        let hump = if k == 0 { [0.0f32, 2.0, 5.0, 2.0, 0.0, 0.0][i] } else { [0.0f32, 0.0, 2.0, 5.0, 2.0, 0.0][i] };
        let c = if i % 2 == 0 { Color::hex(0x8bc34a) } else { Color::hex(0xcddc39) };
        p.inked(&PathData::circle(x, -5.0 - hump, 4.0), c, Color::hex(0x33691e), 0.7);
    }
    p.inked(&PathData::circle(14.0, -7.0, 4.8), Color::hex(0x7cb342), Color::hex(0x33691e), 0.8);
    p.fill(&PathData::circle(16.0, -8.0, 1.1), INK);
    let mut ant = PathData::new();
    ant.move_to(13.0, -11.0).line_to(12.0, -15.0).move_to(16.0, -11.0).line_to(18.0, -15.0);
    p.stroke(&ant, INK, 0.9);
}

fn grasshopper(p: &mut Painter, leaping: bool) {
    if !leaping {
        paint::shadow(p, 10.0, 2.2);
    }
    let lift = if leaping { -10.0 } else { 0.0 };
    p.push(Affine::translate(0.0, lift));
    let green = Color::hex(0x7cb342);
    let dark = Color::hex(0x33691e);
    // The big back leg: folded when sitting, flung out when leaping.
    let mut leg = PathData::new();
    if leaping {
        leg.move_to(-2.0, -8.0).line_to(-12.0, -2.0).line_to(-20.0, 4.0);
    } else {
        leg.move_to(-2.0, -8.0).line_to(-8.0, -16.0).line_to(-12.0, -1.0);
    }
    p.stroke(&leg, dark, 3.4);
    p.stroke(&leg, green, 2.0);
    let body = PathData::smooth_closed(&[(-12.0, -6.0), (0.0, -12.0), (12.0, -10.0), (14.0, -6.0), (0.0, -3.0)], 0.6);
    p.inked(&body, green, dark, 0.8);
    let wing = PathData::smooth_closed(&[(-10.0, -8.0), (2.0, -12.0), (6.0, -9.0)], 0.6);
    p.fill(&wing, Color::hex(0xaed581));
    let mut front = PathData::new();
    front.move_to(8.0, -6.0).line_to(10.0, 0.0).move_to(4.0, -5.0).line_to(5.0, 0.0);
    p.stroke(&front, dark, 1.1);
    p.fill(&PathData::circle(12.0, -9.0, 1.4), INK);
    let mut ant = PathData::new();
    ant.move_to(13.0, -11.0).quad_to(16.0, -20.0, 22.0, -22.0);
    p.stroke(&ant, dark, 0.8);
    p.pop();
}

fn dragonfly(p: &mut Painter, k: u32) {
    p.fill(&PathData::ellipse(0.0, 14.0, 10.0, 2.0), Color::rgba(20, 60, 100, 50));
    let body_c = Color::hex(0x1e88e5);
    // Two pairs of glassy wings, a blur on alternate frames.
    let spread = if k == 0 { 1.0 } else { 0.55 };
    for (dx, sy) in [(-3.0f32, -1.0f32), (3.0, -1.0), (-3.0, 1.0), (3.0, 1.0)] {
        let wing = PathData::ellipse(dx * 1.5, -10.0 + sy * 6.0 * spread, 9.0, 3.0 * spread + 0.6);
        p.fill(&wing, Color::rgba(200, 235, 255, 160));
        p.stroke(&wing, Color::rgba(70, 120, 160, 160), 0.6);
    }
    let mut tail = PathData::new();
    tail.move_to(-2.0, -10.0).line_to(-20.0, -9.0);
    p.stroke(&tail, INK, 3.4);
    p.stroke(&tail, body_c, 2.2);
    p.inked(&PathData::ellipse(2.0, -10.0, 4.0, 2.6), body_c, INK, 0.6);
    p.fill(&PathData::circle(7.0, -10.0, 2.6), Color::hex(0x43a047));
    p.fill(&PathData::circle(7.8, -10.6, 0.8), Color::rgb(255, 255, 255));
}

pub fn items_sheet(tile_px: u32) -> (SheetSpec, Pixmap) {
    let mut sb = SheetBuilder::new("items", (48.0, 48.0), (24.0, 40.0), tile_px);
    for phase in [0.0f32, 1.6] {
        sb.frame("worm", |p| worm(p, phase));
    }
    for up in [true, false] {
        sb.frame("snail", |p| snail(p, up));
    }
    sb.frame("corn", corn);
    for glint in [false, true] {
        sb.frame("feather", |p| gold_feather(p, glint));
    }
    for glint in [false, true] {
        sb.frame("golden_egg", |p| golden_egg(p, glint));
    }
    for open in [1.0f32, 0.35] {
        sb.frame("butterfly", |p| butterfly(p, open));
    }
    for k in 0..2 {
        sb.frame("ladybird", |p| beetle(p, k));
    }
    for k in 0..2 {
        sb.frame("beetle", |p| stag_beetle(p, k));
    }
    for k in 0..2 {
        sb.frame("caterpillar", |p| caterpillar(p, k));
    }
    sb.frame("grasshopper", |p| grasshopper(p, false));
    sb.frame("grasshopper_leap", |p| grasshopper(p, true));
    for k in 0..2 {
        sb.frame("dragonfly", |p| dragonfly(p, k));
    }
    sb.finish()
}

/// The pointer while playing: a little white feather, its quill at the
/// hotspot. Returns the image and the hotspot in pixels.
pub fn feather_cursor(px: u32) -> (Pixmap, (u32, u32)) {
    let s = px as f32 / 32.0;
    let mut p = Painter::new(px, px, s, 0.0, 0.0);
    let vane = PathData::smooth_closed(&[(7.0, 24.0), (12.0, 12.0), (24.0, 3.0), (29.0, 5.0), (22.0, 17.0), (11.0, 25.0)], 1.0);
    p.stroke(&vane, Color::rgb(40, 40, 40), 3.0);
    p.fill(&vane, Color::hex(0xfffdf7));
    let mut shaft = PathData::new();
    shaft.move_to(3.0, 29.0).quad_to(14.0, 17.0, 28.0, 4.0);
    p.stroke(&shaft, Color::hex(0x9b918a), 1.6);
    (p.pm, ((3.0 * s) as u32, (29.0 * s) as u32))
}

fn bubble(p: &mut Painter) {
    let mut d = PathData::round_rect(-13.0, -15.0, 26.0, 22.0, 9.0);
    d.extend(&PathData::polygon(&[(-8.0, 5.0), (-12.0, 13.0), (-1.0, 6.0)]));
    p.stroke(&d, INK, 3.0);
    p.fill(&d, Color::rgb(255, 255, 255));
}

fn bang(p: &mut Painter, x: f32, c: Color, s: f32) {
    p.fill(&PathData::round_rect(x - 2.2 * s, -12.0 * s - 3.0, 4.4 * s, 10.0 * s, 2.0 * s), c);
    p.fill(&PathData::circle(x, 2.0 * s - 2.5, 2.3 * s), c);
}

fn heart_path(cx: f32, cy: f32, s: f32) -> PathData {
    let mut h = PathData::new();
    h.move_to(cx, cy + 7.0 * s)
        .cubic_to(cx - 11.0 * s, cy - 1.0 * s, cx - 7.0 * s, cy - 10.0 * s, cx, cy - 5.0 * s)
        .cubic_to(cx + 7.0 * s, cy - 10.0 * s, cx + 11.0 * s, cy - 1.0 * s, cx, cy + 7.0 * s)
        .close();
    h
}

fn star_path(cx: f32, cy: f32, points: usize, outer: f32, inner: f32, phase: f32) -> PathData {
    let pts: Vec<(f32, f32)> = (0..points * 2)
        .map(|i| {
            let a = phase + i as f32 * std::f32::consts::PI / points as f32;
            let r = if i % 2 == 0 { outer } else { inner };
            (cx + a.sin() * r, cy - a.cos() * r)
        })
        .collect();
    PathData::polygon(&pts)
}

fn sparkle_shape(p: &mut Painter, x: f32, y: f32, r: f32, c: Color) {
    let s = star_path(x, y, 4, r, r * 0.28, 0.0);
    p.fill(&s, c);
}

pub fn fx_sheet(tile_px: u32) -> (SheetSpec, Pixmap) {
    let mut sb = SheetBuilder::new("fx", (40.0, 40.0), (20.0, 20.0), tile_px);
    sb.frame("emote_alert", |p| {
        bubble(p);
        bang(p, 0.0, Color::hex(0xe53935), 1.0);
    });
    sb.frame("emote_puzzled", |p| {
        bubble(p);
        let mut q = PathData::new();
        q.move_to(-4.5, -8.0).quad_to(-4.5, -12.0, 0.0, -12.0).quad_to(4.5, -12.0, 4.5, -8.0).quad_to(4.5, -5.0, 0.0, -3.5).line_to(0.0, -1.5);
        p.stroke(&q, Color::hex(0x3b6fd6), 3.2);
        p.fill(&PathData::circle(0.0, 2.5, 2.0), Color::hex(0x3b6fd6));
    });
    sb.frame("emote_heart", |p| {
        bubble(p);
        p.fill(&heart_path(0.0, -4.0, 0.9), Color::hex(0xff4f86));
    });
    sb.frame("emote_fright", |p| {
        bubble(p);
        bang(p, -4.0, Color::hex(0xff8f1f), 0.85);
        bang(p, 4.0, Color::hex(0xff8f1f), 0.85);
    });
    sb.frame("emote_note", |p| {
        bubble(p);
        let c = Color::hex(0x9b59d0);
        p.push(Affine::translate(-1.5, -4.0));
        p.fill(&PathData::ellipse(-1.5, 4.0, 3.6, 2.7), c);
        let mut stem = PathData::new();
        stem.move_to(1.8, 3.5).line_to(1.8, -8.0).quad_to(5.5, -6.0, 6.5, -2.5);
        p.stroke(&stem, c, 1.8);
        p.pop();
    });
    sb.frame("emote_sweat", |p| {
        let mut d = PathData::new();
        d.move_to(0.0, -10.0).quad_to(7.0, -1.0, 0.0, 5.0).quad_to(-7.0, -1.0, 0.0, -10.0).close();
        p.inked(&d, Color::hex(0x8fd0ff), Color::hex(0x2a6d9c), 1.0);
        p.fill(&PathData::ellipse(-2.0, -1.0, 1.2, 2.2), Color::rgba(255, 255, 255, 220));
    });
    sb.frame("emote_angry", |p| {
        let c = Color::hex(0xe53935);
        let mut v = PathData::new();
        for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            v.move_to(sx * 2.0, sy * 9.0).quad_to(sx * 2.5, sy * 2.5, sx * 9.0, sy * 2.0);
        }
        p.stroke(&v, Color::rgb(255, 255, 255), 5.5);
        p.stroke(&v, c, 3.0);
    });
    for (i, a) in [-30.0f32, 15.0, 70.0].into_iter().enumerate() {
        sb.frame("feather", |p| {
            p.push(rot(a));
            let vane = PathData::smooth_closed(&[(-10.0, 2.0), (-2.0, -4.5), (10.0, -3.0), (4.0, 3.5), (-6.0, 4.5)], 1.0);
            p.inked(&vane, Color::hex(0xfffdf7), Color::hex(0x9b918a), 0.8);
            let mut shaft = PathData::new();
            shaft.move_to(-12.0, 4.0).quad_to(0.0, 0.0, 10.0, -3.0);
            p.stroke(&shaft, Color::hex(0xbcb2a6), 1.0);
            p.pop();
            let _ = i;
        });
    }
    for r in [5.0f32, 8.0, 11.0] {
        sb.frame("dust", |p| {
            let c = Cloud::new(0.0, 0.0, r, r * 0.75, 6, 0.42, (r * 10.0) as u64);
            p.fill(&c.path(0.0), Color::rgba(232, 214, 176, 210));
        });
    }
    for r in [10.0f32, 6.0] {
        sb.frame("sparkle", |p| {
            sparkle_shape(p, 0.0, 0.0, r, Color::hex(0xfff7c2));
            sparkle_shape(p, 0.0, 0.0, r * 0.55, Color::rgb(255, 255, 255));
        });
    }
    sb.frame("star", |p| {
        let s = star_path(0.0, 0.0, 5, 9.0, 4.0, 0.0);
        p.inked(&s, Color::hex(0xffd23f), Color::hex(0x9c6d00), 1.0);
        // A white-hot glint at the heart of it (the part that glows in HDR).
        sparkle_shape(p, 0.0, 0.5, 3.4, Color::rgb(255, 255, 255));
    });
    // A splash: a crown of water, spreading.
    for k in 0..3 {
        sb.frame("splash", |p| {
            let r = 6.0 + k as f32 * 4.5;
            let a = 255 - k as u8 * 60;
            p.stroke(&PathData::ellipse(0.0, 6.0, r * 1.4, r * 0.45), Color::rgba(235, 248, 255, a), 2.2);
            for i in 0..5 {
                let x = -12.0 + i as f32 * 6.0;
                let h = [8.0f32, 13.0, 16.0, 13.0, 8.0][i] * (1.0 - k as f32 * 0.25);
                let mut d = PathData::new();
                d.move_to(x * (1.0 + k as f32 * 0.3), 6.0).quad_to(x * 1.2, 6.0 - h, x * (1.3 + k as f32 * 0.3), 4.0 - h);
                p.stroke(&d, Color::hex(0x2a6d9c).with_alpha(a), 3.2);
                p.stroke(&d, Color::hex(0x8fd0ff).with_alpha(a), 2.0);
            }
            for (x, y) in [(-16.0f32, -8.0f32), (15.0, -10.0), (2.0, -14.0)] {
                p.fill(&PathData::circle(x * (1.0 + k as f32 * 0.4), y + k as f32 * 3.0, 2.0), Color::hex(0x8fd0ff).with_alpha(a));
            }
        });
    }
    // A bug that met a tyre: a cartoon splat, with a little halo.
    for k in 0..2 {
        sb.frame("splat", |p| {
            let mut d = PathData::new();
            let n = 9;
            for i in 0..n * 2 {
                let a = i as f32 * std::f32::consts::PI / n as f32;
                let r = if i % 2 == 0 { 9.0 } else { 5.0 } * (1.0 + k as f32 * 0.15);
                let (x, y) = (a.cos() * r, 4.0 + a.sin() * r * 0.45);
                if i == 0 {
                    d.move_to(x, y);
                } else {
                    d.line_to(x, y);
                }
            }
            d.close();
            p.inked(&d, Color::hex(0xc0ca33), Color::hex(0x827717), 0.8);
            p.stroke(&PathData::ellipse(0.0, -6.0 - k as f32 * 3.0, 6.0, 2.0), Color::hex(0xffe082), 1.4);
        });
    }
    // A puff of exhaust or dust, for a hen who has had a near miss.
    for k in 0..2 {
        sb.frame("puff", |p| {
            let c = Cloud::new(0.0, 0.0, 9.0 + k as f32 * 3.0, 6.0 + k as f32 * 2.0, 6, 0.42, 77 + k as u64);
            p.fill(&c.path(0.0), Color::rgba(250, 250, 250, 200 - k as u8 * 70));
        });
    }
    sb.frame("reject", |p| {
        let mut x = PathData::new();
        x.move_to(-7.0, -7.0).line_to(7.0, 7.0).move_to(7.0, -7.0).line_to(-7.0, 7.0);
        p.stroke(&x, Color::rgb(255, 255, 255), 7.0);
        p.stroke(&x, Color::hex(0xe53935), 4.0);
    });
    sb.frame("check", |p| {
        let mut c = PathData::new();
        c.move_to(-7.0, 0.0).line_to(-2.0, 6.0).line_to(8.0, -7.0);
        p.stroke(&c, Color::rgb(255, 255, 255), 7.0);
        p.stroke(&c, Color::hex(0x43a047), 4.0);
    });
    sb.frame("heart", |p| {
        let h = heart_path(0.0, 0.0, 1.0);
        p.inked(&h, Color::hex(0xff4f86), Color::rgb(255, 255, 255), 1.4);
        // A glint on the left lobe.
        p.fill(&PathData::ellipse(-3.6, -4.2, 1.5, 1.1), Color::rgb(255, 255, 255));
    });
    sb.finish()
}
