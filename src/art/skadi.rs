//! Skaði: Loki's big sister. Long orange hair (like his) in two braids tied with
//! red ribbons, a blue dress with a white embroidered hem, and boots for
//! running. Seen side on running, from the front holding the hen (with her
//! arms as a separate frame, drawn over the hen), and leaning back in a tug
//! of war.

use super::SheetBuilder;
use super::paint::{self, Painter, mirror, rot_about};
use crate::gfx::sprites::SheetSpec;
use crate::gfx::vector::{Color, PathData};
use tiny_skia::Pixmap;

const INK: Color = Color::hex(0x3a2a2a);
const SKIN: Color = Color::hex(0xf3cfb0);
const CHEEK: Color = Color::hex(0xf0a090);
const HAIR: Color = Color::hex(0xe8782e);
const HAIR_DARK: Color = Color::hex(0xb8561c);
const DRESS: Color = Color::hex(0x2f6fb8);
const DRESS_DARK: Color = Color::hex(0x24578f);
const HEM: Color = Color::hex(0xf6f1e4);
const RIBBON: Color = Color::hex(0xe53935);
const BOOT: Color = Color::hex(0x6d4c41);
const TIGHTS: Color = Color::hex(0x8e5a8a);

/// A braid from (x, y) hanging down and back, swung by `swing`.
fn braid(p: &mut Painter, x: f32, y: f32, len: f32, swing: f32) {
    let end = (x - 6.0 - swing * 8.0, y + len);
    let mut d = PathData::new();
    d.move_to(x, y).quad_to(x - 2.0 - swing * 4.0, y + len * 0.5, end.0, end.1);
    p.stroke(&d, INK, 7.4);
    p.stroke(&d, HAIR, 5.6);
    // The plaits: little chevrons down its length.
    let mut plait = PathData::new();
    for i in 1..5 {
        let t = i as f32 / 5.0;
        let px = x + (end.0 - x) * t;
        let py = y + (end.1 - y) * t;
        plait.move_to(px - 2.4, py - 1.5).line_to(px, py + 1.0).line_to(px + 2.4, py - 1.5);
    }
    p.stroke(&plait, HAIR_DARK, 1.0);
    p.inked(&PathData::ellipse(end.0, end.1, 3.6, 2.6), RIBBON, INK, 0.7);
    p.fill(&PathData::circle(end.0 - 1.0, end.1 + 3.0, 2.0), HAIR);
}

fn face_side(p: &mut Painter, x: f32, y: f32, glad: bool) {
    let face = PathData::ellipse(x + 2.0, y, 11.0, 11.5);
    p.inked(&face, SKIN, INK, 1.2);
    if glad {
        paint::happy_eye(p, x + 7.0, y - 1.0, 2.2, INK);
    } else {
        paint::eye(p, x + 7.0, y - 1.0, 2.0, 1.0, INK);
    }
    p.fill(&PathData::circle(x + 5.0, y + 5.0, 2.6), CHEEK.with_alpha(160));
    for (fx, fy) in [(x + 8.0, y + 3.0), (x + 10.0, y + 4.5), (x + 4.0, y + 3.5)] {
        p.fill(&PathData::circle(fx, fy, 0.7), Color::hex(0xc98a6a));
    }
    let mut mouth = PathData::new();
    mouth.move_to(x + 7.0, y + 7.0).quad_to(x + 10.0, y + 9.0, x + 12.0, y + 6.5);
    p.stroke(&mouth, Color::hex(0xb03a48), 1.2);
    // Hair: a cap with a parting, falling to the braid at the back.
    let hair = PathData::smooth_closed(&[(x - 10.0, y + 4.0), (x - 9.0, y - 10.0), (x + 4.0, y - 14.0), (x + 12.0, y - 8.0), (x + 4.0, y - 7.0), (x - 4.0, y - 2.0)], 0.6);
    p.inked(&hair, HAIR, INK, 1.1);
    let mut strands = PathData::new();
    strands.move_to(x + 2.0, y - 13.0).quad_to(x - 4.0, y - 8.0, x - 8.0, y + 2.0);
    p.stroke(&strands, HAIR_DARK, 1.0);
}

