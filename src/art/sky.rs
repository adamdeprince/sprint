//! Things that fly over the lanes and only trouble a flying hen: kites
//! (with their tails), hot-air balloons, parcel drones, skeins of geese, a
//! blimp, paper aeroplanes and a runaway umbrella, and the shadows they
//! cast on the ground below.
//!
//! Drawn about their middle (x = 0, y = 0); the shadows about the point on
//! the ground beneath.

use super::SheetBuilder;
use super::paint::{Painter, mirror, rot_about};
use crate::gfx::sprites::SheetSpec;
use crate::gfx::vector::{Affine, Color, PathData};
use tiny_skia::Pixmap;

const INK: Color = Color::hex(0x2e2a33);
const SHADOW: Color = Color::rgba(20, 30, 20, 60);

fn kite(p: &mut Painter, k: u32) {
    let sway = if k == 0 { -6.0 } else { 6.0 };
    // The tail with its bows, flicking.
    let mut tail = PathData::new();
    tail.move_to(0.0, 26.0)
        .cubic_to(10.0 + sway, 40.0, -12.0 - sway, 52.0, 4.0 + sway, 70.0);
    p.stroke(&tail, INK, 1.6);
    for (i, t) in [0.3f32, 0.6, 0.9].into_iter().enumerate() {
        let x = sway * (if i % 2 == 0 { 0.6 } else { -0.4 }) + 2.0;
        let y = 26.0 + t * 44.0;
        let c = [Color::hex(0xffd54f), Color::hex(0x4fc3f7), Color::hex(0xff7043)][i];
        p.inked(&PathData::polygon(&[(x - 6.0, y - 4.0), (x, y), (x - 6.0, y + 4.0)]), c, INK, 0.6);
        p.inked(&PathData::polygon(&[(x + 6.0, y - 4.0), (x, y), (x + 6.0, y + 4.0)]), c, INK, 0.6);
    }
    // The string down to whoever is flying it, somewhere off the field.
    let mut string = PathData::new();
    string.move_to(0.0, 6.0).quad_to(-20.0, 40.0, -40.0, 90.0);
    p.stroke(&string, Color::rgba(255, 255, 255, 170), 1.0);
    p.push(rot_about(sway * 0.8, 0.0, 0.0));
    let red = Color::hex(0xe53935);
    let yellow = Color::hex(0xffd54f);
    let quarters = [
        (PathData::polygon(&[(0.0, -34.0), (24.0, -6.0), (0.0, 0.0)]), red),
        (PathData::polygon(&[(24.0, -6.0), (0.0, 26.0), (0.0, 0.0)]), yellow),
        (PathData::polygon(&[(0.0, 26.0), (-24.0, -6.0), (0.0, 0.0)]), red),
        (PathData::polygon(&[(-24.0, -6.0), (0.0, -34.0), (0.0, 0.0)]), yellow),
    ];
    let outline = PathData::polygon(&[(0.0, -34.0), (24.0, -6.0), (0.0, 26.0), (-24.0, -6.0)]);
    p.stroke(&outline, INK, 3.0);
    for (d, c) in &quarters {
        p.fill(d, *c);
    }
    let mut spars = PathData::new();
    spars.move_to(0.0, -34.0).line_to(0.0, 26.0).move_to(-24.0, -6.0).line_to(24.0, -6.0);
    p.stroke(&spars, Color::hex(0x8d6e63), 1.4);
    // A grumpy face: kites do not like being flown into.
    p.fill(&PathData::circle(-6.0, -10.0, 2.4), INK);
    p.fill(&PathData::circle(6.0, -10.0, 2.4), INK);
    let mut brow = PathData::new();
    brow.move_to(-10.0, -16.0).line_to(-3.0, -13.0).move_to(10.0, -16.0).line_to(3.0, -13.0);
    p.stroke(&brow, INK, 1.4);
    p.pop();
}

