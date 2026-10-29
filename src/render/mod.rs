//! The renderer: turns a frame model (sprites, scenery state, status lines,
//! overlays) into terminal output.
//!
//! It keeps a model of what the terminal currently holds (uploaded images,
//! each placement and where it is, the text of each status line) and emits
//! only the difference each frame. Sprites are placements of uploaded
//! sheets, selected by source rectangle, so animation costs a placement
//! update and never a new upload. Sprites are clipped to the field (by
//! narrowing the source rectangle), so traffic slides in from the edges
//! rather than popping into view.
//!
//! The field goes through one of two scenery back ends built from the same
//! vector description: on Silkie it is a retained vector image (`f=200`)
//! patched one element at a time (an egg appearing in a nest, crossing
//! lamps flashing); on other terminals it is rasterized here once per level
//! and size, with the decals as small placements of a pre-rendered atlas.

pub mod layout;
pub mod snapshot;
pub mod text;
pub mod upload;

use crate::gfx::scenery::{self, DECAL_STATES, DecalSlot, Scenery, TILE};
use crate::gfx::sprites::Sheet;
use crate::gfx::vector::{Edit, Scene};
use crate::term::kitty_gfx::{self, EncodeStats, Place, Quiet};
use crate::world::level::Level;
use glam::Vec2;
use layout::Layout;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
use std::sync::Arc;
use text::Overlay;
use tiny_skia::{Pixmap, Transform};
use upload::Uploader;

/// Our image ids start here, well clear of small ids other programs use.
pub const IMG_BASE: u32 = 0x2f5d_0000;
pub const IMG_GROUND: u32 = IMG_BASE + 20;
pub const IMG_CANOPY: u32 = IMG_BASE + 21;
pub const IMG_DECALS: u32 = IMG_BASE + 22;
pub const IMG_TITLE: u32 = IMG_BASE + 24;
pub const IMG_POINTER: u32 = IMG_BASE + 25;
/// The white-hot glints of the effects sheet, as an HDR overlay.
pub const IMG_FX_HDR: u32 = IMG_BASE + 26;

// Stacking. Everything sits below INT32_MIN/2 so that cells with a
// background colour (menus) are drawn over the field, while ordinary cells
// let it show.
pub const Z_GROUND: i32 = -2_000_000_000;
pub const Z_DECAL: i32 = -1_950_000_000;
pub const Z_SPRITE: i32 = -1_900_000_000;
pub const Z_CANOPY: i32 = -1_800_000_000;
pub const Z_AIR: i32 = -1_750_000_000;
pub const Z_OVER: i32 = -1_700_000_000;
pub const Z_TITLE: i32 = -1_650_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SheetId {
    Hen,
    Chick,
    Fox,
    Hawk,
    Loki,
    Skadi,
    Cars,
    Floats,
    Sky,
    Items,
    Fx,
    /// Generated at start-up from the typeface, for score pop-ups.
    Glyphs,
}

impl SheetId {
    pub const ALL: [SheetId; 12] = [
        SheetId::Hen,
        SheetId::Chick,
        SheetId::Fox,
        SheetId::Hawk,
        SheetId::Loki,
        SheetId::Skadi,
        SheetId::Cars,
        SheetId::Floats,
        SheetId::Sky,
        SheetId::Items,
        SheetId::Fx,
        SheetId::Glyphs,
    ];

