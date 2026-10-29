//! Road traffic, side on and facing right (mirrored for left): a cherry
//! hatchback with a sheepdog at the window, a tractor, an ice-cream van
//! with a giant cone on the roof, a motorised sofa driven by a knitting
//! granny, a stretch limousine in segments (so it can be any length), a
//! mobility scooter, a bathtub racer, a ride-on mower, a double-decker bus
//! full of sheep, a tandem, a camper van with a surfboard, a milk float, a
//! clown on a unicycle, and the Custard Express. And on the sidewalks: a
//! cyclist with a terrier in her basket, a dad pushing a pram, and a girl
//! pulling her little brother along in a red wagon.
//!
//! Each vehicle is drawn with its wheels on y = 0 and its middle at x = 0.

use super::SheetBuilder;
use super::paint::{Painter, mirror, rot_about};
use crate::gfx::sprites::SheetSpec;
use crate::gfx::vector::{Affine, Color, PathData};
use tiny_skia::Pixmap;

const INK: Color = Color::hex(0x2e2a33);
const TYRE: Color = Color::hex(0x2f2c30);
const HUB: Color = Color::hex(0xc9ccd2);
const GLASS: Color = Color::hex(0xa9dcf2);
const GLASS_DARK: Color = Color::hex(0x6fa9c8);
const CHROME: Color = Color::hex(0xe6e9ee);
const SKIN: Color = Color::hex(0xf2c6a0);
const LAMP: Color = Color::hex(0xfff1a8);
const TAIL: Color = Color::hex(0xe04848);

fn ground_shadow(p: &mut Painter, half: f32) {
    p.fill(&PathData::ellipse(0.0, 1.0, half, 5.0), Color::rgba(20, 20, 30, 70));
}

/// A tyre with a hub; `spin` turns the spokes.
fn wheel(p: &mut Painter, x: f32, r: f32, spin: f32) {
    let y = -r;
    p.fill(&PathData::circle(x, y, r + 1.2), INK);
    p.fill(&PathData::circle(x, y, r), TYRE);
    p.fill(&PathData::circle(x, y, r * 0.55), HUB);
    let mut spokes = PathData::new();
    for k in 0..3 {
        let a = spin + k as f32 * std::f32::consts::TAU / 3.0;
        spokes.move_to(x, y).line_to(x + a.cos() * r * 0.5, y + a.sin() * r * 0.5);
    }
    p.stroke(&spokes, Color::hex(0x8a8f99), r * 0.14);
    p.fill(&PathData::circle(x, y, r * 0.16), Color::hex(0x6a6f78));
}

fn window(p: &mut Painter, d: &PathData) {
    p.inked(d, GLASS, INK, 1.0);
    if let Some((x0, y0, _, y1)) = d.bounds() {
        let mut glint = PathData::new();
        glint.move_to(x0 + 4.0, y1 - 3.0).line_to(x0 + 10.0, y0 + 3.0);
        p.stroke(&glint, Color::rgba(255, 255, 255, 190), 2.4);
    }
}

fn hatchback(p: &mut Painter, k: u32) {
    let spin = k as f32 * 1.0;
    let bob = if k == 1 { -0.8 } else { 0.0 };
    ground_shadow(p, 50.0);
    let red = Color::hex(0xd8443c);
    p.push(Affine::translate(0.0, bob));
    let cabin = PathData::smooth_closed(&[(-42.0, -32.0), (-34.0, -56.0), (8.0, -58.0), (30.0, -34.0)], 0.35);
    p.inked(&cabin, red, INK, 1.4);
    let body = PathData::round_rect(-50.0, -38.0, 100.0, 26.0, 9.0);
    p.inked(&body, red, INK, 1.4);
    p.fill(&PathData::round_rect(-48.0, -24.0, 96.0, 5.0, 2.0), red.shade(-0.18));
    window(p, &PathData::polygon(&[(-34.0, -36.0), (-28.0, -52.0), (-8.0, -53.0), (-8.0, -36.0)]));
    window(p, &PathData::polygon(&[(-3.0, -36.0), (-3.0, -53.0), (7.0, -53.0), (23.0, -36.0)]));
    // A sheepdog with its head out of the back window, ears flying.
    p.fill(&PathData::ellipse(-22.0, -50.0, 9.0, 8.0), Color::hex(0xf4f1ea));
    p.stroke(&PathData::ellipse(-22.0, -50.0, 9.0, 8.0), INK, 1.0);
    let ear = PathData::smooth_closed(&[(-29.0, -54.0), (-40.0, -58.0 - bob * 3.0), (-37.0, -50.0)], 1.0);
    p.inked(&ear, Color::hex(0x6d6a72), INK, 0.8);
    p.fill(&PathData::circle(-16.0, -52.0, 1.6), INK);
    p.fill(&PathData::circle(-14.0, -48.0, 2.2), INK);
    p.fill(&PathData::ellipse(48.0, -30.0, 3.0, 4.0), LAMP);
    p.fill(&PathData::ellipse(-49.0, -30.0, 2.4, 4.0), TAIL);
    p.fill(&PathData::round_rect(-6.0, -31.0, 7.0, 2.4, 1.0), red.shade(-0.4));
    p.pop();
    wheel(p, -29.0, 11.0, spin);
    wheel(p, 30.0, 11.0, spin);
}

fn tractor(p: &mut Painter, k: u32) {
    let spin = k as f32 * 0.8;
    let bob = if k == 1 { -1.2 } else { 0.0 };
    ground_shadow(p, 62.0);
    let green = Color::hex(0x4f9a3a);
    let rim = Color::hex(0xf2c230);
    p.push(Affine::translate(0.0, bob));
    // Exhaust and its puff.
    let mut pipe = PathData::new();
    pipe.move_to(30.0, -40.0).line_to(30.0, -70.0);
    p.stroke(&pipe, INK, 6.0);
    p.stroke(&pipe, Color::hex(0x8b8f96), 3.6);
    let puff_y = if k == 0 { -78.0 } else { -82.0 };
    p.fill(&PathData::circle(27.0, puff_y, 5.5), Color::rgba(220, 220, 220, 200));
    p.fill(&PathData::circle(21.0, puff_y - 7.0, 4.0), Color::rgba(230, 230, 230, 150));
    // Bonnet and grille.
    let hood = PathData::round_rect(-8.0, -44.0, 66.0, 26.0, 6.0);
    p.inked(&hood, green, INK, 1.4);
    let mut louvres = PathData::new();
    for i in 0..4 {
        let x = 6.0 + i as f32 * 10.0;
        louvres.move_to(x, -38.0).line_to(x, -26.0);
    }
    p.stroke(&louvres, green.shade(-0.3), 2.0);
    p.fill(&PathData::ellipse(56.0, -36.0, 3.0, 4.0), LAMP);
    // Cab: posts and a roof.
    let mut posts = PathData::new();
    posts.move_to(-50.0, -44.0).line_to(-48.0, -86.0).move_to(-12.0, -44.0).line_to(-14.0, -86.0);
    p.stroke(&posts, INK, 5.0);
    p.stroke(&posts, green.shade(-0.2), 3.0);
    p.inked(&PathData::round_rect(-56.0, -92.0, 50.0, 8.0, 3.0), Color::hex(0xf5f1e6), INK, 1.2);
    window(p, &PathData::polygon(&[(-46.0, -82.0), (-16.0, -82.0), (-16.0, -50.0), (-46.0, -50.0)]));
    // A farmer in a flat cap.
    p.fill(&PathData::circle(-30.0, -64.0, 7.0), SKIN);
    p.fill(&PathData::ellipse(-30.0, -70.0, 8.5, 3.6), Color::hex(0x6b5a45));
    p.fill(&PathData::round_rect(-37.0, -58.0, 14.0, 10.0, 3.0), Color::hex(0x3e64a8));
    p.inked(&PathData::round_rect(-56.0, -48.0, 50.0, 10.0, 3.0), green, INK, 1.2);
    p.pop();
    // Big wheel at the back, small at the front.
    p.fill(&PathData::circle(-31.0, -26.0, 27.0), INK);
    p.fill(&PathData::circle(-31.0, -26.0, 25.5), TYRE);
    let mut tread = PathData::new();
    for i in 0..12 {
        let a = spin + i as f32 * std::f32::consts::TAU / 12.0;
        tread.move_to(-31.0 + a.cos() * 20.0, -26.0 + a.sin() * 20.0).line_to(-31.0 + a.cos() * 25.0, -26.0 + a.sin() * 25.0);
    }
    p.stroke(&tread, Color::hex(0x55515a), 3.0);
    p.fill(&PathData::circle(-31.0, -26.0, 13.0), rim);
    p.fill(&PathData::circle(-31.0, -26.0, 4.0), rim.shade(-0.35));
    wheel(p, 40.0, 14.0, spin);
    p.fill(&PathData::circle(40.0, -14.0, 7.0), rim);
}

