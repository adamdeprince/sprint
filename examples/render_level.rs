//! Draws a level's scenery (ground, decals in a given state, canopy) to a
//! PNG: `cargo run --release --example render_level -- 1 48 level.png [state]`
use silkie_sprint::gfx::scenery;
use silkie_sprint::world::level;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let px: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(48.0);
    let out = args.get(3).cloned().unwrap_or_else(|| "level.png".into());
    let state: u8 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
    let lvl = level::builtin(n.saturating_sub(1)).expect("no such level");
    let mut s = scenery::build(&lvl);
    for (i, slot) in scenery::decal_slots(&lvl).iter().enumerate() {
        for (id, e) in scenery::decal_elements(i, slot, state) {
            s.ground.set(id, e);
        }
    }
    let k = px / scenery::TILE;
    let mut pm = tiny_skia::Pixmap::new((s.width * k) as u32, (s.height * k) as u32).unwrap();
    let t = tiny_skia::Transform::from_scale(k, k);
    s.ground.render(&mut pm, t);
    s.canopy.render(&mut pm, t);
    pm.save_png(&out).unwrap();
    println!("{}: {} ground elements, {} bytes of records", lvl.name, s.ground.len(), s.ground.records().len());
}
