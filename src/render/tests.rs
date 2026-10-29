//! Renderer tests: the bytes it would write are decoded and checked
//! against what a terminal would then hold.

use super::*;
use crate::render::text::{Anchor, Line, Style};
use crate::world::level;

fn renderer(vector: bool) -> Renderer {
    let mut r = Renderer::new(RenderConfig {
        vector_scenery: vector,
        uploader: Uploader {
            temp_file: false,
            gamut_p3: false,
        },
        sync_output: true,
        cell_alpha: vector,
        hdr_boost: 1.0,
    });
    let l = level::builtin(0).unwrap();
    r.set_layout(Layout::new(120, 40, 10, 20, l.width, l.height()));
    r
}

fn scene() -> SceneRef {
    SceneRef {
        key: "lane".into(),
        level: Arc::new(level::builtin(0).unwrap()),
    }
}

fn hen(x: f32) -> Sprite {
    Sprite {
        key: 7,
        sheet: SheetId::Hen,
        anim: "walk_right".into(),
        frame: 0,
        pos: Vec2::new(x, 6.8),
        layer: Layer::Body,
        z_bias: 0,
    }
}

fn car(x: f32) -> Sprite {
    Sprite {
        key: 9,
        sheet: SheetId::Cars,
        anim: "hatchback_right".into(),
        frame: 0,
        pos: Vec2::new(x, 4.8),
        layer: Layer::Body,
        z_bias: 0,
    }
}

/// Graphics commands in a byte stream, as their control strings.
fn commands(bytes: &[u8]) -> Vec<String> {
    let s = String::from_utf8_lossy(bytes);
    s.split("\x1b_G")
        .skip(1)
        .map(|c| c.split([';', '\x1b']).next().unwrap_or("").to_string())
        .collect()
}

fn decals(n: usize) -> Vec<u8> {
    vec![0; n]
}

fn slots() -> usize {
    crate::gfx::scenery::decal_slots(&level::builtin(0).unwrap()).len()
}

#[test]
fn first_frame_uploads_then_only_moves() {
    let mut r = renderer(false);
    let mut f = FrameModel {
        scene: Some(scene()),
        decals: decals(slots()),
        sprites: vec![hen(3.5)],
        top: "top".into(),
        bottom: "bottom".into(),
        ..Default::default()
    };
    let first = r.render(&f);
    let cmds = commands(&first);
    let uploads = cmds.iter().filter(|c| c.starts_with("a=t")).count();
    assert_eq!(uploads, 4, "the hen's sheet (only what is on screen), ground, canopy, decals");
    assert!(r.is_uploaded(SheetId::Hen.image_id()) && !r.is_uploaded(SheetId::Cars.image_id()));
    // Text goes first and the frame is not held back by a synchronized
    // update while big uploads stream in.
    assert!(first.starts_with(b"\x1b[1;1H"));

    // Nothing changed: nothing is written.
    assert!(r.render(&f).is_empty());

    // The hen moves: exactly one placement, no upload.
    f.sprites = vec![hen(3.7)];
    let second = r.render(&f);
    let cmds = commands(&second);
    assert_eq!(cmds.len(), 1, "{cmds:?}");
    assert!(cmds[0].starts_with("a=p"));
    assert!(second.starts_with(b"\x1b[?2026h"));
}

#[test]
fn sprites_are_clipped_to_the_field() {
    let mut r = renderer(false);
    let l = r.layout.unwrap();
    let mut f = FrameModel {
        scene: Some(scene()),
        decals: decals(slots()),
        sprites: vec![car(0.2)],
        ..Default::default()
    };
    r.render(&f);
    assert_eq!(r.last.clipped, 1, "a car half off the left edge is clipped");
    // Its placement starts at the field's left edge, not before it.
    let (ox, _) = l.origin_px();
    let bytes = {
        f.sprites = vec![car(0.25)];
        r.render(&f)
    };
    let s = String::from_utf8_lossy(&bytes);
    let col: u32 = s.split("\x1b[").filter_map(|p| p.split_once('H')).filter_map(|(rc, _)| rc.split(';').nth(1)?.parse().ok()).next().unwrap();
    assert_eq!(col - 1, (ox as u32) / l.cell_w);
    // Entirely off the field: not placed at all.
    f.sprites = vec![car(-5.0)];
    r.render(&f);
    assert_eq!(r.placements_of(SheetId::Cars.image_id()), 0);
    // Fully inside: not clipped.
    f.sprites = vec![car(8.0)];
    r.render(&f);
    assert_eq!(r.last.clipped, 0);
}

