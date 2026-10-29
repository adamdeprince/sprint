//! Paints the title logo to a PNG: `cargo run --release --example logo -- 1000 logo.png`
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let w: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1000);
    let out = args.get(2).cloned().unwrap_or_else(|| "logo.png".into());
    let h = (w as f32 * silkie_sprint::gfx::title::LOGO_H / silkie_sprint::gfx::title::LOGO_W) as u32;
    let mut bg = tiny_skia::Pixmap::new(w, h).unwrap();
    bg.fill(tiny_skia::Color::from_rgba8(0x8c, 0xc1, 0x69, 255));
    let logo = silkie_sprint::gfx::title::logo(w, h);
    bg.draw_pixmap(0, 0, logo.as_ref(), &tiny_skia::PixmapPaint::default(), tiny_skia::Transform::identity(), None);
    bg.save_png(&out).unwrap();
}
