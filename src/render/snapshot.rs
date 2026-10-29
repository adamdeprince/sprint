//! A picture of a frame without a terminal: the scenery, decals and sprites
//! composited in the order a terminal would stack the placements. For
//! documentation, art review and tests.

use super::{Layer, Sprite, paint_sheet, sprite_res};
use crate::gfx::scenery::{self, TILE};
use crate::world::level::Level;
use std::collections::BTreeMap;
use tiny_skia::{IntRect, Pixmap, PixmapPaint, Transform};

/// Draws `sprites` over the level's scenery at `tile_px` pixels per tile.
pub fn snapshot(level: &Level, decals: &[u8], sprites: &[Sprite], tile_px: u32) -> Pixmap {
    let mut s = scenery::build(level);
    for (i, slot) in scenery::decal_slots(level).iter().enumerate() {
        for (id, e) in scenery::decal_elements(i, slot, decals.get(i).copied().unwrap_or(0)) {
            s.ground.set(id, e);
        }
    }
    let k = tile_px as f32 / TILE;
    let (w, h) = ((s.width * k).round() as u32, (s.height * k).round() as u32);
    let mut pm = Pixmap::new(w.max(1), h.max(1)).expect("snapshot size");
    let t = Transform::from_scale(k, k);
    s.ground.render(&mut pm, t);
    let res = sprite_res(tile_px as f32);
    let mut sheets = BTreeMap::new();
    // Stack as the renderer does: decals, bodies by foot, canopy, air by
    // foot, then everything over the top.
    let rank = |sp: &Sprite| {
        let foot = (sp.pos.y * tile_px as f32).round() as i64;
        match sp.layer {
            Layer::Decal => (0, sp.z_bias as i64, 0),
            Layer::Body => (1, foot * 8 + sp.z_bias as i64, 0),
            Layer::Air => (3, foot * 8 + sp.z_bias as i64, 0),
            Layer::Over => (4, sp.z_bias as i64, 0),
        }
    };
    let mut order: Vec<&Sprite> = sprites.iter().collect();
    order.sort_by_key(|s| (rank(s), s.key));
    let mut canopy_done = false;
    for sp in order {
        if rank(sp).0 >= 3 && !canopy_done {
            s.canopy.render(&mut pm, t);
            canopy_done = true;
        }
        let sheet = sheets.entry(sp.sheet).or_insert_with(|| {
            let sh = paint_sheet(sp.sheet, res);
            let mut img = Pixmap::new(sh.width, sh.height).expect("sheet");
            for (dst, src) in img.pixels_mut().iter_mut().zip(sh.rgba.chunks_exact(4)) {
                *dst = tiny_skia::ColorU8::from_rgba(src[0], src[1], src[2], src[3]).premultiply();
            }
            (sh, img)
        });
        let (sh, img) = (&sheet.0, &sheet.1);
        let Some(frames) = sh.anim(&sp.anim) else { continue };
        if frames.is_empty() {
            continue;
        }
        let [sx, sy, sw, shh] = sh.frame_rect(frames[sp.frame % frames.len()]);
        let Some(cell) = img.clone_rect(IntRect::from_xywh(sx as i32, sy as i32, sw, shh).expect("frame rect")) else { continue };
        let x = (sp.pos.x * tile_px as f32 - sh.spec.anchor_x).round() as i32;
        let y = (sp.pos.y * tile_px as f32 - sh.spec.anchor_y).round() as i32;
        pm.draw_pixmap(x, y, cell.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
    }
    if !canopy_done {
        s.canopy.render(&mut pm, t);
    }
    pm
}