fn balloon(p: &mut Painter, k: u32) {
    let bob = if k == 0 { 0.0 } else { -2.0 };
    p.push(Affine::translate(0.0, bob));
    // Ropes to the basket.
    let mut ropes = PathData::new();
    ropes.move_to(-22.0, 10.0).line_to(-10.0, 34.0).move_to(22.0, 10.0).line_to(10.0, 34.0);
    p.stroke(&ropes, Color::hex(0x6d4c41), 1.4);
    // The basket, with a passenger waving a hanky.
    p.fill(&PathData::circle(-2.0, 30.0, 6.0), Color::hex(0xf2c6a0));
    p.fill(&PathData::ellipse(-2.0, 25.0, 7.0, 3.0), Color::hex(0x5d4037));
    let hanky = if k == 0 { -4.0 } else { 2.0 };
    p.fill(&PathData::polygon(&[(6.0, 26.0), (14.0, 14.0 + hanky), (18.0, 20.0 + hanky)]), Color::rgb(255, 255, 255));
    let basket = PathData::round_rect(-12.0, 32.0, 24.0, 16.0, 3.0);
    p.inked(&basket, Color::hex(0xbf8f55), INK, 1.2);
    let mut weave = PathData::new();
    weave.move_to(-12.0, 38.0).line_to(12.0, 38.0).move_to(-12.0, 43.0).line_to(12.0, 43.0);
    p.stroke(&weave, Color::hex(0x8d6e3c), 1.0);
    // The envelope: stripes of colour.
    let env = PathData::smooth_closed(&[(0.0, -62.0), (40.0, -40.0), (42.0, -6.0), (20.0, 14.0), (-20.0, 14.0), (-42.0, -6.0), (-40.0, -40.0)], 0.8);
    p.stroke(&env, INK, 3.0);
    p.fill(&env, Color::hex(0xff7043));
    let colours = [Color::hex(0xffd54f), Color::hex(0x4fc3f7), Color::hex(0x81c784)];
    for (i, c) in colours.iter().enumerate() {
        let w = 26.0 - i as f32 * 9.0;
        let stripe = PathData::smooth_closed(&[(0.0, -62.0), (w, -36.0), (w * 0.9, -4.0), (w * 0.4, 14.0), (-w * 0.4, 14.0), (-w * 0.9, -4.0), (-w, -36.0)], 0.8);
        p.fill(&stripe, *c);
    }
    p.fill(&PathData::ellipse(-18.0, -36.0, 6.0, 12.0), Color::rgba(255, 255, 255, 110));
    p.pop();
}

fn drone(p: &mut Painter, k: u32) {
    // The parcel, dangling.
    let swing = if k == 0 { -3.0 } else { 3.0 };
    let mut line = PathData::new();
    line.move_to(0.0, 4.0).line_to(swing, 22.0);
    p.stroke(&line, INK, 1.2);
    let parcel = PathData::round_rect(swing - 12.0, 22.0, 24.0, 18.0, 2.0);
    p.inked(&parcel, Color::hex(0xc9a074), INK, 1.2);
    let mut tape = PathData::new();
    tape.move_to(swing, 22.0).line_to(swing, 40.0);
    p.stroke(&tape, Color::hex(0xf5e6c8), 3.0);
    // Body and arms.
    let mut arms = PathData::new();
    arms.move_to(-30.0, -8.0).line_to(30.0, -8.0);
    p.stroke(&arms, INK, 5.0);
    p.stroke(&arms, Color::hex(0x90a4ae), 3.0);
    let body = PathData::round_rect(-12.0, -14.0, 24.0, 16.0, 6.0);
    p.inked(&body, Color::hex(0xeceff1), INK, 1.3);
    p.fill(&PathData::circle(6.0, -6.0, 3.0), Color::hex(0xe53935));
    p.fill(&PathData::circle(-4.0, -6.0, 2.0), Color::hex(0x4fc3f7));
    // Rotors, a blur.
    for x in [-30.0f32, 30.0] {
        let w = if k == 0 { 16.0 } else { 10.0 };
        p.fill(&PathData::ellipse(x, -13.0, w, 3.0), Color::rgba(200, 220, 230, 170));
        p.stroke(&PathData::ellipse(x, -13.0, w, 3.0), INK.with_alpha(120), 0.8);
        p.fill(&PathData::circle(x, -11.0, 2.4), INK);
    }
}

/// One goose, flying right, wings at `beat` (-1 down to 1 up).
fn goose(p: &mut Painter, x: f32, y: f32, beat: f32) {
    let grey = Color::hex(0x8d8a84);
    let wing = PathData::smooth_closed(&[(x - 4.0, y), (x - 2.0, y - 14.0 * beat), (x + 6.0, y - 12.0 * beat), (x + 4.0, y)], 0.6);
    p.inked(&wing, grey.shade(-0.15), INK, 0.8);
    let body = PathData::ellipse(x, y, 10.0, 4.6);
    p.inked(&body, grey, INK, 0.9);
    let mut neck = PathData::new();
    neck.move_to(x + 8.0, y - 1.0).line_to(x + 18.0, y - 3.0);
    p.stroke(&neck, INK, 4.4);
    p.stroke(&neck, Color::hex(0x2b2b2b), 3.0);
    p.fill(&PathData::circle(x + 19.0, y - 3.0, 3.0), Color::hex(0x2b2b2b));
    p.fill(&PathData::ellipse(x + 17.0, y - 1.5, 1.8, 1.0), Color::rgb(255, 255, 255));
    p.fill(&PathData::polygon(&[(x + 21.0, y - 4.0), (x + 25.0, y - 3.0), (x + 21.0, y - 2.0)]), Color::hex(0x424242));
    p.fill(&PathData::ellipse(x - 9.0, y + 0.5, 3.0, 2.4), Color::rgb(255, 255, 255));
}

