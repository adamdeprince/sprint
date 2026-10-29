//! Things to ride across the river: logs in segments (any length), turtles
//! (some of which dive), a giant rubber duck, an inflatable flamingo, a
//! drifting picnic table, lily pads, a swan pedalo, a raft captained by a
//! scarecrow, a canoe full of ducklings and a striped rubber ring.
//!
//! Each is drawn with its waterline on y = 0 and its middle at x = 0. Logs
//! and lily pads are one tile per frame, butting together at x = ±32.

use super::SheetBuilder;
use super::paint::{Painter, mirror};
use crate::gfx::sprites::SheetSpec;
use crate::gfx::vector::{Affine, Color, PathData};
use tiny_skia::Pixmap;

const INK: Color = Color::hex(0x2c2a2e);
const FOAM: Color = Color::rgba(235, 248, 255, 210);
const WATER_SHADE: Color = Color::rgba(20, 60, 100, 70);

fn wake(p: &mut Painter, half: f32, k: u32) {
    p.fill(&PathData::ellipse(0.0, 2.0, half + 2.0, 6.0), WATER_SHADE);
    let mut d = PathData::new();
    let s = if k == 0 { 0.0 } else { 3.0 };
    for side in [-1.0f32, 1.0] {
        let x = side * (half - 2.0 + s);
        d.move_to(x - 6.0, 3.0).quad_to(x, -1.0, x + 6.0, 3.0);
    }
    p.stroke(&d, FOAM, 2.0);
}

const BARK: Color = Color::hex(0x8d6748);
const BARK_DARK: Color = Color::hex(0x5e4330);
const BARK_LIGHT: Color = Color::hex(0xb08660);
const RING: Color = Color::hex(0xe2c497);

/// A log segment: 0 the left end, 1 a middle, 2 the right end.
fn log(p: &mut Painter, part: u32) {
    // Segments overlap a little so no water shows between them however
    // the pixels round.
    let (x0, x1) = match part {
        0 => (-26.0, 35.0),
        2 => (-35.0, 26.0),
        _ => (-35.0, 35.0),
    };
    p.fill(&PathData::rect(x0, -2.0, x1 - x0, 8.0), WATER_SHADE);
    let top = -34.0;
    let bot = 2.0;
    p.fill(&PathData::rect(x0, top, x1 - x0, bot - top), BARK);
    p.fill(&PathData::rect(x0, top + 4.0, x1 - x0, 7.0), BARK_LIGHT);
    p.fill(&PathData::rect(x0, bot - 9.0, x1 - x0, 9.0), BARK_DARK);
    let mut edge = PathData::new();
    edge.move_to(x0, top).line_to(x1, top).move_to(x0, bot).line_to(x1, bot);
    p.stroke(&edge, INK, 2.6);
    // Bark furrows, a knot here and there.
    let mut furrows = PathData::new();
    let seed = part as f32 * 7.0;
    for i in 0..4 {
        let y = top + 8.0 + i as f32 * 6.5;
        let x = x0 + 6.0 + ((i as f32 * 13.0 + seed) % 24.0);
        furrows.move_to(x, y).quad_to(x + 9.0, y - 1.5, x + 18.0, y);
    }
    p.stroke(&furrows, BARK_DARK, 1.6);
    if part == 1 {
        p.inked(&PathData::ellipse(10.0, -18.0, 4.0, 3.0), BARK_DARK, INK, 0.6);
    }
    // Cut ends show their rings.
    let end = |p: &mut Painter, x: f32| {
        let e = PathData::ellipse(x, (top + bot) / 2.0, 10.0, (bot - top) / 2.0);
        p.inked(&e, RING, INK, 1.3);
        for r in [0.65f32, 0.35] {
            p.stroke(&PathData::ellipse(x, (top + bot) / 2.0, 10.0 * r, (bot - top) / 2.0 * r), BARK_LIGHT, 1.0);
        }
    };
    match part {
        0 => end(p, x0),
        2 => end(p, x1),
        _ => {}
    }
    if part == 1 {
        // A mushroom, because logs have them.
        p.inked(&PathData::ellipse(-14.0, top - 2.0, 6.0, 3.6), Color::hex(0xe57373), INK, 0.8);
        p.fill(&PathData::circle(-15.0, top - 3.0, 1.2), Color::rgb(255, 255, 255));
    }
}