fn ice_cream(p: &mut Painter, k: u32) {
    let spin = k as f32;
    let bob = if k == 1 { -0.8 } else { 0.0 };
    ground_shadow(p, 74.0);
    let white = Color::hex(0xfbf7ef);
    let pink = Color::hex(0xf48fb1);
    p.push(Affine::translate(0.0, bob));
    // The giant cone on the roof, leaning back in the breeze.
    p.push(rot_about(-8.0 + k as f32 * 3.0, -10.0, -76.0));
    let cone = PathData::polygon(&[(-22.0, -100.0), (2.0, -100.0), (-10.0, -72.0)]);
    p.inked(&cone, Color::hex(0xe0a95a), INK, 1.2);
    let mut waffle = PathData::new();
    for i in 0..3 {
        let t = i as f32 * 7.0;
        waffle.move_to(-19.0 + t * 0.7, -100.0).line_to(-8.0 + t * 0.2, -78.0);
    }
    p.stroke(&waffle, Color::hex(0xb37a35), 1.2);
    for (x, y, r, c) in [(-16.0f32, -104.0f32, 10.0f32, pink), (-3.0, -106.0, 9.0, Color::hex(0xfff2c2)), (-10.0, -116.0, 9.0, Color::hex(0x8d5a3b))] {
        p.inked(&PathData::circle(x, y, r), c, INK, 1.2);
    }
    p.fill(&PathData::circle(-8.0, -126.0, 3.6), Color::hex(0xe53935));
    p.pop();
    let body = PathData::round_rect(-76.0, -72.0, 152.0, 60.0, 12.0);
    p.inked(&body, white, INK, 1.4);
    let cab = PathData::smooth_closed(&[(42.0, -72.0), (64.0, -70.0), (78.0, -46.0), (78.0, -14.0), (42.0, -14.0)], 0.3);
    p.inked(&cab, white, INK, 1.4);
    window(p, &PathData::polygon(&[(48.0, -66.0), (62.0, -65.0), (72.0, -46.0), (48.0, -46.0)]));
    // Serving hatch with a striped awning.
    p.fill(&PathData::round_rect(-60.0, -60.0, 82.0, 30.0, 4.0), Color::hex(0x5b4a58));
    for i in 0..6 {
        let x = -64.0 + i as f32 * 15.0;
        let c = if i % 2 == 0 { pink } else { white };
        p.inked(&PathData::polygon(&[(x, -66.0), (x + 15.0, -66.0), (x + 15.0, -58.0), (x + 7.5, -54.0), (x, -58.0)]), c, INK, 0.8);
    }
    // A lolly sign and a stripe.
    p.fill(&PathData::round_rect(-76.0, -26.0, 152.0, 7.0, 2.0), pink);
    p.inked(&PathData::circle(-46.0, -42.0, 6.0), Color::hex(0xffd54f), INK, 1.0);
    p.inked(&PathData::circle(-28.0, -40.0, 5.0), Color::hex(0x81d4fa), INK, 1.0);
    p.fill(&PathData::ellipse(77.0, -24.0, 2.6, 4.0), LAMP);
    p.fill(&PathData::ellipse(-76.0, -24.0, 2.4, 4.0), TAIL);
    p.pop();
    wheel(p, -50.0, 12.0, spin);
    wheel(p, 52.0, 12.0, spin);
}

fn sofa(p: &mut Painter, k: u32) {
    let spin = k as f32;
    ground_shadow(p, 58.0);
    let fabric = Color::hex(0x3f9e9a);
    let dark = fabric.shade(-0.3);
    p.push(Affine::translate(0.0, if k == 1 { -1.0 } else { 0.0 }));
    // Back, seat and arms.
    p.inked(&PathData::round_rect(-56.0, -62.0, 82.0, 34.0, 12.0), fabric, INK, 1.4);
    // Granny: white bun, glasses, knitting.
    p.fill(&PathData::round_rect(-16.0, -58.0, 24.0, 28.0, 8.0), Color::hex(0xb05c9a));
    p.fill(&PathData::circle(-4.0, -66.0, 10.0), SKIN);
    p.inked(&PathData::circle(-7.0, -77.0, 7.0), Color::hex(0xf2f2f2), INK, 0.8);
    p.stroke(&PathData::circle(-1.0, -66.0, 3.4), INK, 1.0);
    p.stroke(&PathData::circle(6.0, -66.0, 3.4), INK, 1.0);
    let mut needles = PathData::new();
    let wag = if k == 0 { 0.0 } else { 3.0 };
    needles.move_to(4.0, -36.0).line_to(20.0, -50.0 + wag).move_to(6.0, -34.0).line_to(22.0, -42.0 - wag);
    p.stroke(&needles, Color::hex(0x9e9e9e), 1.6);
    p.inked(&PathData::circle(10.0, -38.0, 5.0), Color::hex(0xffb74d), INK, 0.8);
    p.inked(&PathData::round_rect(-58.0, -36.0, 104.0, 18.0, 6.0), fabric, INK, 1.4);
    for x in [-40.0f32, -14.0, 12.0] {
        p.stroke(&PathData::round_rect(x, -35.0, 24.0, 8.0, 3.0), dark, 1.2);
    }
    p.inked(&PathData::round_rect(-64.0, -48.0, 14.0, 32.0, 6.0), fabric, INK, 1.4);
    p.inked(&PathData::round_rect(40.0, -48.0, 14.0, 32.0, 6.0), fabric, INK, 1.4);
    // Flowers on the fabric.
    for (x, y) in [(-44.0f32, -48.0f32), (-26.0, -54.0), (16.0, -52.0)] {
        p.fill(&PathData::circle(x, y, 2.6), Color::hex(0xffe0b2));
    }
    // A little steering wheel on a stalk.
    let mut col = PathData::new();
    col.move_to(54.0, -40.0).line_to(60.0, -60.0);
    p.stroke(&col, INK, 2.4);
    p.stroke(&PathData::ellipse(60.0, -62.0, 3.0, 7.0), INK, 2.2);
    p.pop();
    wheel(p, -44.0, 8.0, spin);
    wheel(p, 34.0, 8.0, spin);
}