fn geese(p: &mut Painter, k: u32) {
    // A skein in a V, the leader in front; beats ripple down the line.
    let spots = [(50.0f32, 0.0f32), (18.0, -16.0), (18.0, 16.0), (-14.0, -32.0), (-14.0, 32.0)];
    for (i, (x, y)) in spots.into_iter().enumerate() {
        let beat = if (i as u32 + k) % 2 == 0 { 1.0 } else { -0.6 };
        goose(p, x - 20.0, y, beat);
    }
}

fn blimp(p: &mut Painter, k: u32) {
    // A silver airship trailing a banner of bunting.
    let mut banner = PathData::new();
    banner.move_to(-86.0, -2.0).line_to(-96.0, -2.0 + if k == 0 { 2.0 } else { -2.0 });
    p.stroke(&banner, INK, 1.0);
    let env = PathData::smooth_closed(&[(-84.0, 0.0), (-60.0, -24.0), (20.0, -28.0), (82.0, -12.0), (88.0, 0.0), (82.0, 12.0), (20.0, 26.0), (-60.0, 22.0)], 0.8);
    p.inked_radial(&env, INK, 1.6, (0.0, -16.0, 90.0), &[(0.0, Color::hex(0xf5f7fa)), (0.6, Color::hex(0xc5ccd6)), (1.0, Color::hex(0x8e98a6))]);
    let mut seams = PathData::new();
    for x in [-40.0f32, 0.0, 40.0] {
        seams.move_to(x, -26.0).quad_to(x + 4.0, 0.0, x, 24.0);
    }
    p.stroke(&seams, Color::hex(0x9aa3af), 1.0);
    // Tail fins.
    for dy in [-1.0f32, 1.0] {
        let fin = PathData::polygon(&[(-62.0, dy * 10.0), (-90.0, dy * 26.0), (-82.0, dy * 4.0)]);
        p.inked(&fin, Color::hex(0xe53935), INK, 1.0);
    }
    // A stripe and the gondola, with a pilot in goggles.
    p.fill(&PathData::rect(-20.0, -4.0, 70.0, 8.0), Color::hex(0xe53935));
    let gondola = PathData::round_rect(-6.0, 22.0, 34.0, 12.0, 4.0);
    p.inked(&gondola, Color::hex(0x6d4c41), INK, 1.0);
    p.fill(&PathData::circle(16.0, 20.0, 5.0), Color::hex(0xf2c6a0));
    p.fill(&PathData::rect(12.0, 18.0, 9.0, 3.0), Color::hex(0x455a64));
    let prop = if k == 0 { 8.0 } else { 3.0 };
    p.fill(&PathData::ellipse(-10.0, 28.0, 1.6, prop), Color::rgba(80, 80, 90, 180));
}

fn plane(p: &mut Painter, k: u32) {
    let tilt = if k == 0 { -4.0 } else { 3.0 };
    p.push(super::paint::rot(tilt));
    let top = PathData::polygon(&[(26.0, 0.0), (-22.0, -14.0), (-12.0, 0.0)]);
    let bottom = PathData::polygon(&[(26.0, 0.0), (-22.0, 10.0), (-12.0, 0.0)]);
    p.inked(&top, Color::hex(0xfdfdfd), INK, 1.0);
    p.inked(&bottom, Color::hex(0xe0e4ea), INK, 1.0);
    let mut fold = PathData::new();
    fold.move_to(26.0, 0.0).line_to(-14.0, 2.0);
    p.stroke(&fold, Color::hex(0x9aa3af), 1.0);
    // Somebody's maths homework, by the look of it.
    let mut lines = PathData::new();
    lines.move_to(-14.0, -8.0).line_to(6.0, -4.0).move_to(-12.0, -4.0).line_to(10.0, -2.0);
    p.stroke(&lines, Color::hex(0x90caf9), 0.8);
    p.pop();
    let mut whoosh = PathData::new();
    whoosh.move_to(-30.0, -4.0).line_to(-42.0, -4.0).move_to(-30.0, 4.0).line_to(-38.0, 4.0);
    p.stroke(&whoosh, Color::rgba(255, 255, 255, 170), 1.4);
}