const SHELL: Color = Color::hex(0x6d8b3a);
const SHELL_DARK: Color = Color::hex(0x4b6527);
const TURTLE_SKIN: Color = Color::hex(0x9ccc65);

/// One turtle; `sink` from 0 (swimming) to 1 (gone under, bubbles only).
fn turtle(p: &mut Painter, k: u32, sink: f32) {
    if sink >= 1.0 {
        for (x, y, r) in [(-8.0f32, -6.0f32, 3.0f32), (4.0, -10.0, 2.2), (10.0, -4.0, 3.4)] {
            p.stroke(&PathData::circle(x, y - k as f32 * 3.0, r), FOAM, 1.2);
        }
        let mut ring = PathData::new();
        ring.move_to(-22.0, 2.0).quad_to(0.0, -4.0, 22.0, 2.0);
        p.stroke(&ring, FOAM, 2.0);
        return;
    }
    wake(p, 26.0, k);
    p.push(Affine::translate(0.0, sink * 22.0));
    let paddle = if k == 0 { 4.0 } else { -4.0 };
    for (x, a) in [(-20.0f32, paddle), (16.0, -paddle)] {
        p.inked(&PathData::ellipse(x, -4.0 + a * 0.3, 8.0, 4.0), TURTLE_SKIN, INK, 0.8);
    }
    // Head looking ahead (right; mirrored for left).
    p.inked(&PathData::ellipse(28.0, -16.0, 9.0, 7.0), TURTLE_SKIN, INK, 1.0);
    p.fill(&PathData::circle(31.0, -18.0, 1.8), INK);
    let mut smile = PathData::new();
    smile.move_to(30.0, -12.0).quad_to(33.0, -10.0, 36.0, -13.0);
    p.stroke(&smile, INK, 0.9);
    let shell = PathData::smooth_closed(&[(-28.0, -4.0), (-22.0, -24.0), (0.0, -32.0), (22.0, -24.0), (28.0, -4.0)], 0.6);
    p.inked_radial(&shell, INK, 1.3, (-6.0, -26.0, 34.0), &[(0.0, SHELL.shade(0.2)), (1.0, SHELL_DARK)]);
    let mut plates = PathData::new();
    plates.move_to(-14.0, -6.0).line_to(-10.0, -24.0).move_to(12.0, -6.0).line_to(10.0, -24.0).move_to(-24.0, -14.0).line_to(24.0, -14.0);
    p.stroke(&plates, SHELL_DARK, 1.4);
    p.pop();
    if sink > 0.0 {
        // Bubbles say it is about to go.
        for (x, y) in [(-30.0f32, -10.0f32), (32.0, -6.0), (0.0, -36.0 + sink * 22.0)] {
            p.stroke(&PathData::circle(x, y - k as f32 * 2.0, 2.6), FOAM, 1.2);
        }
    }
}

fn duck(p: &mut Painter, k: u32) {
    wake(p, 60.0, k);
    let yellow = Color::hex(0xffd84a);
    let yellow_dark = Color::hex(0xf2b51d);
    let tilt = if k == 0 { 0.0 } else { 2.0 };
    p.push(Affine::translate(0.0, tilt));
    let body = PathData::smooth_closed(&[(-62.0, -30.0), (-52.0, -10.0), (-10.0, 2.0), (40.0, 0.0), (58.0, -16.0), (40.0, -36.0), (-20.0, -40.0), (-56.0, -52.0)], 0.7);
    p.inked_radial(&body, INK, 1.6, (-6.0, -36.0, 70.0), &[(0.0, Color::hex(0xfff3a0)), (0.5, yellow), (1.0, yellow_dark)]);
    let wing = PathData::smooth_closed(&[(-30.0, -28.0), (8.0, -32.0), (16.0, -18.0), (-20.0, -12.0)], 0.8);
    p.inked(&wing, yellow_dark, INK, 1.0);
    let head = PathData::circle(30.0, -62.0, 22.0);
    p.inked_radial(&head, INK, 1.6, (22.0, -72.0, 30.0), &[(0.0, Color::hex(0xfff3a0)), (1.0, yellow)]);
    let beak = PathData::smooth_closed(&[(46.0, -64.0), (66.0, -60.0), (64.0, -52.0), (46.0, -52.0)], 0.6);
    p.inked(&beak, Color::hex(0xff8f00), INK, 1.2);
    p.fill(&PathData::circle(36.0, -68.0, 4.0), INK);
    p.fill(&PathData::circle(37.5, -69.5, 1.4), Color::rgb(255, 255, 255));
    p.fill(&PathData::ellipse(18.0, -74.0, 6.0, 3.0), Color::rgba(255, 255, 255, 170));
    p.pop();
}