#[test]
fn decals_patch_vectors_on_silkie_and_move_placements_elsewhere() {
    let n = slots();
    // Silkie: an egg appearing is a patch of the ground image, no upload.
    let mut r = renderer(true);
    let mut f = FrameModel {
        scene: Some(scene()),
        decals: decals(n),
        ..Default::default()
    };
    r.render(&f);
    f.decals[1] = 1;
    let bytes = r.render(&f);
    let cmds = commands(&bytes);
    assert!(cmds.iter().all(|c| !c.starts_with("a=t")), "{cmds:?}");
    assert!(cmds.iter().any(|c| c.contains("a=e")), "{cmds:?}");
    assert_eq!(r.last.vector_full_redraws, 0);

    // kitty: a different source rectangle of the decal atlas.
    let mut r = renderer(false);
    f.decals = decals(n);
    r.render(&f);
    f.decals[1] = 1;
    let bytes = r.render(&f);
    let cmds = commands(&bytes);
    assert_eq!(cmds.len(), 1, "{cmds:?}");
    assert!(cmds[0].starts_with(&format!("a=p,i={IMG_DECALS}")));
}

#[test]
fn vector_scenery_is_sent_once_and_survives_resizes() {
    let mut r = renderer(true);
    let f = FrameModel {
        scene: Some(scene()),
        decals: decals(slots()),
        ..Default::default()
    };
    let first = r.render(&f);
    assert!(commands(&first).iter().any(|c| c.contains("f=200")));
    let l = level::builtin(0).unwrap();
    r.set_layout(Layout::new(150, 50, 10, 20, l.width, l.height()));
    let second = r.render(&f);
    let cmds = commands(&second);
    assert!(cmds.iter().all(|c| c.starts_with("a=p")), "{cmds:?}");
}

#[test]
fn overlays_and_status_lines_are_diffed() {
    let mut r = renderer(false);
    let mut f = FrameModel {
        top: "hello".into(),
        overlays: vec![Overlay::new(Anchor::Center, vec![Line::new("Paused", Style::Heading)])],
        ..Default::default()
    };
    let a = r.render(&f);
    assert!(String::from_utf8_lossy(&a).contains("Paused"));
    assert!(r.render(&f).is_empty());
    f.overlays.clear();
    let b = String::from_utf8_lossy(&r.render(&f)).to_string();
    assert!(!b.contains("Paused"));
    assert!(b.contains("\x1b[0m "), "the old overlay is blanked");
}

#[test]
fn teardown_deletes_every_image() {
    let mut r = renderer(false);
    let f = FrameModel {
        scene: Some(scene()),
        decals: decals(slots()),
        sprites: vec![hen(3.0), car(6.0)],
        ..Default::default()
    };
    r.render(&f);
    let bytes = r.teardown();
    let deletes = commands(&bytes).iter().filter(|c| c.contains("a=d")).count();
    assert_eq!(deletes, 5);
}

#[test]
fn evicted_images_are_sent_again() {
    let mut r = renderer(false);
    let f = FrameModel {
        scene: Some(scene()),
        decals: decals(slots()),
        sprites: vec![hen(3.0)],
        ..Default::default()
    };
    r.render(&f);
    r.on_graphics_error(SheetId::Hen.image_id(), "ENOENT: no such image");
    let bytes = r.render(&f);
    assert!(commands(&bytes).iter().any(|c| c.starts_with("a=t") && c.contains(&format!("i={}", SheetId::Hen.image_id()))));
}

#[test]
fn every_sheet_paints_at_a_small_size() {
    for id in SheetId::ALL {
        let s = paint_sheet(id, 24);
        assert!(s.width > 0 && s.height > 0, "{id:?}");
        assert_eq!(s.rgba.len(), (s.width * s.height * 4) as usize);
    }
}

#[test]
fn every_sheet_fits_the_terminal_at_the_largest_size() {
    let largest = sprite_res(10_000.0);
    for id in SheetId::ALL {
        let s = paint_sheet(id, largest);
        assert!(
            s.width <= crate::art::MAX_IMAGE_SIDE && s.height <= crate::art::MAX_IMAGE_SIDE,
            "{id:?} is {}x{} at {largest} px per tile",
            s.width,
            s.height
        );
    }
}