fn umbrella(p: &mut Painter, k: u32) {
    let sway = if k == 0 { -8.0 } else { 8.0 };
    p.push(super::paint::rot(sway));
    let canopy = PathData::smooth_closed(&[(-34.0, 0.0), (-26.0, -20.0), (0.0, -30.0), (26.0, -20.0), (34.0, 0.0)], 0.7);
    p.stroke(&canopy, INK, 3.0);
    p.fill(&canopy, Color::hex(0x5e35b1));
    for (i, x) in [-22.0f32, 0.0, 22.0].into_iter().enumerate() {
        let panel = PathData::polygon(&[(x - 6.0, -2.0), (x * 0.4, -28.0), (x + 6.0, -2.0)]);
        p.fill(&panel, if i % 2 == 0 { Color::hex(0xffca28) } else { Color::hex(0x7e57c2) });
    }
    let mut scallops = PathData::new();
    scallops.move_to(-34.0, 0.0);
    for i in 0..6 {
        let x0 = -34.0 + i as f32 * 68.0 / 6.0;
        scallops.quad_to(x0 + 5.6, 5.0, x0 + 11.3, 0.0);
    }
    p.stroke(&scallops, INK, 2.0);
    // The handle, and a carpet bag hanging off it.
    let mut stick = PathData::new();
    stick.move_to(0.0, -30.0).line_to(0.0, 30.0).quad_to(0.0, 38.0, -7.0, 36.0);
    p.stroke(&stick, INK, 3.2);
    p.stroke(&stick, Color::hex(0x8d5a2b), 1.8);
    let bag = PathData::round_rect(-8.0, 38.0, 18.0, 13.0, 3.0);
    p.inked(&bag, Color::hex(0xad1457), INK, 1.0);
    p.fill(&PathData::rect(-8.0, 43.0, 18.0, 2.0), Color::hex(0xffd54f));
    p.pop();
}

fn shadow(p: &mut Painter, rx: f32, ry: f32) {
    p.fill(&PathData::ellipse(0.0, 0.0, rx, ry), SHADOW);
}

/// `kite`, `balloon`, `umbrella`, then `drone`, `geese`, `blimp` and
/// `plane` facing right and left, and a `shadow_{name}` for each.
pub fn sheet(tile_px: u32) -> (SheetSpec, Pixmap) {
    let mut sb = SheetBuilder::new("sky", (208.0, 176.0), (104.0, 84.0), tile_px);
    for k in 0..2 {
        sb.frame("kite", |p| kite(p, k));
    }
    for k in 0..2 {
        sb.frame("balloon", |p| balloon(p, k));
    }
    for k in 0..2 {
        sb.frame("umbrella", |p| umbrella(p, k));
    }
    type Draw = fn(&mut Painter, u32);
    let facing: [(&str, Draw); 4] = [("drone", drone), ("geese", geese), ("blimp", blimp), ("plane", plane)];
    for (dir, left) in [("right", false), ("left", true)] {
        for (name, draw) in facing {
            for k in 0..2 {
                sb.frame(&format!("{name}_{dir}"), |p| {
                    if left {
                        p.push(mirror());
                    }
                    draw(p, k);
                    if left {
                        p.pop();
                    }
                });
            }
        }
    }
    sb.frame("shadow_kite", |p| shadow(p, 18.0, 6.0));
    sb.frame("shadow_balloon", |p| shadow(p, 34.0, 9.0));
    sb.frame("shadow_drone", |p| shadow(p, 24.0, 6.0));
    sb.frame("shadow_umbrella", |p| shadow(p, 26.0, 6.0));
    sb.frame("shadow_geese", |p| {
        for (x, y) in [(30.0f32, 0.0f32), (-2.0, -6.0), (-2.0, 6.0), (-34.0, -12.0), (-34.0, 12.0)] {
            p.fill(&PathData::ellipse(x, y * 0.5, 9.0, 3.0), SHADOW);
        }
    });
    sb.frame("shadow_blimp", |p| shadow(p, 80.0, 12.0));
    sb.frame("shadow_plane", |p| shadow(p, 18.0, 3.6));
    sb.finish()
}
