//! Status lines, banners, menus and the capability report.

use super::control::PAUSE_MENU;
use super::{App, Screen};
use crate::render::FrameModel;
use crate::render::text::{Anchor, Line, Overlay, Style, width};
use crate::sim::rules::MIN_TAKEOFF;
use crate::sim::{Game, Nest, Phase};
use crate::term::caps::Capabilities;

const GOLD: &str = "\x1b[1;38;2;255;214;102m";
const CREAM: &str = "\x1b[38;2;244;236;214m";
const DIM: &str = "\x1b[38;2;150;160;130m";
const RED: &str = "\x1b[38;2;255;105;120m";
const SKY: &str = "\x1b[38;2;140;210;255m";
const GREEN: &str = "\x1b[38;2;156;222;120m";
const AMBER: &str = "\x1b[38;2;255;179;71m";
const RESET: &str = "\x1b[0m";

fn spread(left: &str, right: &str, cols: usize) -> String {
    let gap = cols.saturating_sub(width(left) + width(right)).max(1);
    format!("{left}{}{right}", " ".repeat(gap))
}

/// A bar of `n` cells, `frac` full.
fn bar(frac: f32, n: usize, on: &str) -> String {
    let full = ((frac.clamp(0.0, 1.0) * n as f32).round() as usize).min(n);
    format!("{on}{}{DIM}{}{RESET}", "▮".repeat(full), "▯".repeat(n - full))
}

fn status_top(g: Option<&Game>, best: u32, cols: usize, title: bool) -> String {
    let left = match g {
        Some(g) if !title => format!(
            " {GOLD}SILKIE SPRINT{RESET}  {DIM}Level {}{RESET} {CREAM}{}{RESET}",
            g.level_index + 1,
            g.level.name
        ),
        _ => format!(" {GOLD}SILKIE SPRINT{RESET}  {DIM}why did the Silkie cross the road?{RESET}"),
    };
    let right = match g {
        Some(g) if !title => format!(
            "{DIM}SCORE{RESET} {GOLD}{:06}{RESET}   {DIM}BEST{RESET} {CREAM}{:06}{RESET} ",
            g.session.score,
            best.max(g.session.score)
        ),
        _ => format!("{DIM}BEST{RESET} {CREAM}{best:06}{RESET} "),
    };
    spread(&left, &right, cols)
}

fn status_bottom(g: &Game, cols: usize) -> String {
    let hearts: String = "♥".repeat(g.session.lives.min(9) as usize);
    let more = if g.session.lives > 9 { format!("+{}", g.session.lives - 9) } else { String::new() };
    let nests: String = g
        .nests
        .iter()
        .map(|n| match n {
            Nest::Empty => format!("{DIM}○{RESET}"),
            _ => format!("{CREAM}●{RESET}"),
        })
        .collect();
    let meter = g.hen.meter;
    let wing_colour = if g.hen.flying {
        AMBER
    } else if meter >= MIN_TAKEOFF {
        SKY
    } else {
        RED
    };
    let frac = (g.time_left / g.level.time).clamp(0.0, 1.0);
    let clock_colour = if frac > 0.5 {
        GREEN
    } else if frac > 0.2 {
        AMBER
    } else {
        RED
    };
    let chick = if g.hen.carrying.is_some() { format!("  {GOLD}+chick{RESET}") } else { String::new() };
    let combo = if g.combo >= 2 && g.combo_left > 0.0 { format!("  {AMBER}bug combo x{}{RESET}", g.combo) } else { String::new() };
    let left = format!(
        " {RED}{hearts}{more}{RESET}  {DIM}nests{RESET} {nests}  {DIM}wings{RESET} {}  {DIM}time{RESET} {}{chick}{combo}",
        bar(meter, 8, wing_colour),
        bar(frac, 10, clock_colour),
    );
    let right = format!("{DIM}arrows: hop · space: fly/land · p: pause{RESET} ");
    spread(&left, &right, cols)
}

