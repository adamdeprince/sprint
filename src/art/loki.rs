//! Loki: a small, determined child with a shock of ginger hair, a stripy
//! jumper and wellies, bouncing about on an orange space hopper. Also the
//! cuddle: Loki seen from the front with his arms ready, and the arms
//! alone, which are drawn over the hen so that she is held, not stood on.

use super::SheetBuilder;
use super::paint::{self, Cloud, Painter, mirror, rot_about, scale_about};
use crate::gfx::sprites::SheetSpec;
use crate::gfx::vector::{Affine, Color, PathData};
use tiny_skia::Pixmap;

const INK: Color = Color::hex(0x3a2a2a);
const SKIN: Color = Color::hex(0xf6cfae);
const CHEEK: Color = Color::hex(0xf59a8a);
const HAIR: Color = Color::hex(0xe0702a);
const HAIR_LIGHT: Color = Color::hex(0xf39a4e);
const JUMPER: Color = Color::hex(0x3d7bd6);
const STRIPE: Color = Color::hex(0xfdf3d8);
const SHORTS: Color = Color::hex(0x5d4a7a);
const WELLY: Color = Color::hex(0x2e9c5a);
const HOPPER: Color = Color::hex(0xf28c28);
const HOPPER_DARK: Color = Color::hex(0xc8641a);

/// The space hopper, squashed by `squash` (1 round, less is flatter).
fn hopper(p: &mut Painter, squash: f32, facing_front: bool) {
    let ry = 21.0 * squash;
    let rx = 21.0 / squash.sqrt();
    let cy = -ry;
    let ball = PathData::ellipse(0.0, cy, rx, ry);
    p.inked_radial(&ball, INK, 1.5, (-7.0, cy - 9.0, rx * 1.6), &[(0.0, Color::hex(0xffc27a)), (0.5, HOPPER), (1.0, HOPPER_DARK)]);
    // The face printed on it, grinning.
    let fx = if facing_front { 0.0 } else { 8.0 };
    p.fill(&PathData::circle(fx - 6.0, cy - 4.0, 2.2), INK);
    p.fill(&PathData::circle(fx + 6.0, cy - 4.0, 2.2), INK);
    let mut grin = PathData::new();
    grin.move_to(fx - 8.0, cy + 4.0).quad_to(fx, cy + 11.0 * squash, fx + 8.0, cy + 4.0);
    p.stroke(&grin, INK, 1.6);
    p.fill(&PathData::ellipse(-9.0, cy - 10.0 * squash, 5.0, 3.0), Color::rgba(255, 255, 255, 150));
    // The horns he holds on to.
    for x in [-9.0f32, 9.0] {
        let x = if facing_front { x } else { x * 0.5 + 6.0 };
        let mut horn = PathData::new();
        horn.move_to(x, cy - ry + 3.0).quad_to(x * 1.3, cy - ry - 7.0, x * 1.1 + 2.0, cy - ry - 12.0);
        p.stroke(&horn, INK, 6.0);
        p.stroke(&horn, HOPPER, 4.0);
    }
}

fn head(p: &mut Painter, x: f32, y: f32, front: bool, joy: f32) {
    // Hair behind.
    let hair_back = Cloud::new(x - if front { 0.0 } else { 3.0 }, y - 4.0, 16.0, 14.0, 9, 0.4, 41);
    p.cloud(&hair_back, INK, 1.3, HAIR);
    let face = PathData::ellipse(x + if front { 0.0 } else { 3.0 }, y + 2.0, 13.0, 12.5);
    p.inked(&face, SKIN, INK, 1.2);
    let (ex, ex2) = if front { (x - 5.0, x + 5.0) } else { (x + 6.0, x + 12.0) };
    if joy > 0.5 {
        paint::happy_eye(p, ex, y, 2.4, INK);
        paint::happy_eye(p, ex2, y, 2.4, INK);
    } else {
        paint::eye(p, ex, y, 2.2, 1.0, INK);
        paint::eye(p, ex2, y, 2.2, 1.0, INK);
    }
    p.fill(&PathData::circle(ex - 2.5, y + 6.0, 3.0), CHEEK.with_alpha(170));
    p.fill(&PathData::circle(ex2 + 2.5, y + 6.0, 3.0), CHEEK.with_alpha(170));
    let mx = (ex + ex2) / 2.0;
    let mut mouth = PathData::new();
    mouth.move_to(mx - 4.0, y + 7.0).quad_to(mx, y + 11.0 + joy * 3.0, mx + 4.0, y + 7.0).close();
    p.inked(&mouth, Color::hex(0xb03a48), INK, 0.6);
    // A fringe flopping forward, and the one tuft that never lies flat.
    let fringe = Cloud::new(x + if front { 0.0 } else { 4.0 }, y - 10.0, 13.0, 6.0, 7, 0.45, 43);
    p.cloud_lit(&fringe, INK, 1.1, x - 4.0, y - 14.0, &[(0.0, HAIR_LIGHT), (1.0, HAIR)]);
    let mut tuft = PathData::new();
    tuft.move_to(x - 2.0, y - 14.0).quad_to(x + 2.0, y - 26.0, x + 8.0, y - 24.0);
    p.stroke(&tuft, INK, 4.0);
    p.stroke(&tuft, HAIR, 2.4);
}