const LIMO: Color = Color::hex(0x24222b);
const LIMO_SHINE: Color = Color::hex(0x4a4757);

/// One tile of limousine: `part` 0 is the nose, 1 a middle stretch, 2 the
/// tail. Segments butt together exactly at x = ±32.
fn limo(p: &mut Painter, part: u32, k: u32) {
    let spin = k as f32;
    p.fill(&PathData::rect(-33.0, -4.0, 66.0, 9.0), Color::rgba(20, 20, 30, 70));
    // Segments overlap a little so the road never shows between them.
    let (x0, x1) = match part {
        0 => (-35.0, 30.0),
        2 => (-30.0, 35.0),
        _ => (-35.0, 35.0),
    };
    let body = PathData::round_rect(x0, -38.0, x1 - x0, 26.0, if part == 1 { 0.0 } else { 9.0 });
    p.fill(&body, LIMO);
    let mut edge = PathData::new();
    edge.move_to(x0, -38.0).line_to(x1, -38.0).move_to(x0, -12.0).line_to(x1, -12.0);
    p.stroke(&edge, INK, 2.8);
    p.fill(&PathData::rect(x0, -28.0, x1 - x0, 2.2), CHROME);
    p.fill(&PathData::rect(x0, -36.0, x1 - x0, 3.0), LIMO_SHINE);
    match part {
        0 => {
            let roof = PathData::polygon(&[(-35.0, -38.0), (-35.0, -54.0), (-6.0, -54.0), (12.0, -38.0)]);
            p.inked(&roof, LIMO, INK, 1.3);
            p.fill(&PathData::polygon(&[(-31.0, -40.0), (-31.0, -51.0), (-8.0, -51.0), (6.0, -40.0)]), GLASS_DARK);
            p.stroke(&PathData::round_rect(x0, -38.0, x1 - x0, 26.0, 9.0), INK, 1.4);
            p.fill(&PathData::ellipse(28.0, -30.0, 2.6, 3.6), LAMP);
            // The bonnet ornament.
            p.fill(&PathData::circle(24.0, -41.0, 2.0), CHROME);
            wheel(p, 12.0, 11.0, spin);
        }
        2 => {
            let roof = PathData::polygon(&[(-12.0, -38.0), (4.0, -54.0), (35.0, -54.0), (35.0, -38.0)]);
            p.inked(&roof, LIMO, INK, 1.3);
            p.fill(&PathData::polygon(&[(-6.0, -40.0), (6.0, -51.0), (31.0, -51.0), (31.0, -40.0)]), GLASS_DARK);
            p.stroke(&PathData::round_rect(x0, -38.0, x1 - x0, 26.0, 9.0), INK, 1.4);
            p.fill(&PathData::ellipse(-29.0, -30.0, 2.4, 3.6), TAIL);
            wheel(p, -12.0, 11.0, spin);
        }
        _ => {
            p.fill(&PathData::rect(-35.0, -54.0, 70.0, 16.0), LIMO);
            let mut roofline = PathData::new();
            roofline.move_to(-35.0, -54.0).line_to(35.0, -54.0);
            p.stroke(&roofline, INK, 2.8);
            p.fill(&PathData::rect(-28.0, -51.0, 56.0, 11.0), GLASS_DARK);
            // A glimpse of the party inside.
            p.fill(&PathData::circle(-10.0, -43.0, 4.0), Color::hex(0xffcc80).with_alpha(160));
            p.fill(&PathData::circle(12.0, -44.0, 3.0), Color::hex(0xce93d8).with_alpha(160));
        }
    }
}

fn scooter(p: &mut Painter, k: u32) {
    let spin = k as f32;
    ground_shadow(p, 36.0);
    let red = Color::hex(0xc0392b);
    // Flag on a whippy pole.
    let mut pole = PathData::new();
    pole.move_to(-28.0, -30.0).quad_to(-32.0, -70.0, -36.0 + k as f32 * 3.0, -98.0);
    p.stroke(&pole, INK, 1.4);
    p.fill(&PathData::polygon(&[(-36.0 + k as f32 * 3.0, -98.0), (-52.0, -94.0 + k as f32 * 2.0), (-35.0 + k as f32 * 3.0, -88.0)]), Color::hex(0xff7043));
    // The old gentleman: flat cap, blanket over his knees.
    p.fill(&PathData::round_rect(-20.0, -60.0, 20.0, 26.0, 7.0), Color::hex(0x8d6e63));
    p.fill(&PathData::circle(-8.0, -66.0, 8.5), SKIN);
    p.fill(&PathData::ellipse(-7.0, -72.0, 10.0, 4.0), Color::hex(0x5d6d58));
    p.fill(&PathData::ellipse(-1.0, -64.0, 3.0, 1.6), Color::hex(0xdddddd));
    p.inked(&PathData::round_rect(-26.0, -40.0, 30.0, 12.0, 5.0), Color::hex(0x7b8fd6), INK, 1.0);
    p.inked(&PathData::round_rect(-32.0, -30.0, 64.0, 14.0, 6.0), red, INK, 1.3);
    let mut tiller = PathData::new();
    tiller.move_to(20.0, -28.0).line_to(14.0, -56.0).line_to(6.0, -56.0);
    p.stroke(&tiller, INK, 3.0);
    // The basket, with a baguette.
    p.inked(&PathData::round_rect(18.0, -44.0, 16.0, 13.0, 3.0), Color::hex(0xd7a86e), INK, 1.0);
    let mut bread = PathData::new();
    bread.move_to(22.0, -40.0).line_to(30.0, -60.0);
    p.stroke(&bread, Color::hex(0x8d5a2b), 5.0);
    p.stroke(&bread, Color::hex(0xe0a85a), 3.4);
    wheel(p, -22.0, 7.0, spin);
    wheel(p, 24.0, 7.0, spin);
}