fn dress(p: &mut Painter, top: f32, hem: f32, flare: f32) {
    let d = PathData::smooth_closed(&[(-8.0, top), (8.0, top), (12.0 + flare, hem), (-12.0 - flare, hem)], 0.15);
    p.inked(&d, DRESS, INK, 1.2);
    p.fill(&PathData::rect(-12.0 - flare, hem - 5.0, 24.0 + flare * 2.0, 5.0), HEM);
    let mut stitch = PathData::new();
    let mut x = -10.0 - flare;
    while x < 10.0 + flare {
        stitch.move_to(x, hem - 4.0).line_to(x + 2.0, hem - 1.0).line_to(x + 4.0, hem - 4.0);
        x += 4.0;
    }
    p.stroke(&stitch, RIBBON, 0.8);
    p.fill(&PathData::rect(-8.0, top + 12.0, 16.0, 3.0), DRESS_DARK);
}

fn leg(p: &mut Painter, hip: (f32, f32), angle: f32) {
    let a = angle.to_radians();
    let foot = (hip.0 + a.sin() * 26.0, hip.1 + a.cos() * 26.0);
    let mut l = PathData::new();
    l.move_to(hip.0, hip.1).line_to(foot.0, foot.1);
    p.stroke(&l, INK, 6.4);
    p.stroke(&l, TIGHTS, 4.6);
    p.inked(&PathData::round_rect(foot.0 - 4.0, foot.1 - 6.0, 10.0, 7.0, 3.0), BOOT, INK, 0.9);
}

fn arm(p: &mut Painter, shoulder: (f32, f32), hand: (f32, f32)) {
    let mut a = PathData::new();
    a.move_to(shoulder.0, shoulder.1).line_to(hand.0, hand.1);
    p.stroke(&a, INK, 6.0);
    p.stroke(&a, DRESS, 4.4);
    p.fill(&PathData::circle(hand.0, hand.1, 3.2), SKIN);
    p.stroke(&PathData::circle(hand.0, hand.1, 3.2), INK, 0.7);
}

/// Running, side on, facing right; `k` from 0 to 3 through a stride.
fn running(p: &mut Painter, k: u32) {
    let ph = k as f32 * std::f32::consts::FRAC_PI_2;
    let stride = ph.sin() * 30.0;
    let bob = -ph.cos().abs() * 2.5;
    paint::shadow(p, 14.0, 4.0);
    p.push(crate::gfx::vector::Affine::translate(0.0, bob));
    leg(p, (-2.0, -30.0), -stride);
    arm(p, (-2.0, -62.0), (-2.0 + stride * 0.45, -44.0));
    braid(p, -6.0, -84.0, 30.0, 0.6 + ph.cos() * 0.4);
    leg(p, (2.0, -30.0), stride);
    dress(p, -66.0, -28.0, 3.0);
    arm(p, (2.0, -62.0), (4.0 - stride * 0.45, -44.0));
    face_side(p, 2.0, -82.0, false);
    braid(p, -2.0, -80.0, 28.0, 0.4 + ph.sin() * 0.3);
    p.pop();
}

/// Leaning back with her arms out, pulling (facing right).
fn tugging(p: &mut Painter, k: u32) {
    paint::shadow(p, 16.0, 4.0);
    let lean = if k == 0 { -16.0 } else { -22.0 };
    leg(p, (-2.0, -30.0), 28.0);
    leg(p, (2.0, -30.0), 8.0);
    p.push(rot_about(lean, 0.0, -30.0));
    braid(p, -6.0, -84.0, 30.0, 1.0);
    dress(p, -66.0, -30.0, 2.0);
    arm(p, (2.0, -62.0), (26.0, -58.0));
    face_side(p, 2.0, -82.0, false);
    // Gritted teeth: she is not letting go.
    p.fill(&PathData::rect(8.0, -76.0, 4.0, 2.0), Color::rgb(255, 255, 255));
    arm(p, (-2.0, -62.0), (24.0, -54.0));
    p.pop();
}