fn flamingo(p: &mut Painter, k: u32) {
    wake(p, 50.0, k);
    let pink = Color::hex(0xf78fb3);
    let pink_dark = Color::hex(0xd96b93);
    let sway = if k == 0 { 0.0 } else { 3.0 };
    let body = PathData::smooth_closed(&[(-50.0, -10.0), (-40.0, -34.0), (10.0, -38.0), (44.0, -20.0), (40.0, 0.0), (-40.0, 2.0)], 0.6);
    p.inked_radial(&body, INK, 1.5, (-8.0, -30.0, 60.0), &[(0.0, Color::hex(0xffc1d6)), (1.0, pink)]);
    // Seams of the inflatable.
    let mut seam = PathData::new();
    seam.move_to(-44.0, -14.0).quad_to(0.0, -24.0, 40.0, -12.0);
    p.stroke(&seam, pink_dark, 1.2);
    let mut neck = PathData::new();
    neck.move_to(30.0, -26.0).cubic_to(54.0, -50.0, 20.0 + sway, -66.0, 34.0 + sway, -86.0);
    p.stroke(&neck, INK, 15.0);
    p.stroke(&neck, pink, 12.0);
    let head = PathData::circle(38.0 + sway, -90.0, 10.0);
    p.inked(&head, pink, INK, 1.4);
    let beak = PathData::smooth_closed(&[(44.0 + sway, -92.0), (58.0 + sway, -88.0), (52.0 + sway, -78.0), (46.0 + sway, -84.0)], 0.6);
    p.inked(&beak, Color::hex(0xf5f5f5), INK, 1.0);
    p.fill(&PathData::polygon(&[(52.0 + sway, -82.0), (58.0 + sway, -88.0), (54.0 + sway, -78.0)]), INK);
    p.fill(&PathData::circle(40.0 + sway, -94.0, 2.0), INK);
    p.fill(&PathData::ellipse(-20.0, -30.0, 10.0, 4.0), Color::rgba(255, 255, 255, 160));
    // The valve.
    p.fill(&PathData::circle(-40.0, -14.0, 2.6), Color::hex(0xffffff));
}

fn picnic(p: &mut Painter, k: u32) {
    wake(p, 62.0, k);
    let wood = Color::hex(0xc48a51);
    let wood_dark = Color::hex(0x8c5d33);
    let bob = if k == 0 { 0.0 } else { 1.5 };
    p.push(Affine::translate(0.0, bob));
    // Benches, legs and the top; the rest is under water.
    for x in [-60.0f32, 40.0] {
        p.inked(&PathData::rect(x, -18.0, 20.0, 8.0), wood, INK, 1.0);
    }
    let mut legs = PathData::new();
    legs.move_to(-44.0, -36.0).line_to(-54.0, -2.0).move_to(44.0, -36.0).line_to(54.0, -2.0);
    p.stroke(&legs, INK, 7.0);
    p.stroke(&legs, wood_dark, 5.0);
    let top = PathData::rect(-62.0, -44.0, 124.0, 10.0);
    p.inked(&top, wood, INK, 1.3);
    // A gingham cloth and an abandoned pie.
    let cloth = PathData::polygon(&[(-40.0, -44.0), (30.0, -44.0), (36.0, -30.0), (-34.0, -30.0)]);
    p.inked(&cloth, Color::hex(0xfdfdfd), INK, 0.8);
    let mut check = PathData::new();
    for i in 0..5 {
        let x = -34.0 + i as f32 * 15.0;
        check.move_to(x, -44.0).line_to(x + 3.0, -30.0);
    }
    check.move_to(-38.0, -37.0).line_to(33.0, -37.0);
    p.stroke(&check, Color::hex(0xe53935).with_alpha(200), 3.0);
    let pie = PathData::smooth_closed(&[(-16.0, -46.0), (-14.0, -56.0), (12.0, -56.0), (14.0, -46.0)], 0.4);
    p.inked(&pie, Color::hex(0xe0a85a), INK, 1.0);
    let mut lattice = PathData::new();
    lattice.move_to(-10.0, -55.0).line_to(-4.0, -47.0).move_to(0.0, -55.0).line_to(6.0, -47.0).move_to(-12.0, -50.0).line_to(10.0, -50.0);
    p.stroke(&lattice, Color::hex(0xa86b2d), 1.2);
    p.pop();
}