fn bathtub(p: &mut Painter, k: u32) {
    let spin = k as f32;
    ground_shadow(p, 54.0);
    // Bubbles streaming out behind.
    for (i, (x, y, r)) in [(-60.0f32, -48.0f32, 4.0f32), (-70.0, -58.0, 3.0), (-64.0, -70.0, 2.4)].into_iter().enumerate() {
        let dy = if (i as u32 + k) % 2 == 0 { 0.0 } else { -4.0 };
        p.stroke(&PathData::circle(x, y + dy, r), Color::hex(0x9fd6f0), 1.2);
        p.fill(&PathData::circle(x - r * 0.3, y + dy - r * 0.3, r * 0.3), Color::rgb(255, 255, 255));
    }
    // The bather: shower cap, scrubbing brush, rubber duck.
    p.fill(&PathData::circle(-6.0, -58.0, 10.0), SKIN);
    p.inked(&PathData::ellipse(-7.0, -66.0, 12.0, 6.0), Color::hex(0xf48fb1), INK, 0.8);
    p.fill(&PathData::circle(-2.0, -59.0, 1.4), INK);
    let mut brush = PathData::new();
    brush.move_to(6.0, -54.0).line_to(20.0, -72.0 - k as f32 * 3.0);
    p.stroke(&brush, Color::hex(0xa1887f), 2.4);
    p.fill(&PathData::ellipse(22.0, -75.0 - k as f32 * 3.0, 5.0, 3.0), Color::hex(0xffcc80));
    // Foam on top.
    for x in [-36.0f32, -26.0, 10.0, 22.0, 32.0] {
        p.fill(&PathData::circle(x, -48.0, 7.0), Color::hex(0xffffff));
        p.stroke(&PathData::circle(x, -48.0, 7.0), Color::hex(0xb3e5fc), 1.0);
    }
    let tub = PathData::smooth_closed(&[(-48.0, -48.0), (46.0, -48.0), (40.0, -22.0), (24.0, -14.0), (-28.0, -14.0), (-44.0, -22.0)], 0.4);
    p.inked(&tub, Color::hex(0xfdfdfd), INK, 1.4);
    let mut rim = PathData::new();
    rim.move_to(-50.0, -48.0).line_to(48.0, -48.0);
    p.stroke(&rim, INK, 4.0);
    p.stroke(&rim, CHROME, 2.4);
    // Taps at the front, and a duck riding the rim.
    p.fill(&PathData::round_rect(40.0, -58.0, 4.0, 10.0, 1.0), Color::hex(0xd4af37));
    p.inked(&PathData::circle(34.0, -56.0, 5.0), Color::hex(0xffd54f), INK, 0.8);
    p.fill(&PathData::polygon(&[(38.0, -57.0), (43.0, -56.0), (38.0, -54.0)]), Color::hex(0xff8f00));
    // Claw feet on wheels.
    for x in [-30.0f32, 26.0] {
        p.fill(&PathData::polygon(&[(x - 4.0, -16.0), (x + 4.0, -16.0), (x + 2.0, -10.0), (x - 2.0, -10.0)]), Color::hex(0xd4af37));
    }
    wheel(p, -30.0, 8.0, spin);
    wheel(p, 26.0, 8.0, spin);
}

fn mower(p: &mut Painter, k: u32) {
    let spin = k as f32;
    ground_shadow(p, 40.0);
    // Clippings flying out of the side.
    for (i, (x, y)) in [(-44.0f32, -14.0f32), (-50.0, -22.0), (-40.0, -26.0)].into_iter().enumerate() {
        let dy = if (i as u32 + k) % 2 == 0 { 0.0 } else { -5.0 };
        p.fill(&PathData::ellipse(x, y + dy, 3.0, 1.4), Color::hex(0x7cc35a));
    }
    let red = Color::hex(0xe24b36);
    // The gardener in a straw hat.
    p.fill(&PathData::round_rect(-20.0, -58.0, 20.0, 24.0, 7.0), Color::hex(0x8bc34a));
    p.fill(&PathData::circle(-10.0, -64.0, 8.0), SKIN);
    p.inked(&PathData::ellipse(-10.0, -70.0, 15.0, 3.6), Color::hex(0xf2d27b), INK, 0.8);
    p.inked(&PathData::ellipse(-10.0, -75.0, 7.0, 6.0), Color::hex(0xf2d27b), INK, 0.8);
    p.fill(&PathData::rect(-16.0, -72.0, 12.0, 2.0), Color::hex(0xe53935));
    let body = PathData::smooth_closed(&[(-36.0, -36.0), (22.0, -38.0), (38.0, -26.0), (36.0, -12.0), (-36.0, -12.0)], 0.4);
    p.inked(&body, red, INK, 1.3);
    p.inked(&PathData::round_rect(-30.0, -44.0, 22.0, 8.0, 3.0), Color::hex(0x333333), INK, 0.8);
    let mut wheel_col = PathData::new();
    wheel_col.move_to(14.0, -38.0).line_to(8.0, -54.0);
    p.stroke(&wheel_col, INK, 2.4);
    p.stroke(&PathData::ellipse(6.0, -56.0, 6.0, 2.6), INK, 2.0);
    p.fill(&PathData::ellipse(37.0, -24.0, 2.0, 3.0), LAMP);
    wheel(p, -24.0, 10.0, spin);
    wheel(p, 26.0, 8.0, spin);
}

const CUSTARD: Color = Color::hex(0xf6d36b);
const CUSTARD_DARK: Color = Color::hex(0xd6a933);
const MAROON: Color = Color::hex(0x7d2b3a);

fn engine(p: &mut Painter, k: u32) {
    let spin = k as f32 * 1.3;
    ground_shadow(p, 84.0);
    // Steam.
    for (i, (x, y, r)) in [(44.0f32, -96.0f32, 9.0f32), (30.0, -106.0, 11.0), (12.0, -112.0, 13.0)].into_iter().enumerate() {
        let dy = if (i as u32 + k) % 2 == 0 { 0.0 } else { -4.0 };
        p.fill(&PathData::circle(x, y + dy, r), Color::rgba(255, 255, 255, 200 - i as u8 * 40));
    }
    // Boiler, chimney, dome.
    let boiler = PathData::round_rect(-22.0, -70.0, 98.0, 40.0, 18.0);
    p.inked(&boiler, CUSTARD, INK, 1.5);
    for x in [0.0f32, 26.0, 52.0] {
        p.fill(&PathData::rect(x, -70.0, 4.0, 40.0), CUSTARD_DARK);
    }
    p.inked(&PathData::round_rect(50.0, -92.0, 14.0, 24.0, 3.0), Color::hex(0x3a3440), INK, 1.2);
    p.inked(&PathData::round_rect(47.0, -96.0, 20.0, 6.0, 2.0), Color::hex(0x3a3440), INK, 1.0);
    p.inked(&PathData::ellipse(20.0, -74.0, 10.0, 7.0), Color::hex(0xd4af37), INK, 1.0);
    // Smokebox front with a smiley face.
    p.inked(&PathData::round_rect(70.0, -70.0, 12.0, 40.0, 6.0), Color::hex(0x3a3440), INK, 1.2);
    // Cab.
    p.inked(&PathData::rect(-80.0, -86.0, 60.0, 62.0), MAROON, INK, 1.5);
    p.inked(&PathData::round_rect(-86.0, -94.0, 72.0, 10.0, 4.0), Color::hex(0x3a3440), INK, 1.2);
    window(p, &PathData::round_rect(-70.0, -78.0, 22.0, 20.0, 4.0));
    // The driver, a hen in an engineer's cap: this is her line.
    p.fill(&PathData::circle(-58.0, -66.0, 7.0), Color::hex(0xfffdf7));
    p.fill(&PathData::ellipse(-58.0, -73.0, 8.0, 3.0), Color::hex(0x3e64a8));
    p.fill(&PathData::polygon(&[(-51.0, -67.0), (-46.0, -65.0), (-51.0, -63.0)]), Color::hex(0x8191a8));
    p.inked(&PathData::rect(-86.0, -30.0, 172.0, 10.0), Color::hex(0x3a3440), INK, 1.2);
    // Cowcatcher.
    p.inked(&PathData::polygon(&[(80.0, -30.0), (92.0, -4.0), (72.0, -4.0)]), Color::hex(0xb71c1c), INK, 1.0);
    // Driving wheels and their coupling rod.
    for x in [-50.0f32, -14.0, 22.0] {
        wheel(p, x, 15.0, spin);
    }
    wheel(p, 58.0, 10.0, spin);
    let (dx, dy) = (spin.cos() * 7.0, spin.sin() * 7.0);
    let mut rod = PathData::new();
    rod.move_to(-50.0 + dx, -15.0 + dy).line_to(22.0 + dx, -15.0 + dy);
    p.stroke(&rod, INK, 4.0);
    p.stroke(&rod, CHROME, 2.4);
}