pub fn decorate(app: &App, m: &mut FrameModel) {
    let cols = app.renderer.layout.map_or(80, |l| l.cols as usize);
    let best = app.scores.best();
    match &app.screen {
        Screen::Title { .. } => {
            m.top = status_top(None, best, cols, true);
            m.bottom = spread(
                &format!(" {CREAM}Enter{RESET}{DIM}: play{RESET}   {CREAM}C{RESET}{DIM}: terminal features{RESET}   {CREAM}Q{RESET}{DIM}: quit{RESET}"),
                &format!("{DIM}arrows/WASD hop · space flies (briefly){RESET} "),
                cols,
            );
            let mut lines = vec![
                Line::new("Hop the traffic. Ride the river. Fly when you must.", Style::Normal),
                Line::new("Chase bugs, dodge prams, mind the hawk.", Style::Dim),
                Line::new("Loki and Skaði only want cuddles (and Skaði is the more careful).", Style::Dim),
                Line::new("", Style::Normal),
                Line::new("Press Enter to play", Style::Heading),
            ];
            let enhanced: Vec<&str> = app
                .caps
                .summary()
                .into_iter()
                .skip(2)
                .filter(|(_, on)| *on)
                .map(|(n, _)| n)
                .collect();
            if !enhanced.is_empty() && !app.opts.kitty_only {
                lines.push(Line::new("", Style::Normal));
                lines.push(Line::new(format!("this terminal adds: {}", enhanced.join(" · ")), Style::Dim));
            }
            m.overlays.push(Overlay::new(Anchor::Low, lines));
            if !app.scores.entries.is_empty() {
                let mut hs = vec![Line::new("Best sprinters", Style::Heading)];
                for (i, e) in app.scores.entries.iter().take(5).enumerate() {
                    hs.push(Line::new(format!("{}. {:<12} {:>7}", i + 1, e.name, e.score), Style::Normal));
                }
                let mut o = Overlay::new(Anchor::TopLeft, hs);
                o.alpha = 60;
                m.overlays.push(o);
            }
        }
        Screen::Report => {
            m.top = status_top(None, best, cols, true);
            m.bottom = format!(" {DIM}any key: back{RESET}");
            let lines: Vec<Line> = capability_report(&app.caps)
                .into_iter()
                .enumerate()
                .map(|(i, s)| Line::new(s, if i == 0 { Style::Heading } else { Style::Normal }))
                .collect();
            let mut o = Overlay::new(Anchor::Center, lines);
            o.alpha = 92;
            m.overlays.push(o);
        }
        _ => {
            let Some(g) = &app.game else { return };
            m.top = status_top(Some(g), best, cols, false);
            m.bottom = status_bottom(g, cols);
            match &app.screen {
                Screen::Paused { menu } => {
                    let mut lines = vec![Line::new("Paused", Style::Heading), Line::new("", Style::Normal)];
                    for (i, item) in PAUSE_MENU.iter().enumerate() {
                        lines.push(Line::new(
                            format!("  {item}  "),
                            if i == *menu { Style::Selected } else { Style::Normal },
                        ));
                    }
                    let mut o = Overlay::new(Anchor::Center, lines);
                    o.min_width = 24;
                    m.overlays.push(o);
                }
                Screen::GameOver { name, rank } => {
                    let mut lines = vec![
                        Line::new("Game over", Style::Heading),
                        Line::new(format!("Score {}  ·  eggs laid {}  ·  chicks rescued {}", g.session.score, g.session.eggs, g.session.chicks_saved), Style::Normal),
                        Line::new("", Style::Normal),
                    ];
                    match rank {
                        Some(r) => {
                            lines.push(Line::new(format!("A new best sprint, number {}!", r + 1), Style::Good));
                            lines.push(Line::new(format!("Your name: {name}_"), Style::Accent));
                            lines.push(Line::new("Enter to save", Style::Dim));
                        }
                        None => lines.push(Line::new("Enter: back to the title", Style::Dim)),
                    }
                    let mut o = Overlay::new(Anchor::Center, lines);
                    o.min_width = 30;
                    m.overlays.push(o);
                }
                _ => phase_banner(app, g, m),
            }
        }
    }
}