    pub fn image_id(self) -> u32 {
        IMG_BASE + 1 + self as u32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    /// On the ground, under everybody (shadows, treats).
    Decal,
    /// On the ground, sorted by how far down the screen their feet are.
    Body,
    /// Above the canopy: whatever flies, sorted the same way.
    Air,
    /// Above everything (emotes, particles, pop-ups).
    Over,
}

#[derive(Clone, Debug)]
pub struct Sprite {
    /// Stable for as long as the thing exists; the placement id.
    pub key: u32,
    pub sheet: SheetId,
    pub anim: String,
    /// Advances through the animation, wrapping.
    pub frame: usize,
    /// Anchor point in the field, in tiles.
    pub pos: Vec2,
    pub layer: Layer,
    /// Extra depth among sprites at the same height.
    pub z_bias: i32,
}

#[derive(Clone)]
pub struct SceneRef {
    /// Identifies the level; scenery is rebuilt when it changes.
    pub key: String,
    pub level: Arc<Level>,
}

#[derive(Default)]
pub struct FrameModel {
    pub scene: Option<SceneRef>,
    /// The state of each decal slot of the level (see `scenery`).
    pub decals: Vec<u8>,
    pub sprites: Vec<Sprite>,
    pub top: String,
    pub bottom: String,
    pub overlays: Vec<Overlay>,
    pub title: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameStats {
    pub bytes: usize,
    pub commands: u64,
    pub placements: u32,
    pub deletions: u32,
    pub uploads: u32,
    pub upload_bytes: usize,
    pub vector_edits: u32,
    pub vector_bytes: usize,
    pub vector_full_redraws: u32,
    pub text_bytes: usize,
    pub clipped: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Totals {
    pub frames: u64,
    pub bytes: u64,
    pub placements: u64,
    pub uploads: u64,
    pub upload_bytes: u64,
    pub vector_edits: u64,
    pub vector_bytes: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct RenderConfig {
    pub vector_scenery: bool,
    pub uploader: Uploader,
    pub sync_output: bool,
    /// Translucent menu backgrounds (Silkie cell alpha); harmless elsewhere.
    pub cell_alpha: bool,
    /// Sparkles brighter than SDR white, by this factor (1 = off).
    pub hdr_boost: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Placed {
    row: u32,
    col: u32,
    place: Place,
}

struct SceneryState {
    key: String,
    scenery: Scenery,
    slots: Vec<DecalSlot>,
    /// Pixels per tile the raster ground and canopy were made at.
    raster_for: Option<u32>,
    /// Tile size the decal atlas was made for.
    atlas_for: Option<f32>,
    decals_sent: Vec<u8>,
}

pub struct Renderer {
    pub cfg: RenderConfig,
    pub layout: Option<Layout>,
    /// Sheets painted for `sheets_for` pixels per tile, made on first use.
    sheets: Vec<Option<Sheet>>,
    sheets_for: Option<u32>,
    /// A loading banner written into the field's text cells, to clear.
    banner: Option<(u32, u32, u32)>,
    uploaded: BTreeSet<u32>,
    placed: BTreeMap<(u32, u32), Placed>,
    scenery: Option<SceneryState>,
    text_rows: BTreeMap<u32, String>,
    overlay_rects: Vec<(u32, u32, u32, u32)>,
    overlay_text: Vec<String>,
    title: Option<(Layout, u32, u32)>,
    pub last: FrameStats,
    pub totals: Totals,
    /// The latest error the terminal reported for our graphics commands,
    /// and how many there were.
    pub last_error: Option<String>,
    pub error_count: u64,
}

/// A decal atlas cell is this many tiles square, the slot at its middle.
const DECAL_CELL_TILES: f32 = 1.6;

/// Sprite resolutions (pixels per tile), about 11% apart. Sprites are drawn
/// at the largest step that fits the tile, so resizing the window a little
/// re-sends nothing; positions stay exact, only sprite size is quantized.
const LADDER: [u32; 22] = [24, 28, 32, 36, 40, 45, 50, 56, 63, 70, 78, 87, 96, 107, 119, 133, 148, 165, 184, 205, 228, 254];

pub fn sprite_res(tile_px: f32) -> u32 {
    let limit = tile_px * 1.04;
    LADDER.iter().copied().rev().find(|&s| s as f32 <= limit).unwrap_or(LADDER[0])
}

/// Raster scenery for a terminal at the far end of a link (no shared
/// files) is sent at this fraction of the sprite resolution and enlarged by
/// the terminal: a softer field for a third of the bytes.
const REMOTE_SCENERY_SCALE: f32 = 0.65;

impl Renderer {
    pub fn new(cfg: RenderConfig) -> Self {
        Renderer {
            cfg,
            layout: None,
            sheets: SheetId::ALL.iter().map(|_| None).collect(),
            sheets_for: None,
            banner: None,
            uploaded: BTreeSet::new(),
            placed: BTreeMap::new(),
            scenery: None,
            text_rows: BTreeMap::new(),
            overlay_rects: Vec::new(),
            overlay_text: Vec::new(),
            title: None,
            last: FrameStats::default(),
            totals: Totals::default(),
            last_error: None,
            error_count: 0,
        }
    }

    /// Changes the layout; images sized to the old one are redone lazily.
    pub fn set_layout(&mut self, l: Option<Layout>) {
        self.layout = l;
    }

    /// Forgets everything the terminal held (after a suspend, a terminal
    /// reset, or a resize that cleared the screen): it is all sent again.
    pub fn invalidate(&mut self) {
        self.uploaded.clear();
        self.placed.clear();
        self.sheets_for = None;
        self.text_rows.clear();
        self.overlay_rects.clear();
        self.overlay_text.clear();
        self.title = None;
        self.scenery = None;
    }

    /// The terminal reported an error for one of our images (for example
    /// evicted under storage pressure): send it again.
    pub fn on_graphics_error(&mut self, id: u32, message: &str) {
        tracing::warn!("terminal reported a graphics error for image {id:#x}: {message}");
        self.error_count = self.error_count.saturating_add(1);
        self.last_error = Some(format!("{id:#x}: {message}"));
        if message.starts_with("ENOENT") {
            self.uploaded.remove(&id);
            self.placed.retain(|k, _| k.0 != id);
            if id == IMG_GROUND || id == IMG_CANOPY || id == IMG_DECALS {
                self.scenery = None;
            }
            if id == IMG_FX_HDR {
                self.uploaded.remove(&SheetId::Fx.image_id());
            }
        }
    }

    /// Every image id the renderer may have created, for cleanup.
    pub fn image_ids() -> Vec<u32> {
        let mut v: Vec<u32> = SheetId::ALL.iter().map(|s| s.image_id()).collect();
        v.extend([IMG_GROUND, IMG_CANOPY, IMG_DECALS, IMG_TITLE, IMG_POINTER, IMG_FX_HDR]);
        v
    }

    /// Uploads the pointer image (`px` pixels square) and returns the bytes,
    /// with the hotspot.
    pub fn pointer_image(&mut self, px: u32) -> (Vec<u8>, (u32, u32)) {
        let mut out = Vec::new();
        let (pm, hot) = crate::art::fx::feather_cursor(px);
        let s = self.cfg.uploader.pixmap(&mut out, IMG_POINTER, &pm);
        let mut st = FrameStats::default();
        self.note_upload(IMG_POINTER, s, &mut st);
        (out, hot)
    }

    fn note_upload(&mut self, id: u32, s: EncodeStats, st: &mut FrameStats) {
        self.uploaded.insert(id);
        self.placed.retain(|k, _| k.0 != id);
        st.uploads += 1;
        st.upload_bytes += s.bytes as usize;
        st.commands += s.commands;
    }

    /// Makes sure a sheet is painted for the current sprite resolution and
    /// uploaded; sheets are sent the first time something needs them.
    fn ensure_sheet(&mut self, out: &mut Vec<u8>, id: SheetId, res: u32, st: &mut FrameStats) {
        if self.sheets_for != Some(res) {
            self.sheets_for = Some(res);
            self.sheets.fill(None);
            for sid in SheetId::ALL {
                self.uploaded.remove(&sid.image_id());
            }
            self.uploaded.remove(&IMG_FX_HDR);
        }
        let i = id as usize;
        if self.sheets[i].is_none() {
            self.sheets[i] = Some(paint_sheet(id, res));
        }
        if self.uploaded.contains(&id.image_id()) {
            return;
        }
        let sheet = self.sheets[i].as_ref().expect("painted above");
        let s = self.cfg.uploader.rgba(out, id.image_id(), sheet.width, sheet.height, &sheet.rgba);
        let hdr = (id == SheetId::Fx && self.cfg.hdr_boost > 1.0).then(|| {
            let data = upload::hdr_glints(&sheet.rgba, self.cfg.hdr_boost);
            self.cfg.uploader.hdr(out, IMG_FX_HDR, sheet.width, sheet.height, &data)
        });
        self.note_upload(id.image_id(), s, st);
        if let Some(h) = hdr {
            self.note_upload(IMG_FX_HDR, h, st);
        }
    }

    fn sheet(&self, id: SheetId) -> Option<&Sheet> {
        self.sheets[id as usize].as_ref()
    }

    /// Whether drawing `f` would mean sending a lot of image data first
    /// (a new level, a new size): the app puts up a banner meanwhile.
    pub fn needs_heavy_upload(&self, f: &FrameModel) -> bool {
        let Some(l) = self.layout else { return false };
        let res = sprite_res(l.tile_px);
        let sheets = self.sheets_for != Some(res) || f.sprites.iter().any(|s| !self.uploaded.contains(&s.sheet.image_id()));
        let scenery = f.scene.as_ref().is_some_and(|sr| {
            self.scenery.as_ref().is_none_or(|s| s.key != sr.key || (!self.cfg.vector_scenery && s.raster_for != Some(self.raster_res(&l))))
        });
        (sheets || scenery) && !self.cfg.uploader.temp_file
    }

    fn raster_res(&self, l: &Layout) -> u32 {
        let res = sprite_res(l.tile_px) as f32;
        let k = if self.cfg.uploader.temp_file { 1.0 } else { REMOTE_SCENERY_SCALE };
        ((res * k).round() as u32).max(16)
    }

    /// Renders a frame and returns the bytes to write (empty when nothing
    /// changed).
    pub fn render(&mut self, f: &FrameModel) -> Vec<u8> {
        let mut st = FrameStats::default();
        let mut out = Vec::new();
        let Some(l) = self.layout else {
            return out;
        };
        let res = sprite_res(l.tile_px);
        let needed: BTreeSet<SheetId> = f.sprites.iter().map(|s| s.sheet).collect();
        for id in needed {
            self.ensure_sheet(&mut out, id, res, &mut st);
        }
        let mut desired: BTreeMap<(u32, u32), Placed> = BTreeMap::new();
        match &f.scene {
            Some(sr) => self.scenery_frame(&mut out, &l, sr, f, &mut desired, &mut st),
            None => self.scenery = None,
        }
        if f.title {
            self.title_frame(&mut out, &l, &mut desired, &mut st);
        }
        st.clipped = self.sprite_placements(&l, &f.sprites, &mut desired);
        self.apply_placements(&mut out, desired, &mut st);
        let mut text = Vec::new();
        // The loading banner stays up while uploads stream, and goes with
        // the first frame after them.
        if st.upload_bytes == 0 {
            if let Some((row, col, w)) = self.banner.take() {
                let _ = write!(text, "\x1b[{};{}H\x1b[0m{}", row + 1, col + 1, " ".repeat(w as usize));
            }
        }
        self.status_lines(&mut text, &l, f);
        self.overlays(&mut text, &l, &f.overlays);
        st.text_bytes = text.len();
        if st.upload_bytes > 0 {
            // Big uploads stream for a while: show the text first, outside
            // a synchronized update, so the screen is not frozen meanwhile.
            text.extend_from_slice(&out);
            out = text;
            st.bytes = out.len();
            self.count(&st);
            self.last = st;
            return out;
        }
        out.extend_from_slice(&text);
        if out.is_empty() {
            self.last = st;
            return out;
        }
        if self.cfg.sync_output {
            let mut wrapped = Vec::with_capacity(out.len() + 16);
            wrapped.extend_from_slice(b"\x1b[?2026h");
            wrapped.extend_from_slice(&out);
            wrapped.extend_from_slice(b"\x1b[?2026l");
            out = wrapped;
        }
        st.bytes = out.len();
        self.count(&st);
        self.last = st;
        out
    }

    fn count(&mut self, st: &FrameStats) {
        self.totals.frames += 1;
        self.totals.bytes += st.bytes as u64;
        self.totals.placements += st.placements as u64;
        self.totals.uploads += st.uploads as u64;
        self.totals.upload_bytes += st.upload_bytes as u64;
        self.totals.vector_edits += st.vector_edits as u64;
        self.totals.vector_bytes += st.vector_bytes as u64;
    }

    /// A one-line banner in the middle of the field, cleared by the next
    /// frame. For long uploads over slow links.
    pub fn banner(&mut self, msg: &str) -> Vec<u8> {
        let Some(l) = self.layout else { return Vec::new() };
        let w = text::width(msg) as u32 + 4;
        let row = l.field_row + l.field_rows / 2;
        let col = l.cols.saturating_sub(w) / 2;
        self.banner = Some((row, col, w));
        let mut out = Vec::new();
        let _ = write!(
            out,
            "\x1b[{};{}H\x1b[0m\x1b[48;2;{};{};{}m\x1b[38;2;255;214;102m  {msg}  \x1b[0m",
            row + 1,
            col + 1,
            text::BG.0,
            text::BG.1,
            text::BG.2
        );
        out
    }

    fn scenery_frame(&mut self, out: &mut Vec<u8>, l: &Layout, sr: &SceneRef, f: &FrameModel, desired: &mut BTreeMap<(u32, u32), Placed>, st: &mut FrameStats) {
        let rebuild = self.scenery.as_ref().is_none_or(|s| s.key != sr.key);
        if rebuild {
            let mut scenery = scenery::build(&sr.level);
            let slots = scenery::decal_slots(&sr.level);
            // Decals start in the state of this frame, so nothing needs
            // patching straight away.
            let states: Vec<u8> = (0..slots.len()).map(|i| f.decals.get(i).copied().unwrap_or(0)).collect();
            for (i, slot) in slots.iter().enumerate() {
                for (id, e) in scenery::decal_elements(i, slot, states[i]) {
                    scenery.ground.set(id, e);
                }
            }
            self.scenery = Some(SceneryState {
                key: sr.key.clone(),
                scenery,
                slots,
                raster_for: None,
                atlas_for: None,
                decals_sent: states,
            });
            if self.cfg.vector_scenery {
                self.transmit_vectors(out, st);
            }
        }
        // Vector scenery is never re-sent for a new size: the terminal
        // re-rasterizes it. Raster scenery is re-made only when the sprite
        // resolution step changes, and otherwise enlarged by the terminal.
        if !self.cfg.vector_scenery {
            let res = self.raster_res(l);
            if self.scenery.as_ref().is_some_and(|s| s.raster_for != Some(res)) {
                self.raster_scenery(out, res, st);
            }
            if self.scenery.as_ref().is_some_and(|s| s.atlas_for != Some(l.tile_px)) {
                self.raster_atlas(out, l, st);
            }
        }
        let field = |z: i32, id: u32| Placed {
            row: l.field_row,
            col: l.field_col,
            place: Place {
                id,
                placement: 1,
                src: None,
                offset: (0, 0),
                z,
                cols: Some(l.field_cols),
                rows: None,
            },
        };
        desired.insert((IMG_GROUND, 1), field(Z_GROUND, IMG_GROUND));
        desired.insert((IMG_CANOPY, 1), field(Z_CANOPY, IMG_CANOPY));
        if self.cfg.vector_scenery {
            self.vector_decals(out, f, st);
        } else {
            self.raster_decals(l, f, desired);
        }
    }

    fn transmit_vectors(&mut self, out: &mut Vec<u8>, st: &mut FrameStats) {
        let s = self.scenery.as_ref().expect("scenery built");
        let g = &s.scenery.ground;
        let rec = g.records();
        let es = kitty_gfx::vector_transmit(out, IMG_GROUND, g.width as f64, g.height as f64, &rec, Quiet::ErrorsOnly);
        let c = &s.scenery.canopy;
        let rec2 = c.records();
        let es2 = kitty_gfx::vector_transmit(out, IMG_CANOPY, c.width as f64, c.height as f64, &rec2, Quiet::ErrorsOnly);
        st.vector_bytes += (es.bytes + es2.bytes) as usize;
        self.note_upload(IMG_GROUND, es, st);
        self.note_upload(IMG_CANOPY, es2, st);
    }

    /// The ground and canopy as bitmaps at `res` pixels per tile; the
    /// terminal scales them to the field.
    fn raster_scenery(&mut self, out: &mut Vec<u8>, res: u32, st: &mut FrameStats) {
        let scale = res as f32 / TILE;
        let s = self.scenery.as_mut().expect("scenery built");
        let w = ((s.scenery.width * scale).round() as u32).max(1);
        let h = ((s.scenery.height * scale).round() as u32).max(1);
        let t = Transform::from_scale(scale, scale);
        let mut ground = Pixmap::new(w, h).expect("field size");
        // Decals are separate placements in raster mode.
        let mut base = s.scenery.ground.clone();
        for i in 0..s.slots.len() {
            for id in scenery::decal_ids(i) {
                base.delete(id);
            }
        }
        base.render(&mut ground, t);
        let mut canopy = Pixmap::new(w, h).expect("field size");
        s.scenery.canopy.render(&mut canopy, t);
        s.raster_for = Some(res);
        let up = self.cfg.uploader;
        let a = up.pixmap(out, IMG_GROUND, &ground);
        self.note_upload(IMG_GROUND, a, st);
        let b = up.pixmap(out, IMG_CANOPY, &canopy);
        self.note_upload(IMG_CANOPY, b, st);
    }

    /// Every decal in every state, placed unscaled at exact positions, so
    /// the atlas follows the tile size exactly. It is small.
    fn raster_atlas(&mut self, out: &mut Vec<u8>, l: &Layout, st: &mut FrameStats) {
        let s = self.scenery.as_mut().expect("scenery built");
        s.atlas_for = Some(l.tile_px);
        if s.slots.is_empty() {
            return;
        }
        let pm = decal_atlas(&s.slots, l.tile_px);
        let up = self.cfg.uploader;
        let c = up.pixmap(out, IMG_DECALS, &pm);
        self.note_upload(IMG_DECALS, c, st);
    }

    fn raster_decals(&mut self, l: &Layout, f: &FrameModel, desired: &mut BTreeMap<(u32, u32), Placed>) {
        let Some(s) = &self.scenery else { return };
        let cell = (DECAL_CELL_TILES * l.tile_px).round() as u32;
        for (i, slot) in s.slots.iter().enumerate() {
            let state = f.decals.get(i).copied().unwrap_or(0).min(DECAL_STATES - 1);
            let src = [state as u32 * cell, i as u32 * cell, cell, cell];
            let (x, y) = l.world_to_px(Vec2::new(slot.at.0, slot.at.1));
            let half = cell as f32 / 2.0;
            if let Some((row, c, xo, yo)) = l.cell_at(x - half, y - half) {
                desired.insert(
                    (IMG_DECALS, i as u32 + 1),
                    Placed {
                        row,
                        col: c,
                        place: Place {
                            id: IMG_DECALS,
                            placement: i as u32 + 1,
                            src: Some(src),
                            offset: (xo, yo),
                            z: Z_DECAL,
                            cols: None,
                            rows: None,
                        },
                    },
                );
            }
        }
    }

    fn vector_decals(&mut self, out: &mut Vec<u8>, f: &FrameModel, st: &mut FrameStats) {
        let Some(s) = &mut self.scenery else { return };
        let mut edits: Vec<Edit> = Vec::new();
        for (i, slot) in s.slots.iter().enumerate() {
            let state = f.decals.get(i).copied().unwrap_or(0);
            if s.decals_sent.get(i) != Some(&state) {
                for (id, e) in scenery::decal_elements(i, slot, state) {
                    edits.push(Edit::Set(id, e));
                }
                if let Some(v) = s.decals_sent.get_mut(i) {
                    *v = state;
                }
            }
        }
        if edits.is_empty() {
            return;
        }
        let mut text = String::new();
        for e in &edits {
            if e.redraws_everything(&s.scenery.ground) {
                st.vector_full_redraws += 1;
            }
            s.scenery.ground.apply(e);
            text.push_str(&e.record());
            text.push('\n');
        }
        let es = kitty_gfx::vector_edit(out, IMG_GROUND, &text, Quiet::ErrorsOnly);
        st.vector_edits += edits.len() as u32;
        st.vector_bytes += es.bytes as usize;
        st.commands += es.commands;
    }

    pub fn has_vector_ground(&self) -> bool {
        self.cfg.vector_scenery && self.scenery.is_some() && self.uploaded.contains(&IMG_GROUND)
    }

    fn title_frame(&mut self, out: &mut Vec<u8>, l: &Layout, desired: &mut BTreeMap<(u32, u32), Placed>, st: &mut FrameStats) {
        use crate::gfx::title::{LOGO_H, LOGO_W};
        // The logo spans most of the field's width, near its top. On Silkie
        // it is a vector image sent once and scaled by the terminal.
        let cols = ((l.field_cols as f32) * 0.72).round().max(1.0) as u32;
        let w = cols * l.cell_w;
        let h = (w as f32 * LOGO_H / LOGO_W).round() as u32;
        if self.cfg.vector_scenery {
            if !self.uploaded.contains(&IMG_TITLE) {
                let scene = crate::gfx::title::logo_scene();
                let s = kitty_gfx::vector_transmit(out, IMG_TITLE, LOGO_W as f64, LOGO_H as f64, &scene.records(), Quiet::ErrorsOnly);
                st.vector_bytes += s.bytes as usize;
                self.note_upload(IMG_TITLE, s, st);
            }
        } else {
            let needs = self.title.is_none_or(|(tl, _, _)| tl != *l) || !self.uploaded.contains(&IMG_TITLE);
            if needs {
                let pm = crate::gfx::title::logo(w, h);
                let s = self.cfg.uploader.pixmap(out, IMG_TITLE, &pm);
                self.note_upload(IMG_TITLE, s, st);
                self.title = Some((*l, w, h));
            }
        }
        let (ox, oy) = l.origin_px();
        let x = ox + ((l.field_cols - cols) / 2 * l.cell_w) as f32;
        let y = oy + l.field_h as f32 * 0.07;
        if let Some((row, col, xo, yo)) = l.cell_at(x, y) {
            desired.insert(
                (IMG_TITLE, 1),
                Placed {
                    row,
                    col,
                    place: Place {
                        id: IMG_TITLE,
                        placement: 1,
                        src: None,
                        offset: (xo, yo),
                        z: Z_TITLE,
                        cols: self.cfg.vector_scenery.then_some(cols),
                        rows: None,
                    },
                },
            );
        }
    }

    /// Places every sprite, clipped to the field. Returns how many were
    /// clipped (partly off the field).
    fn sprite_placements(&self, l: &Layout, sprites: &[Sprite], desired: &mut BTreeMap<(u32, u32), Placed>) -> u32 {
        let (fx0, fy0) = l.origin_px();
        let (fx1, fy1) = (fx0 + l.field_w as f32, fy0 + l.field_h as f32);
        let mut clipped = 0;
        for sp in sprites {
            let Some(sheet) = self.sheet(sp.sheet) else { continue };
            let Some(frames) = sheet.anim(&sp.anim) else { continue };
            if frames.is_empty() {
                continue;
            }
            let frame = frames[sp.frame % frames.len()];
            let [sx, sy, sw, sh] = sheet.frame_rect(frame);
            let (ax, ay) = l.world_to_px(sp.pos);
            // Whole pixels, so that the clip and the placement agree.
            let x0 = (ax - sheet.spec.anchor_x).round();
            let y0 = (ay - sheet.spec.anchor_y).round();
            let cx0 = x0.max(fx0.round());
            let cy0 = y0.max(fy0.round());
            let cx1 = (x0 + sw as f32).min(fx1.round());
            let cy1 = (y0 + sh as f32).min(fy1.round());
            if cx1 <= cx0 || cy1 <= cy0 {
                continue;
            }
            let src = [
                sx + (cx0 - x0) as u32,
                sy + (cy0 - y0) as u32,
                (cx1 - cx0) as u32,
                (cy1 - cy0) as u32,
            ];
            if src[2] != sw || src[3] != sh {
                clipped += 1;
            }
            let Some((row, col, xo, yo)) = l.cell_at(cx0, cy0) else {
                continue;
            };
            let foot = ay.round() as i32;
            let z = match sp.layer {
                Layer::Decal => Z_DECAL + 10 + sp.z_bias,
                Layer::Body => Z_SPRITE + foot * 8 + sp.z_bias,
                Layer::Air => Z_AIR + foot * 8 + sp.z_bias,
                Layer::Over => Z_OVER + sp.z_bias,
            };
            let place = |id, z| Placed {
                row,
                col,
                place: Place {
                    id,
                    placement: sp.key,
                    src: Some(src),
                    offset: (xo, yo),
                    z,
                    cols: None,
                    rows: None,
                },
            };
            desired.insert((sp.sheet.image_id(), sp.key), place(sp.sheet.image_id(), z));
            // Sparkles, stars and hearts get their white-hot glints on top.
            let glow = sp.sheet == SheetId::Fx && matches!(sp.anim.as_str(), "sparkle" | "star" | "heart");
            if glow && self.cfg.hdr_boost > 1.0 {
                desired.insert((IMG_FX_HDR, sp.key), place(IMG_FX_HDR, z + 1));
            }
        }
        clipped
    }

    fn apply_placements(&mut self, out: &mut Vec<u8>, desired: BTreeMap<(u32, u32), Placed>, st: &mut FrameStats) {
        let stale: Vec<(u32, u32)> = self.placed.keys().filter(|k| !desired.contains_key(k)).copied().collect();
        for k in stale {
            let s = kitty_gfx::delete_placement(out, k.0, k.1);
            st.commands += s.commands;
            st.deletions += 1;
            self.placed.remove(&k);
        }
        for (k, p) in desired {
            if self.placed.get(&k) == Some(&p) {
                continue;
            }
            if !self.uploaded.contains(&k.0) {
                continue;
            }
            let _ = write!(out, "\x1b[{};{}H", p.row + 1, p.col + 1);
            let s = p.place.write(out, Quiet::ErrorsOnly);
            st.commands += s.commands;
            st.placements += 1;
            self.placed.insert(k, p);
        }
    }

    fn status_lines(&mut self, out: &mut Vec<u8>, l: &Layout, f: &FrameModel) {
        for (row, line) in [(0u32, &f.top), (l.rows - 1, &f.bottom)] {
            if self.text_rows.get(&row) == Some(line) {
                continue;
            }
            let _ = write!(out, "\x1b[{};1H\x1b[0m\x1b[2K{}\x1b[0m", row + 1, text::fit(line, l.cols as usize));
            self.text_rows.insert(row, line.clone());
        }
    }

    fn overlays(&mut self, out: &mut Vec<u8>, l: &Layout, overlays: &[Overlay]) {
        let rendered: Vec<(String, (u32, u32, u32, u32))> = overlays
            .iter()
            .map(|o| text::render_overlay(o, l, self.cfg.cell_alpha))
            .collect();
        let texts: Vec<String> = rendered.iter().map(|r| r.0.clone()).collect();
        if texts == self.overlay_text {
            return;
        }
        // Blank what the previous overlays covered, then draw the new ones.
        for &(col, row, w, h) in &self.overlay_rects {
            for r in row..row + h {
                let _ = write!(out, "\x1b[{};{}H\x1b[0m{}", r + 1, col + 1, " ".repeat(w as usize));
            }
        }
        for (s, _) in &rendered {
            out.extend_from_slice(s.as_bytes());
        }
        out.extend_from_slice(b"\x1b[0m");
        self.overlay_rects = rendered.iter().map(|r| r.1).collect();
        self.overlay_text = texts;
    }

    /// Forgets the status lines and overlays (after the text area was
    /// cleared), so they are written again.
    pub fn reset_text(&mut self) {
        self.text_rows.clear();
        self.overlay_rects.clear();
        self.overlay_text.clear();
    }

    /// Placements currently on screen, for tests and diagnostics.
    pub fn placement_count(&self) -> usize {
        self.placed.len()
    }

    pub fn placements_of(&self, image: u32) -> usize {
        self.placed.keys().filter(|k| k.0 == image).count()
    }

    pub fn is_uploaded(&self, image: u32) -> bool {
        self.uploaded.contains(&image)
    }

    pub fn scenery_scene(&self) -> Option<&Scene> {
        self.scenery.as_ref().map(|s| &s.scenery.ground)
    }

    /// Takes every placement off the screen but keeps the images, for a
    /// window too small to show the field: when it grows again they are
    /// only placed anew.
    pub fn hide(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        for (image, placement) in std::mem::take(&mut self.placed).into_keys() {
            kitty_gfx::delete_placement(&mut out, image, placement);
        }
        self.text_rows.clear();
        self.overlay_rects.clear();
        self.overlay_text.clear();
        out
    }

    /// Deletes every image (on exit or before handing the screen over).
    pub fn teardown(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        for id in std::mem::take(&mut self.uploaded) {
            kitty_gfx::delete_image(&mut out, id);
        }
        self.placed.clear();
        out
    }
}

/// Paints one sheet at `res` pixels per tile.
pub fn paint_sheet(id: SheetId, res: u32) -> Sheet {
    let (spec, pm) = match id {
        SheetId::Hen => crate::art::hen::sheet(res),
        SheetId::Chick => crate::art::chick::sheet(res),
        SheetId::Fox => crate::art::critters::fox_sheet(res),
        SheetId::Hawk => crate::art::critters::hawk_sheet(res),
        SheetId::Loki => crate::art::loki::sheet(res),
        SheetId::Skadi => crate::art::skadi::sheet(res),
        SheetId::Cars => crate::art::vehicles::sheet(res),
        SheetId::Floats => crate::art::floats::sheet(res),
        SheetId::Sky => crate::art::sky::sheet(res),
        SheetId::Items => crate::art::fx::items_sheet(res),
        SheetId::Fx => crate::art::fx::fx_sheet(res),
        SheetId::Glyphs => return crate::gfx::title::glyph_sheet(res),
    };
    let (width, height) = (pm.width(), pm.height());
    Sheet {
        spec,
        width,
        height,
        rgba: pm.take_demultiplied(),
    }
}

/// Every decal in every state: rows are slots, columns are states.
fn decal_atlas(slots: &[DecalSlot], tile_px: f32) -> Pixmap {
    let cell = (DECAL_CELL_TILES * tile_px).round().max(1.0) as u32;
    let mut pm = Pixmap::new(cell * DECAL_STATES as u32, cell * slots.len().max(1) as u32).expect("decal atlas size");
    let scale = tile_px / TILE;
    for (i, slot) in slots.iter().enumerate() {
        for state in 0..DECAL_STATES {
            let mut scene = Scene::new(0.0, 0.0);
            scene.set(scenery::G_DECALS, crate::gfx::vector::Element::group(0));
            for (id, e) in scenery::decal_elements(i, slot, state) {
                scene.set(id, e);
            }
            // Map the slot's centre to the cell's centre.
            let (cx, cy) = (slot.at.0 * TILE, slot.at.1 * TILE);
            let ox = state as f32 * cell as f32 + cell as f32 / 2.0;
            let oy = i as f32 * cell as f32 + cell as f32 / 2.0;
            let t = Transform::from_row(scale, 0.0, 0.0, scale, ox - cx * scale, oy - cy * scale);
            scene.render(&mut pm, t);
        }
    }
    pm
}

#[cfg(test)]
mod tests;