fn lily(p: &mut Painter, flower: bool, k: u32) {
    p.fill(&PathData::ellipse(0.0, 2.0, 30.0, 8.0), WATER_SHADE);
    let mut pad = PathData::new();
    let n = 18;
    pad.move_to(0.0, -10.0);
    for i in 1..n {
        let a = 0.3 + i as f32 / n as f32 * (std::f32::consts::TAU - 0.6) - std::f32::consts::FRAC_PI_2;
        pad.line_to(a.cos() * 29.0, -10.0 + a.sin() * 12.0);
    }
    pad.close();
    p.inked(&pad, Color::hex(0x6aad4a), INK.with_alpha(160), 1.0);
    let mut veins = PathData::new();
    for a in [0.6f32, 1.6, 2.6, 3.6, 4.6] {
        veins.move_to(0.0, -10.0).line_to(a.cos() * 22.0, -10.0 + a.sin() * 9.0);
    }
    p.stroke(&veins, Color::hex(0x8ccc66), 1.2);
    if flower {
        let s = if k == 0 { 1.0 } else { 1.06 };
        for i in 0..6 {
            let a = i as f32 * std::f32::consts::TAU / 6.0;
            let petal = PathData::ellipse(10.0 + a.cos() * 6.0 * s, -18.0 + a.sin() * 3.0 * s, 4.0, 3.0);
            p.inked(&petal, Color::hex(0xf8bbd0), INK.with_alpha(120), 0.6);
        }
        p.fill(&PathData::circle(10.0, -19.0, 2.6), Color::hex(0xffeb3b));
    }
}

fn pedalo(p: &mut Painter, k: u32) {
    wake(p, 58.0, k);
    let white = Color::hex(0xfbfbf8);
    let shade = Color::hex(0xdcdad3);
    let bob = if k == 0 { 0.0 } else { 1.5 };
    p.push(Affine::translate(0.0, bob));
    // The paddle wheel churning at the back.
    let mut paddles = PathData::new();
    for i in 0..4 {
        let a = k as f32 * 0.4 + i as f32 * std::f32::consts::FRAC_PI_2;
        paddles.move_to(-48.0, -14.0).line_to(-48.0 + a.cos() * 14.0, -14.0 + a.sin() * 14.0);
    }
    p.stroke(&paddles, INK, 4.0);
    p.stroke(&paddles, Color::hex(0x4fc3f7), 2.4);
    let hull = PathData::smooth_closed(&[(-56.0, -30.0), (-40.0, -4.0), (30.0, 0.0), (52.0, -16.0), (40.0, -34.0), (-30.0, -38.0)], 0.6);
    p.inked_radial(&hull, INK, 1.5, (-10.0, -30.0, 60.0), &[(0.0, white), (1.0, shade)]);
    let wing = PathData::smooth_closed(&[(-34.0, -30.0), (4.0, -42.0), (14.0, -26.0), (-24.0, -18.0)], 0.7);
    p.inked(&wing, white, INK, 1.0);
    // The long neck and orange beak.
    let mut neck = PathData::new();
    neck.move_to(34.0, -26.0).cubic_to(56.0, -50.0, 30.0, -70.0, 42.0, -86.0);
    p.stroke(&neck, INK, 15.0);
    p.stroke(&neck, white, 12.0);
    p.inked(&PathData::circle(46.0, -88.0, 10.0), white, INK, 1.3);
    p.inked(&PathData::polygon(&[(54.0, -92.0), (68.0, -86.0), (54.0, -82.0)]), Color::hex(0xff8f00), INK, 1.0);
    p.fill(&PathData::polygon(&[(50.0, -94.0), (56.0, -92.0), (54.0, -82.0), (48.0, -86.0)]), INK);
    p.fill(&PathData::circle(46.0, -92.0, 1.8), Color::rgb(255, 255, 255));
    p.pop();
}