/// From the front, ready to hold the hen.
fn holding(p: &mut Painter, tilt: f32) {
    paint::shadow(p, 16.0, 4.5);
    leg(p, (-5.0, -30.0), -4.0);
    leg(p, (5.0, -30.0), 4.0);
    dress(p, -66.0, -28.0, 4.0);
    p.push(rot_about(tilt, 0.0, -70.0));
    // Braids over both shoulders.
    for s in [-1.0f32, 1.0] {
        let mut d = PathData::new();
        d.move_to(s * 9.0, -84.0).quad_to(s * 14.0, -70.0, s * 12.0, -54.0);
        p.stroke(&d, INK, 7.4);
        p.stroke(&d, HAIR, 5.6);
        p.inked(&PathData::ellipse(s * 12.0, -53.0, 3.6, 2.6), RIBBON, INK, 0.7);
    }
    let face = PathData::ellipse(0.0, -82.0, 11.5, 11.5);
    p.inked(&face, SKIN, INK, 1.2);
    paint::happy_eye(p, -4.5, -83.0, 2.2, INK);
    paint::happy_eye(p, 4.5, -83.0, 2.2, INK);
    p.fill(&PathData::circle(-7.0, -78.0, 2.4), CHEEK.with_alpha(160));
    p.fill(&PathData::circle(7.0, -78.0, 2.4), CHEEK.with_alpha(160));
    let mut mouth = PathData::new();
    mouth.move_to(-3.0, -76.0).quad_to(0.0, -73.5, 3.0, -76.0);
    p.stroke(&mouth, Color::hex(0xb03a48), 1.2);
    let fringe = PathData::smooth_closed(&[(-12.0, -84.0), (-10.0, -94.0), (0.0, -97.0), (10.0, -94.0), (12.0, -84.0), (0.0, -89.0)], 0.6);
    p.inked(&fringe, HAIR, INK, 1.1);
    let mut part = PathData::new();
    part.move_to(0.0, -97.0).line_to(0.0, -89.0);
    p.stroke(&part, HAIR_DARK, 1.0);
    p.pop();
}

/// Her arms alone, round whatever she holds against her dress.
fn arms(p: &mut Painter, squeeze: f32) {
    for s in [-1.0f32, 1.0] {
        let mut a = PathData::new();
        a.move_to(s * 9.0, -64.0).quad_to(s * (20.0 - squeeze * 2.0), -52.0, s * (5.0 - squeeze), -46.0);
        p.stroke(&a, INK, 7.0);
        p.stroke(&a, DRESS, 5.4);
        p.fill(&PathData::circle(s * (4.0 - squeeze), -46.0, 3.4), SKIN);
        p.stroke(&PathData::circle(s * (4.0 - squeeze), -46.0, 3.4), INK, 0.7);
    }
}

/// `run_{right|left}` (four frames), `tug_{right|left}` (two), `hold`
/// (two) and `arms` (two, matching).
pub fn sheet(tile_px: u32) -> (SheetSpec, Pixmap) {
    let mut sb = SheetBuilder::new("skadi", (96.0, 144.0), (48.0, 132.0), tile_px);
    for (dir, left) in [("right", false), ("left", true)] {
        for k in 0..4 {
            sb.frame(&format!("run_{dir}"), |p| {
                if left {
                    p.push(mirror());
                }
                running(p, k);
                if left {
                    p.pop();
                }
            });
        }
        for k in 0..2 {
            sb.frame(&format!("tug_{dir}"), |p| {
                if left {
                    p.push(mirror());
                }
                tugging(p, k);
                if left {
                    p.pop();
                }
            });
        }
    }
    for tilt in [-4.0f32, 4.0] {
        sb.frame("hold", |p| holding(p, tilt));
    }
    for squeeze in [0.0f32, 1.0] {
        sb.frame("arms", |p| arms(p, squeeze));
    }
    sb.finish()
}