fn jumper(p: &mut Painter, x: f32, y: f32, w: f32, h: f32) {
    let body = PathData::round_rect(x - w / 2.0, y, w, h, 7.0);
    p.inked(&body, JUMPER, INK, 1.3);
    let mut stripes = PathData::new();
    for i in 1..4 {
        let sy = y + i as f32 * h / 4.0;
        stripes.move_to(x - w / 2.0 + 1.5, sy).line_to(x + w / 2.0 - 1.5, sy);
    }
    p.stroke(&stripes, STRIPE, 2.6);
}

/// On the hopper, side on (facing right), at height `lift` above the
/// ground with the ball squashed by `squash`.
fn bouncing(p: &mut Painter, lift: f32, squash: f32) {
    paint::shadow(p, 18.0 - lift * 0.15, 4.5);
    p.push(Affine::translate(0.0, -lift));
    hopper(p, squash, false);
    let seat = -42.0 * squash;
    // Legs astride, wellies dangling.
    for (x, a) in [(-6.0f32, 12.0f32), (6.0, -8.0)] {
        let mut leg = PathData::new();
        leg.move_to(x, seat).line_to(x + a * 0.6, seat + 18.0);
        p.stroke(&leg, INK, 8.0);
        p.stroke(&leg, SHORTS, 6.0);
        p.inked(&PathData::round_rect(x + a * 0.6 - 4.0, seat + 14.0, 10.0, 10.0, 3.0), WELLY, INK, 1.0);
    }
    jumper(p, 2.0, seat - 26.0, 22.0, 26.0);
    // Arms forward to the horns.
    let mut arm = PathData::new();
    arm.move_to(4.0, seat - 20.0).quad_to(12.0, seat - 12.0, 14.0, seat - 3.0);
    p.stroke(&arm, INK, 7.0);
    p.stroke(&arm, JUMPER, 5.0);
    p.fill(&PathData::circle(14.0, seat - 2.0, 3.4), SKIN);
    head(p, 4.0, seat - 40.0, false, if lift > 6.0 { 1.0 } else { 0.0 });
    p.pop();
}

/// On the hopper, leaning back hard with both arms out (facing right):
/// pulling in a tug of war.
fn tugging(p: &mut Painter, k: u32) {
    paint::shadow(p, 18.0, 4.5);
    hopper(p, 0.86, false);
    let seat = -42.0 * 0.86;
    let lean = if k == 0 { -14.0 } else { -20.0 };
    p.push(rot_about(lean, 0.0, seat));
    for (x, a) in [(-6.0f32, 20.0f32), (6.0, 4.0)] {
        let mut leg = PathData::new();
        leg.move_to(x, seat).line_to(x + a * 0.6, seat + 18.0);
        p.stroke(&leg, INK, 8.0);
        p.stroke(&leg, SHORTS, 6.0);
        p.inked(&PathData::round_rect(x + a * 0.6 - 4.0, seat + 14.0, 10.0, 10.0, 3.0), WELLY, INK, 1.0);
    }
    jumper(p, 2.0, seat - 26.0, 22.0, 26.0);
    for dy in [0.0f32, 5.0] {
        let mut arm = PathData::new();
        arm.move_to(4.0, seat - 20.0 + dy).line_to(26.0, seat - 18.0 + dy);
        p.stroke(&arm, INK, 7.0);
        p.stroke(&arm, JUMPER, 5.0);
        p.fill(&PathData::circle(27.0, seat - 18.0 + dy, 3.4), SKIN);
    }
    head(p, 4.0, seat - 40.0, false, 0.0);
    // Puffed cheeks of effort.
    p.fill(&PathData::circle(12.0, seat - 34.0, 3.4), CHEEK);
    p.pop();
}