fn carriage(p: &mut Painter, k: u32) {
    let spin = k as f32 * 1.3;
    ground_shadow(p, 76.0);
    let body = PathData::round_rect(-74.0, -84.0, 148.0, 60.0, 6.0);
    p.inked(&body, CUSTARD, INK, 1.5);
    p.fill(&PathData::rect(-74.0, -40.0, 148.0, 16.0), MAROON);
    let roof = PathData::round_rect(-78.0, -92.0, 156.0, 12.0, 6.0);
    p.inked(&roof, Color::hex(0x6d6875), INK, 1.2);
    // Passengers: hens on a day out, with the occasional duck.
    for (i, x) in [-58.0f32, -22.0, 14.0, 50.0].into_iter().enumerate() {
        window(p, &PathData::round_rect(x - 12.0, -76.0, 24.0, 26.0, 5.0));
        let duck = i == 2;
        let c = if duck { Color::hex(0xffd54f) } else { Color::hex(0xfffdf7) };
        p.fill(&PathData::circle(x, -58.0, 7.0), c);
        p.fill(&PathData::circle(x + 2.0, -60.0, 1.3), INK);
        let beak = if duck { Color::hex(0xff8f00) } else { Color::hex(0x8191a8) };
        p.fill(&PathData::polygon(&[(x + 6.0, -59.0), (x + 11.0, -57.0), (x + 6.0, -55.0)]), beak);
    }
    p.inked(&PathData::rect(-78.0, -24.0, 156.0, 8.0), Color::hex(0x3a3440), INK, 1.0);
    for x in [-50.0f32, -30.0, 30.0, 50.0] {
        wheel(p, x, 10.0, spin);
    }
}

/// A woolly passenger at a window.
fn sheep(p: &mut Painter, x: f32, y: f32, k: u32) {
    let wool = super::paint::Cloud::new(x, y, 7.5, 6.5, 6, 0.4, (x.abs() * 3.0 + y.abs()) as u64);
    p.cloud(&wool, INK, 0.8, Color::hex(0xf7f4ec));
    let look = if k == 0 { 2.0 } else { 3.0 };
    p.fill(&PathData::ellipse(x + look, y + 1.0, 3.6, 4.4), Color::hex(0x3a3238));
    p.fill(&PathData::circle(x + look + 1.4, y, 1.0), Color::rgb(255, 255, 255));
}

fn bus(p: &mut Painter, k: u32) {
    let spin = k as f32;
    ground_shadow(p, 90.0);
    let red = Color::hex(0xd63a2f);
    let bob = if k == 1 { -0.8 } else { 0.0 };
    p.push(Affine::translate(0.0, bob));
    let body = PathData::round_rect(-88.0, -120.0, 176.0, 106.0, 12.0);
    p.inked(&body, red, INK, 1.6);
    p.fill(&PathData::rect(-88.0, -72.0, 176.0, 7.0), Color::hex(0xf2e6c8));
    p.fill(&PathData::rect(-88.0, -24.0, 176.0, 6.0), red.shade(-0.25));
    // Upper and lower decks, every seat taken by a sheep.
    for (row, y) in [(0u32, -110.0f32), (1, -62.0)] {
        for i in 0..5 {
            let x = -76.0 + i as f32 * 30.0;
            if row == 1 && i == 4 {
                continue;
            }
            window(p, &PathData::round_rect(x, y, 24.0, 28.0, 4.0));
            sheep(p, x + 12.0, y + 16.0, (k + i) % 2);
        }
    }
    // The driver's door and windscreen at the front.
    window(p, &PathData::round_rect(64.0, -62.0, 20.0, 34.0, 4.0));
    p.fill(&PathData::circle(73.0, -46.0, 6.0), SKIN);
    p.fill(&PathData::ellipse(73.0, -52.0, 7.0, 2.6), Color::hex(0x2e3a59));
    // A destination blind (no words: just a sheep).
    p.inked(&PathData::round_rect(48.0, -118.0, 34.0, 8.0, 2.0), Color::hex(0x1f1f24), INK, 0.6);
    p.fill(&PathData::circle(65.0, -114.0, 2.6), Color::hex(0xffd54f));
    p.fill(&PathData::ellipse(86.0, -26.0, 2.6, 4.0), LAMP);
    p.fill(&PathData::ellipse(-87.0, -26.0, 2.4, 4.0), TAIL);
    p.pop();
    wheel(p, -56.0, 13.0, spin);
    wheel(p, 58.0, 13.0, spin);
}