fn phase_banner(app: &App, g: &Game, m: &mut FrameModel) {
    match g.phase {
        Phase::Ready { .. } => {
            let mut lines = vec![
                Line::new(format!("Level {} · {}", g.level_index + 1 + app.round as usize * super::level_count(), g.level.name), Style::Heading),
                Line::new(g.level.tagline.clone(), Style::Normal),
                Line::new(format!("Lay an egg in each of the {} nest boxes", g.level.nests), Style::Accent),
            ];
            if g.level_index == 0 && app.round == 0 {
                for l in [
                    "",
                    "Arrows hop. Ride things across the river: the water is not for Silkies.",
                    "Space flies over everything, briefly. The wings meter refills slowly, and not between eggs.",
                    "Peck up bugs for points (butterflies and dragonflies only in the air). Be quick: combos!",
                    "Loki and his big sister Skaði want cuddles. Outrun them, or fly.",
                    "Both put you down somewhere safe, but Skaði is the more careful: Loki only just.",
                    "If both grab you at once: tug of war! The winner runs off with you.",
                    "When the hawk's shadow comes: land if you're flying, hop away if you're not.",
                    "Hop up into an empty nest box to lay an egg. Any key to start.",
                ] {
                    lines.push(Line::new(l, Style::Dim));
                }
            }
            let mut o = Overlay::new(Anchor::Center, lines);
            o.alpha = 78;
            m.overlays.push(o);
        }
        Phase::Dying { fate, .. } => {
            let left = g.session.lives.saturating_sub(1);
            let tail = if left == 0 { " That was the last hen.".to_string() } else { format!("  ({left} left)") };
            m.overlays.push(Overlay::new(Anchor::Low, vec![Line::new(format!("{}{tail}", fate.describe()), Style::Bad)]));
        }
        Phase::Nested { .. } => {
            let left = g.nests_left();
            let msg = if left == 0 { "The last egg!".to_string() } else { format!("Egg laid! {left} nest box{} to go.", if left == 1 { "" } else { "es" }) };
            m.overlays.push(Overlay::new(Anchor::Low, vec![Line::new(msg, Style::Good)]));
        }
        Phase::Cleared { .. } => m.overlays.push(Overlay::new(
            Anchor::Center,
            vec![
                Line::new("Every egg has hatched!", Style::Heading),
                Line::new(format!("{} bonus", crate::sim::rules::POINTS_ALL_NESTS), Style::Good),
            ],
        )),
        _ => {
            use crate::sim::kids::{Kid, KidState};
            let tug = g.kids.iter().filter(|c| matches!(c.state, KidState::Tugging { .. })).count() >= 2;
            let msg = if tug {
                Some("Tug of war! Loki and Skaði both want the chicken.".to_string())
            } else {
                g.kids.iter().find(|c| c.holding()).map(|c| match c.kid {
                    Kid::Loki => "Loki: \"Soft chicken!\" (the clock stops for cuddles)".to_string(),
                    Kid::Skadi => "Skaði: \"Gently, gently...\" (she'll put you somewhere really safe)".to_string(),
                })
            };
            if let Some(msg) = msg {
                m.overlays.push(Overlay::new(Anchor::Low, vec![Line::new(msg, Style::Accent)]));
            }
        }
    }
}

fn mark(on: bool) -> &'static str {
    if on { "yes" } else { "no" }
}

/// What the terminal answered, for `--report`, the title screen's C key
/// and the diagnostics panel.
pub fn capability_report(c: &Capabilities) -> Vec<String> {
    let mut v = vec!["Terminal capabilities (as answered by the terminal)".to_string()];
    v.push(format!("graphics protocol: {}   temp-file medium: {}", mark(c.graphics), mark(c.temp_file)));
    v.push(format!(
        "kitty keyboard: {}   synchronized output: {}   SGR pixel mouse: {}",
        c.kitty_keyboard.map_or("no".to_string(), |f| format!("yes (flags {f})")),
        mark(c.sync_output),
        mark(c.sgr_pixels)
    ));
    v.push(format!(
        "cell: {}   window: {}",
        c.cell_px.map_or("?".into(), |(w, h)| format!("{w}x{h} px")),
        c.window_px.map_or("?".into(), |(w, h)| format!("{w}x{h} px"))
    ));
    match &c.listing {
        Some(l) => v.push(format!(
            "OSC 777 extensions: {}",
            l.iter().map(|(k, ver)| format!("{k}.{ver}")).collect::<Vec<_>>().join(" ")
        )),
        None => v.push("OSC 777 extensions: none listed".into()),
    }
    v.push(format!(
        "vector images: {}   HDR images: {}   wide-gamut images: {}",
        mark(c.vector),
        mark(c.hdr_images),
        mark(c.wide_gamut_images)
    ));
    v.push(format!(
        "pointer shapes: crosshair {}   image pointers {}",
        mark(c.pointer_crosshair),
        mark(c.pointer_image)
    ));
    for (name, s) in &c.selectors {
        let fields: Vec<String> = s
            .fields
            .iter()
            .filter(|(k, _)| !matches!(k.as_str(), "v" | "op" | "id"))
            .map(|(k, val)| format!("{k}={val}"))
            .collect();
        v.push(format!("  {name}: {}", fields.join(" ")));
    }
    if let Some(d) = &c.display {
        v.push(format!(
            "display: {} {} {}-bit gamut={} headroom={} scale={} refresh={} ppi={}",
            d.kind,
            d.color,
            d.bits,
            d.gamut,
            d.headroom,
            d.scale,
            if d.period_us > 0 { format!("{:.1} Hz", 1e6 / d.period_us as f64) } else { "?".into() },
            d.ppi
        ));
    }
    for (what, d) in &c.timings {
        v.push(format!("discovery {what}: {:.1} ms", d.as_secs_f64() * 1000.0));
    }
    if c.timed_out {
        v.push("(some queries timed out)".into());
    }
    v
}