fn raft(p: &mut Painter, k: u32) {
    wake(p, 60.0, k);
    let wood = Color::hex(0xb98552);
    let rope = Color::hex(0xe0c48a);
    // Planks lashed together.
    for i in 0..6 {
        let x = -60.0 + i as f32 * 20.0;
        p.inked(&PathData::round_rect(x, -16.0, 20.0, 16.0, 5.0), wood.shade(if i % 2 == 0 { 0.0 } else { -0.08 }), INK, 1.0);
    }
    let mut lash = PathData::new();
    lash.move_to(-60.0, -11.0).line_to(60.0, -11.0).move_to(-60.0, -5.0).line_to(60.0, -5.0);
    p.stroke(&lash, rope, 1.6);
    // A mast and a patched sail, billowing.
    let mut mast = PathData::new();
    mast.move_to(-6.0, -14.0).line_to(-6.0, -96.0);
    p.stroke(&mast, INK, 4.0);
    p.stroke(&mast, Color::hex(0x8d5a2b), 2.4);
    let belly = if k == 0 { 16.0 } else { 20.0 };
    let sail = PathData::smooth_closed(&[(-4.0, -92.0), (belly, -70.0), (-4.0, -40.0)], 0.7);
    p.inked(&sail, Color::hex(0xf5ead2), INK, 1.2);
    p.fill(&PathData::rect(2.0, -72.0, 8.0, 8.0), Color::hex(0xe57373));
    // The scarecrow captain: straw hat, button eyes, pipe.
    let mut post = PathData::new();
    post.move_to(-36.0, -14.0).line_to(-36.0, -50.0).move_to(-52.0, -44.0).line_to(-20.0, -44.0);
    p.stroke(&post, INK, 4.0);
    p.stroke(&post, Color::hex(0x8d5a2b), 2.4);
    p.inked(&PathData::round_rect(-46.0, -52.0, 20.0, 26.0, 4.0), Color::hex(0x5c8d4b), INK, 1.0);
    let mut straw = PathData::new();
    for (x, y) in [(-52.0f32, -44.0f32), (-20.0, -44.0), (-40.0, -26.0), (-32.0, -26.0)] {
        straw.move_to(x, y).line_to(x + if x < -36.0 { -5.0 } else { 5.0 }, y + 5.0);
    }
    p.stroke(&straw, Color::hex(0xf2d27b), 2.0);
    p.inked(&PathData::circle(-36.0, -60.0, 9.0), Color::hex(0xe6c79c), INK, 1.0);
    p.fill(&PathData::circle(-39.0, -61.0, 1.6), INK);
    p.fill(&PathData::circle(-33.0, -61.0, 1.6), INK);
    p.inked(&PathData::ellipse(-36.0, -68.0, 14.0, 3.6), Color::hex(0xf2d27b), INK, 0.8);
    p.inked(&PathData::ellipse(-36.0, -73.0, 7.0, 5.0), Color::hex(0xf2d27b), INK, 0.8);
}

fn canoe(p: &mut Painter, k: u32) {
    wake(p, 72.0, k);
    let red = Color::hex(0xc0392b);
    // Ducklings, one per seat, the last one facing the wrong way.
    for (i, x) in [-36.0f32, -8.0, 20.0, 44.0].into_iter().enumerate() {
        let hop = if (k as usize + i) % 2 == 0 { 0.0 } else { -2.0 };
        let y = -22.0 + hop;
        p.inked(&PathData::ellipse(x, y, 9.0, 8.0), Color::hex(0xffe082), INK, 0.9);
        let back = i == 0;
        let hx = if back { x - 5.0 } else { x + 5.0 };
        p.inked(&PathData::circle(hx, y - 9.0, 5.5), Color::hex(0xffe082), INK, 0.9);
        p.fill(&PathData::circle(hx + if back { -1.5 } else { 1.5 }, y - 10.0, 1.2), INK);
        let bx = if back { hx - 5.0 } else { hx + 5.0 };
        p.fill(&PathData::polygon(&[(bx, y - 10.0), (bx + if back { -4.0 } else { 4.0 }, y - 8.0), (bx, y - 6.0)]), Color::hex(0xff8f00));
    }
    let hull = PathData::smooth_closed(&[(-76.0, -22.0), (-50.0, -6.0), (0.0, 0.0), (50.0, -6.0), (76.0, -22.0), (0.0, -14.0)], 0.5);
    p.inked(&hull, red, INK, 1.5);
    let mut gunwale = PathData::new();
    gunwale.move_to(-74.0, -21.0).quad_to(0.0, -12.0, 74.0, -21.0);
    p.stroke(&gunwale, Color::hex(0xe8c49a), 2.4);
    // A paddle resting across.
    let mut paddle = PathData::new();
    paddle.move_to(-20.0, -8.0).line_to(26.0, -30.0);
    p.stroke(&paddle, INK, 3.4);
    p.stroke(&paddle, Color::hex(0xc48a51), 2.0);
}