fn tandem(p: &mut Painter, k: u32) {
    let spin = k as f32 * 1.4;
    ground_shadow(p, 52.0);
    // Wheels: big and spoked.
    for x in [-38.0f32, 40.0] {
        p.stroke(&PathData::circle(x, -15.0, 15.0), INK, 3.6);
        p.stroke(&PathData::circle(x, -15.0, 15.0), Color::hex(0x55505a), 2.0);
        let mut spokes = PathData::new();
        for i in 0..6 {
            let a = spin + i as f32 * std::f32::consts::PI / 3.0;
            spokes.move_to(x, -15.0).line_to(x + a.cos() * 14.0, -15.0 + a.sin() * 14.0);
        }
        p.stroke(&spokes, Color::hex(0xb0b4bb), 0.9);
    }
    // The long frame.
    let mut frame = PathData::new();
    frame
        .move_to(-38.0, -15.0)
        .line_to(-22.0, -40.0)
        .line_to(18.0, -40.0)
        .line_to(40.0, -15.0)
        .move_to(-22.0, -40.0)
        .line_to(-14.0, -16.0)
        .line_to(18.0, -40.0)
        .move_to(-14.0, -16.0)
        .line_to(14.0, -16.0)
        .line_to(18.0, -40.0)
        .move_to(34.0, -46.0)
        .line_to(40.0, -15.0);
    p.stroke(&frame, INK, 4.0);
    p.stroke(&frame, Color::hex(0x2e86de), 2.4);
    // Two riders, pedalling out of step.
    let riders = [(-22.0f32, Color::hex(0xff7043), true), (16.0, Color::hex(0x66bb6a), false)];
    for (i, (x, shirt, plaits)) in riders.into_iter().enumerate() {
        let pedal = if (k + i as u32) % 2 == 0 { 6.0 } else { -6.0 };
        let mut legs = PathData::new();
        legs.move_to(x, -48.0).line_to(x + 6.0, -30.0).line_to(x + 4.0 + pedal * 0.4, -16.0 + pedal * 0.5);
        p.stroke(&legs, INK, 5.0);
        p.stroke(&legs, Color::hex(0x37474f), 3.4);
        let lean = PathData::round_rect(x - 6.0, -68.0, 14.0, 22.0, 6.0);
        p.inked(&lean, shirt, INK, 1.0);
        let mut arm = PathData::new();
        arm.move_to(x + 4.0, -62.0).line_to(x + 18.0, -48.0);
        p.stroke(&arm, INK, 4.0);
        p.stroke(&arm, shirt, 2.6);
        p.fill(&PathData::circle(x + 3.0, -76.0, 7.0), SKIN);
        p.inked(&PathData::ellipse(x + 2.0, -80.0, 8.0, 5.0), Color::hex(0xfdd835), INK, 0.8);
        if plaits {
            let mut pl = PathData::new();
            pl.move_to(x - 4.0, -78.0).quad_to(x - 10.0, -72.0 + pedal * 0.3, x - 12.0, -66.0);
            p.stroke(&pl, Color::hex(0x8d5a2b), 2.6);
        } else {
            p.fill(&PathData::ellipse(x + 6.0, -71.0, 4.0, 3.0), Color::hex(0x6d4c41));
        }
    }
    p.fill(&PathData::ellipse(-26.0, -42.0, 6.0, 2.4), INK);
    p.fill(&PathData::ellipse(12.0, -42.0, 6.0, 2.4), INK);
}

fn camper(p: &mut Painter, k: u32) {
    let spin = k as f32;
    let bob = if k == 1 { -1.0 } else { 0.0 };
    ground_shadow(p, 70.0);
    let teal = Color::hex(0x4fb3a9);
    let cream = Color::hex(0xfbf3df);
    p.push(Affine::translate(0.0, bob));
    // A surfboard on the roof.
    let board = PathData::smooth_closed(&[(-70.0, -102.0), (-20.0, -108.0), (56.0, -104.0), (66.0, -99.0), (-60.0, -96.0)], 0.6);
    p.inked(&board, Color::hex(0xffb74d), INK, 1.2);
    p.fill(&PathData::rect(-30.0, -105.0, 4.0, 8.0), Color::hex(0xe53935));
    let top = PathData::smooth_closed(&[(-70.0, -60.0), (-68.0, -90.0), (40.0, -94.0), (66.0, -66.0), (70.0, -60.0)], 0.3);
    p.inked(&top, cream, INK, 1.4);
    let bottom = PathData::round_rect(-72.0, -62.0, 144.0, 48.0, 10.0);
    p.inked(&bottom, teal, INK, 1.4);
    // The V on the nose and a round headlamp.
    let mut v = PathData::new();
    v.move_to(58.0, -60.0).line_to(70.0, -40.0).line_to(72.0, -60.0);
    p.stroke(&v, cream, 4.0);
    p.inked(&PathData::circle(66.0, -32.0, 5.0), LAMP, INK, 1.0);
    for x in [-60.0f32, -30.0, 0.0] {
        window(p, &PathData::round_rect(x, -86.0, 24.0, 20.0, 4.0));
    }
    window(p, &PathData::polygon(&[(32.0, -86.0), (50.0, -86.0), (62.0, -66.0), (32.0, -66.0)]));
    // Curtains, and a big flower on the side.
    p.fill(&PathData::rect(-60.0, -86.0, 5.0, 20.0), Color::hex(0xff8a65));
    p.fill(&PathData::rect(-11.0, -86.0, 5.0, 20.0), Color::hex(0xff8a65));
    for i in 0..5 {
        let a = i as f32 * std::f32::consts::TAU / 5.0;
        p.fill(&PathData::circle(-30.0 + a.cos() * 7.0, -40.0 + a.sin() * 7.0, 5.0), Color::hex(0xfff176));
    }
    p.fill(&PathData::circle(-30.0, -40.0, 4.0), Color::hex(0xff7043));
    p.fill(&PathData::ellipse(-71.0, -28.0, 2.4, 4.0), TAIL);
    p.pop();
    wheel(p, -44.0, 12.0, spin);
    wheel(p, 46.0, 12.0, spin);
}

fn milk_float(p: &mut Painter, k: u32) {
    let spin = k as f32 * 0.6;
    ground_shadow(p, 60.0);
    let blue = Color::hex(0x3d6fb6);
    // The flatbed of crates and bottles.
    p.inked(&PathData::rect(-58.0, -30.0, 88.0, 14.0), blue, INK, 1.3);
    for c in 0..3 {
        let x = -54.0 + c as f32 * 28.0;
        p.inked(&PathData::rect(x, -52.0, 26.0, 22.0), Color::hex(0xffca28), INK, 1.0);
        for b in 0..3 {
            let bx = x + 5.0 + b as f32 * 8.0;
            let jiggle = if (k + b + c) % 2 == 0 { 0.0 } else { -1.5 };
            let bottle = PathData::round_rect(bx - 3.0, -62.0 + jiggle, 6.0, 12.0, 2.0);
            p.inked(&bottle, Color::hex(0xfdfdfd), INK, 0.6);
            p.fill(&PathData::rect(bx - 3.0, -63.0 + jiggle, 6.0, 3.0), Color::hex(0xc0c4ca));
        }
    }
    // The cab, with the milkman in his cap.
    let cab = PathData::round_rect(30.0, -76.0, 34.0, 62.0, 8.0);
    p.inked(&cab, Color::hex(0xf4f4f0), INK, 1.4);
    window(p, &PathData::round_rect(36.0, -70.0, 24.0, 22.0, 4.0));
    p.fill(&PathData::circle(46.0, -58.0, 6.5), SKIN);
    p.fill(&PathData::ellipse(46.0, -64.0, 8.0, 3.0), blue);
    p.fill(&PathData::ellipse(62.0, -24.0, 2.4, 3.6), LAMP);
    p.inked(&PathData::rect(28.0, -80.0, 38.0, 6.0), blue, INK, 1.0);
    wheel(p, -42.0, 9.0, spin);
    wheel(p, 46.0, 9.0, spin);
}