/// From the front, sitting on the hopper with his arms out for a cuddle.
fn cuddle(p: &mut Painter, tilt: f32) {
    paint::shadow(p, 20.0, 5.0);
    hopper(p, 0.92, true);
    let seat = -40.0;
    for x in [-8.0f32, 8.0] {
        let mut leg = PathData::new();
        leg.move_to(x, seat).line_to(x * 1.6, seat + 16.0);
        p.stroke(&leg, INK, 8.0);
        p.stroke(&leg, SHORTS, 6.0);
        p.inked(&PathData::round_rect(x * 1.6 - 5.0, seat + 12.0, 10.0, 10.0, 3.0), WELLY, INK, 1.0);
    }
    jumper(p, 0.0, seat - 28.0, 28.0, 28.0);
    p.push(rot_about(tilt, 0.0, seat - 28.0));
    head(p, 0.0, seat - 44.0, true, 1.0);
    p.pop();
}

/// The arms alone, wrapped round whatever sits in front of his jumper.
fn arms(p: &mut Painter, squeeze: f32) {
    let seat = -40.0;
    for s in [-1.0f32, 1.0] {
        let mut arm = PathData::new();
        arm.move_to(s * 13.0, seat - 24.0).quad_to(s * (24.0 - squeeze * 3.0), seat - 12.0, s * (6.0 - squeeze * 2.0), seat - 8.0);
        p.stroke(&arm, INK, 8.0);
        p.stroke(&arm, JUMPER, 6.0);
        p.fill(&PathData::circle(s * (5.0 - squeeze * 2.0), seat - 8.0, 3.8), SKIN);
        p.stroke(&PathData::circle(s * (5.0 - squeeze * 2.0), seat - 8.0, 3.8), INK, 0.8);
    }
}

/// `bounce_{right|left}` (four frames through a bounce), `tug_{right|left}`
/// (two), `cuddle` (two frames) and `arms` (two, matching).
pub fn sheet(tile_px: u32) -> (SheetSpec, Pixmap) {
    let mut sb = SheetBuilder::new("loki", (96.0, 144.0), (48.0, 132.0), tile_px);
    let bounce = [(0.0f32, 0.78f32), (8.0, 1.05), (18.0, 1.0), (8.0, 1.05)];
    for (dir, left) in [("right", false), ("left", true)] {
        for (lift, squash) in bounce {
            sb.frame(&format!("bounce_{dir}"), |p| {
                p.push(scale_about(0.9, 0.9, 0.0, 0.0));
                if left {
                    p.push(mirror());
                }
                bouncing(p, lift, squash);
                if left {
                    p.pop();
                }
                p.pop();
            });
        }
    }
    for (dir, left) in [("right", false), ("left", true)] {
        for k in 0..2 {
            sb.frame(&format!("tug_{dir}"), |p| {
                p.push(scale_about(0.9, 0.9, 0.0, 0.0));
                if left {
                    p.push(mirror());
                }
                tugging(p, k);
                if left {
                    p.pop();
                }
                p.pop();
            });
        }
    }
    for tilt in [-6.0f32, 6.0] {
        sb.frame("cuddle", |p| {
            p.push(scale_about(0.9, 0.9, 0.0, 0.0));
            cuddle(p, tilt);
            p.pop();
        });
    }
    for squeeze in [0.0f32, 1.0] {
        sb.frame("arms", |p| {
            p.push(scale_about(0.9, 0.9, 0.0, 0.0));
            arms(p, squeeze);
            p.pop();
        });
    }
    sb.finish()
}