fn ring(p: &mut Painter, k: u32) {
    p.fill(&PathData::ellipse(0.0, 2.0, 32.0, 8.0), WATER_SHADE);
    let bob = if k == 0 { 0.0 } else { 1.5 };
    let outer = PathData::ellipse(0.0, -8.0 + bob, 30.0, 14.0);
    p.stroke(&outer, INK, 3.0);
    // Red and white quarters.
    for q in 0..8 {
        let a0 = q as f32 * std::f32::consts::TAU / 8.0;
        let a1 = a0 + std::f32::consts::TAU / 8.0;
        let mut seg = PathData::new();
        seg.move_to(a0.cos() * 30.0, -8.0 + bob + a0.sin() * 14.0);
        let steps = 6;
        for i in 1..=steps {
            let a = a0 + (a1 - a0) * i as f32 / steps as f32;
            seg.line_to(a.cos() * 30.0, -8.0 + bob + a.sin() * 14.0);
        }
        for i in (0..=steps).rev() {
            let a = a0 + (a1 - a0) * i as f32 / steps as f32;
            seg.line_to(a.cos() * 15.0, -8.0 + bob + a.sin() * 6.0);
        }
        seg.close();
        p.fill(&seg, if q % 2 == 0 { Color::hex(0xe53935) } else { Color::hex(0xfdfdfd) });
    }
    p.fill(&PathData::ellipse(0.0, -8.0 + bob, 15.0, 6.0), Color::hex(0x4290c2));
    p.stroke(&PathData::ellipse(0.0, -8.0 + bob, 15.0, 6.0), INK, 1.6);
    p.fill(&PathData::ellipse(-14.0, -16.0 + bob, 6.0, 2.0), Color::rgba(255, 255, 255, 170));
}

/// Everything that floats, facing right and left where it matters:
/// `log_l`, `log_m`, `log_r`, `turtle_{dir}` (swim), `turtle_sink_{dir}`,
/// `turtle_under`, `duck_{dir}`, `flamingo_{dir}`, `picnic`, `lily`,
/// `lily_flower`.
pub fn sheet(tile_px: u32) -> (SheetSpec, Pixmap) {
    let mut sb = SheetBuilder::new("floats", (176.0, 128.0), (88.0, 100.0), tile_px);
    for (part, name) in ["log_l", "log_m", "log_r"].into_iter().enumerate() {
        sb.frame(name, |p| log(p, part as u32));
    }
    for (dir, left) in [("right", false), ("left", true)] {
        let with = |p: &mut Painter, f: &dyn Fn(&mut Painter)| {
            if left {
                p.push(mirror());
            }
            f(p);
            if left {
                p.pop();
            }
        };
        for k in 0..2 {
            sb.frame(&format!("turtle_{dir}"), |p| with(p, &|p| turtle(p, k, 0.0)));
        }
        for k in 0..2 {
            sb.frame(&format!("turtle_sink_{dir}"), |p| with(p, &|p| turtle(p, k, 0.45)));
        }
        for k in 0..2 {
            sb.frame(&format!("duck_{dir}"), |p| with(p, &|p| duck(p, k)));
        }
        for k in 0..2 {
            sb.frame(&format!("flamingo_{dir}"), |p| with(p, &|p| flamingo(p, k)));
        }
        for k in 0..2 {
            sb.frame(&format!("pedalo_{dir}"), |p| with(p, &|p| pedalo(p, k)));
        }
        for k in 0..2 {
            sb.frame(&format!("raft_{dir}"), |p| with(p, &|p| raft(p, k)));
        }
        for k in 0..2 {
            sb.frame(&format!("canoe_{dir}"), |p| with(p, &|p| canoe(p, k)));
        }
    }
    for k in 0..2 {
        sb.frame("ring", |p| ring(p, k));
    }
    for k in 0..2 {
        sb.frame("turtle_under", |p| turtle(p, k, 1.0));
    }
    for k in 0..2 {
        sb.frame("picnic", |p| picnic(p, k));
    }
    sb.frame("lily", |p| lily(p, false, 0));
    for k in 0..2 {
        sb.frame("lily_flower", |p| lily(p, true, k));
    }
    sb.finish()
}