fn unicycle(p: &mut Painter, k: u32) {
    let spin = k as f32 * 1.2;
    ground_shadow(p, 18.0);
    p.stroke(&PathData::circle(0.0, -14.0, 14.0), INK, 3.6);
    p.stroke(&PathData::circle(0.0, -14.0, 14.0), Color::hex(0xe53935), 2.2);
    let mut spokes = PathData::new();
    for i in 0..4 {
        let a = spin + i as f32 * std::f32::consts::FRAC_PI_2;
        spokes.move_to(0.0, -14.0).line_to(a.cos() * 13.0, -14.0 + a.sin() * 13.0);
    }
    p.stroke(&spokes, Color::hex(0xb0b4bb), 1.0);
    let mut post = PathData::new();
    post.move_to(0.0, -14.0).line_to(0.0, -44.0);
    p.stroke(&post, INK, 3.0);
    // The clown: baggy spotted suit, ruff, red nose, juggling.
    let pedal = if k == 0 { 5.0 } else { -5.0 };
    let mut legs = PathData::new();
    legs.move_to(-4.0, -48.0).line_to(-6.0 + pedal * 0.5, -20.0).move_to(4.0, -48.0).line_to(6.0 - pedal * 0.5, -20.0);
    p.stroke(&legs, INK, 6.0);
    p.stroke(&legs, Color::hex(0x7e57c2), 4.4);
    let body = PathData::round_rect(-11.0, -76.0, 22.0, 30.0, 9.0);
    p.inked(&body, Color::hex(0xfdd835), INK, 1.2);
    for (x, y) in [(-4.0f32, -66.0f32), (5.0, -58.0), (-5.0, -52.0)] {
        p.fill(&PathData::circle(x, y, 2.4), Color::hex(0xe53935));
    }
    let ruff = super::paint::Cloud::new(0.0, -78.0, 13.0, 4.5, 8, 0.45, 61);
    p.cloud(&ruff, INK, 0.8, Color::rgb(255, 255, 255));
    p.fill(&PathData::circle(0.0, -88.0, 9.0), Color::hex(0xfdfdfd));
    p.stroke(&PathData::circle(0.0, -88.0, 9.0), INK, 1.0);
    p.fill(&PathData::circle(4.0, -87.0, 3.0), Color::hex(0xe53935));
    p.fill(&PathData::circle(-6.0, -95.0, 5.0), Color::hex(0xff7043));
    p.fill(&PathData::circle(8.0, -95.0, 5.0), Color::hex(0xff7043));
    let mut smile = PathData::new();
    smile.move_to(-3.0, -83.0).quad_to(2.0, -79.0, 7.0, -83.0);
    p.stroke(&smile, Color::hex(0xc62828), 1.4);
    let mut arms = PathData::new();
    arms.move_to(-9.0, -70.0).line_to(-18.0, -80.0).move_to(9.0, -70.0).line_to(18.0, -80.0);
    p.stroke(&arms, INK, 4.0);
    p.stroke(&arms, Color::hex(0xfdd835), 2.6);
    let balls: [(f32, f32); 3] = if k == 0 { [(-18.0, -92.0), (0.0, -112.0), (18.0, -96.0)] } else { [(-16.0, -100.0), (6.0, -108.0), (18.0, -88.0)] };
    for (i, (x, y)) in balls.into_iter().enumerate() {
        let c = [Color::hex(0x42a5f5), Color::hex(0x66bb6a), Color::hex(0xffca28)][i];
        p.inked(&PathData::circle(x, y, 3.6), c, INK, 0.7);
    }
}

fn spoked(p: &mut Painter, x: f32, r: f32, spin: f32, rim: Color) {
    p.stroke(&PathData::circle(x, -r, r), INK, 3.4);
    p.stroke(&PathData::circle(x, -r, r), rim, 2.0);
    let mut spokes = PathData::new();
    for i in 0..6 {
        let a = spin + i as f32 * std::f32::consts::PI / 3.0;
        spokes.move_to(x, -r).line_to(x + a.cos() * (r - 1.0), -r + a.sin() * (r - 1.0));
    }
    p.stroke(&spokes, Color::hex(0xb0b4bb), 0.9);
}

fn bicycle(p: &mut Painter, k: u32) {
    let spin = k as f32 * 1.4;
    ground_shadow(p, 34.0);
    spoked(p, -24.0, 14.0, spin, Color::hex(0x55505a));
    spoked(p, 26.0, 14.0, spin, Color::hex(0x55505a));
    let mut frame = PathData::new();
    frame
        .move_to(-24.0, -14.0)
        .line_to(-8.0, -36.0)
        .line_to(18.0, -36.0)
        .line_to(26.0, -14.0)
        .move_to(-8.0, -36.0)
        .line_to(0.0, -14.0)
        .line_to(18.0, -36.0)
        .move_to(-24.0, -14.0)
        .line_to(0.0, -14.0)
        .move_to(20.0, -44.0)
        .line_to(26.0, -14.0);
    p.stroke(&frame, INK, 4.0);
    p.stroke(&frame, Color::hex(0xe91e63), 2.4);
    // A wicker basket with a baguette and a small terrier in it.
    p.inked(&PathData::round_rect(22.0, -54.0, 18.0, 13.0, 3.0), Color::hex(0xd7a86e), INK, 1.0);
    p.fill(&PathData::circle(31.0, -58.0, 5.0), Color::hex(0xf5f0e6));
    p.fill(&PathData::circle(33.0, -59.0, 1.2), INK);
    p.fill(&PathData::ellipse(28.0, -62.0, 2.0, 3.6), Color::hex(0x8d6e63));
    // The rider, pedalling.
    let pedal = if k == 0 { 6.0 } else { -6.0 };
    let mut legs = PathData::new();
    legs.move_to(-6.0, -46.0).line_to(2.0, -28.0).line_to(pedal * 0.6, -16.0 + pedal * 0.4);
    p.stroke(&legs, INK, 5.0);
    p.stroke(&legs, Color::hex(0x455a64), 3.4);
    p.inked(&PathData::round_rect(-14.0, -68.0, 16.0, 24.0, 7.0), Color::hex(0x26a69a), INK, 1.0);
    let mut arm = PathData::new();
    arm.move_to(-2.0, -62.0).line_to(18.0, -46.0);
    p.stroke(&arm, INK, 4.0);
    p.stroke(&arm, Color::hex(0x26a69a), 2.6);
    p.fill(&PathData::circle(-4.0, -76.0, 7.5), SKIN);
    p.inked(&PathData::ellipse(-5.0, -81.0, 9.0, 5.0), Color::hex(0xff7043), INK, 0.8);
    p.fill(&PathData::ellipse(-6.0, -75.0, 2.0, 1.4), Color::hex(0x6d4c41));
    p.fill(&PathData::ellipse(-14.0, -38.0, 6.0, 2.4), INK);
}

fn stroller(p: &mut Painter, k: u32) {
    let spin = k as f32;
    ground_shadow(p, 30.0);
    // A dad in a striped scarf, pushing at a brisk walk.
    let stride = if k == 0 { 8.0 } else { -8.0 };
    let mut legs = PathData::new();
    legs.move_to(-20.0, -40.0).line_to(-20.0 + stride, -2.0).move_to(-20.0, -40.0).line_to(-20.0 - stride, -2.0);
    p.stroke(&legs, INK, 6.0);
    p.stroke(&legs, Color::hex(0x5d6d7e), 4.4);
    p.inked(&PathData::round_rect(-30.0, -78.0, 20.0, 40.0, 8.0), Color::hex(0xa1887f), INK, 1.2);
    let mut scarf = PathData::new();
    scarf.move_to(-28.0, -74.0).line_to(-12.0, -74.0).move_to(-22.0, -74.0).line_to(-30.0, -60.0);
    p.stroke(&scarf, Color::hex(0xe53935), 3.6);
    let mut arm = PathData::new();
    arm.move_to(-14.0, -70.0).line_to(-2.0, -56.0);
    p.stroke(&arm, INK, 4.4);
    p.stroke(&arm, Color::hex(0xa1887f), 3.0);
    p.fill(&PathData::circle(-20.0, -86.0, 8.0), SKIN);
    p.fill(&PathData::ellipse(-21.0, -92.0, 8.0, 3.6), Color::hex(0x4e342e));
    p.fill(&PathData::ellipse(-15.0, -82.0, 4.0, 3.0), Color::hex(0x4e342e));
    // The pram: a big hood, a baby waving a rattle.
    let mut handle = PathData::new();
    handle.move_to(-2.0, -56.0).line_to(6.0, -28.0);
    p.stroke(&handle, INK, 2.6);
    let body = PathData::smooth_closed(&[(4.0, -40.0), (36.0, -40.0), (34.0, -18.0), (8.0, -18.0)], 0.4);
    p.inked(&body, Color::hex(0x5c6bc0), INK, 1.2);
    let hood = PathData::smooth_closed(&[(4.0, -40.0), (8.0, -58.0), (22.0, -60.0), (22.0, -40.0)], 0.6);
    p.inked(&hood, Color::hex(0x3949ab), INK, 1.2);
    p.fill(&PathData::circle(28.0, -45.0, 6.0), SKIN);
    p.fill(&PathData::circle(30.0, -46.0, 1.0), INK);
    let rattle = if k == 0 { -56.0 } else { -60.0 };
    let mut r = PathData::new();
    r.move_to(32.0, -44.0).line_to(38.0, rattle + 4.0);
    p.stroke(&r, INK, 1.4);
    p.inked(&PathData::circle(38.0, rattle, 3.4), Color::hex(0xffca28), INK, 0.6);
    wheel(p, 10.0, 6.0, spin);
    wheel(p, 32.0, 6.0, spin);
}

fn wagon(p: &mut Painter, k: u32) {
    let spin = k as f32;
    ground_shadow(p, 44.0);
    // The wagon and its passenger, waving.
    let wave = if k == 0 { -6.0 } else { 2.0 };
    p.fill(&PathData::round_rect(-34.0, -44.0, 16.0, 22.0, 6.0), Color::hex(0xffeb3b));
    p.fill(&PathData::circle(-26.0, -52.0, 7.5), SKIN);
    p.fill(&PathData::ellipse(-27.0, -58.0, 8.0, 4.0), Color::hex(0x3e2723));
    let mut arm = PathData::new();
    arm.move_to(-20.0, -44.0).line_to(-12.0, -58.0 + wave);
    p.stroke(&arm, INK, 4.0);
    p.stroke(&arm, Color::hex(0xffeb3b), 2.6);
    let bed = PathData::round_rect(-44.0, -30.0, 40.0, 16.0, 3.0);
    p.inked(&bed, Color::hex(0xe53935), INK, 1.3);
    p.fill(&PathData::rect(-42.0, -28.0, 36.0, 3.0), Color::hex(0xff8a80));
    wheel(p, -36.0, 6.0, spin);
    wheel(p, -12.0, 6.0, spin);
    // The handle, and the big sister pulling it at a run.
    let mut handle = PathData::new();
    handle.move_to(-4.0, -22.0).line_to(16.0, -40.0);
    p.stroke(&handle, INK, 2.6);
    let stride = if k == 0 { 9.0 } else { -9.0 };
    let mut legs = PathData::new();
    legs.move_to(24.0, -36.0).line_to(24.0 + stride, -2.0).move_to(24.0, -36.0).line_to(24.0 - stride, -2.0);
    p.stroke(&legs, INK, 5.0);
    p.stroke(&legs, SKIN, 3.4);
    let dress = PathData::polygon(&[(18.0, -64.0), (30.0, -64.0), (36.0, -34.0), (12.0, -34.0)]);
    p.inked(&dress, Color::hex(0x7e57c2), INK, 1.1);
    let mut pull = PathData::new();
    pull.move_to(20.0, -58.0).line_to(14.0, -42.0);
    p.stroke(&pull, INK, 3.8);
    p.stroke(&pull, SKIN, 2.4);
    p.fill(&PathData::circle(25.0, -72.0, 8.0), SKIN);
    let mut pony = PathData::new();
    pony.move_to(19.0, -76.0).quad_to(10.0, -78.0 + stride * 0.3, 8.0, -70.0);
    p.stroke(&pony, Color::hex(0xf9a825), 4.0);
    p.fill(&PathData::ellipse(25.0, -78.0, 8.5, 4.0), Color::hex(0xf9a825));
    p.fill(&PathData::circle(29.0, -72.0, 1.3), INK);
}

/// Every road vehicle facing both ways: animations `{name}_{right|left}`.
pub fn sheet(tile_px: u32) -> (SheetSpec, Pixmap) {
    let mut sb = SheetBuilder::new("cars", (192.0, 144.0), (96.0, 128.0), tile_px);
    type Draw = fn(&mut Painter, u32);
    let kinds: [(&str, Draw); 17] = [
        ("hatchback", hatchback),
        ("tractor", tractor),
        ("icecream", ice_cream),
        ("sofa", sofa),
        ("scooter", scooter),
        ("bathtub", bathtub),
        ("mower", mower),
        ("engine", engine),
        ("carriage", carriage),
        ("bus", bus),
        ("tandem", tandem),
        ("camper", camper),
        ("milkfloat", milk_float),
        ("unicycle", unicycle),
        ("bicycle", bicycle),
        ("stroller", stroller),
        ("wagon", wagon),
    ];
    for (dir, left) in [("right", false), ("left", true)] {
        for (name, draw) in kinds {
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
        for (part, name) in ["limo_nose", "limo_mid", "limo_tail"].into_iter().enumerate() {
            for k in 0..2 {
                sb.frame(&format!("{name}_{dir}"), |p| {
                    if left {
                        p.push(mirror());
                    }
                    limo(p, part as u32, k);
                    if left {
                        p.pop();
                    }
                });
            }
        }
    }
    sb.finish()
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_vehicle_has_both_directions() {
        let (spec, _) = super::sheet(32);
        for name in [
            "hatchback", "tractor", "icecream", "sofa", "scooter", "bathtub", "mower", "engine", "carriage", "limo_nose", "limo_mid", "limo_tail", "bus",
            "tandem", "camper", "milkfloat", "unicycle", "bicycle", "stroller", "wagon",
        ] {
            for dir in ["right", "left"] {
                assert_eq!(spec.anims[&format!("{name}_{dir}")].len(), 2, "{name}_{dir}");
            }
        }
    }
}
