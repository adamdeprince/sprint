//! Silkie Sprint: hop the traffic, ride the river, fly when you must.

use clap::{Parser, Subcommand};
use silkie_sprint::app::{self, Options, show::Show};
use silkie_sprint::render::snapshot::snapshot;
use silkie_sprint::sim::autopilot::Autopilot;
use silkie_sprint::sim::replay::Recording;
use silkie_sprint::sim::{Game, Phase, Session, SimEvent};
use silkie_sprint::world::level;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "A lane-crossing arcade game about a Silkie hen who can (briefly) fly.")]
struct Cli {
    /// Random seed (the same seed and inputs replay the same game).
    #[arg(long)]
    seed: Option<u64>,
    /// Level to start on (1-based).
    #[arg(long, default_value_t = 1)]
    level: usize,
    /// Show the diagnostics panel from the start (toggle with F2 or `).
    #[arg(long)]
    diagnostics: bool,
    /// Use only what plain kitty offers, even when the terminal has more.
    #[arg(long)]
    kitty_only: bool,
    /// Draw the scenery as bitmaps even where vector images are available.
    #[arg(long)]
    no_vector: bool,
    /// Do not ask for the terminal's low-latency mode.
    #[arg(long)]
    no_latency: bool,
    /// Pace frames by timer even where presentation reports exist.
    #[arg(long)]
    no_frame_feedback: bool,
    /// No haptic feedback.
    #[arg(long)]
    no_haptics: bool,
    /// No HDR glints, even where the terminal has the headroom.
    #[arg(long)]
    no_hdr: bool,
    /// Never send images through temporary files (as over SSH).
    #[arg(long)]
    no_tempfile: bool,
    /// Record the inputs of the game to this file, for `replay`.
    #[arg(long)]
    record: Option<PathBuf>,
    /// Print what the terminal supports and exit.
    #[arg(long)]
    report: bool,
    /// Let the autopilot play the hen (a demo).
    #[arg(long)]
    autoplay: bool,
    /// Write a log here (the terminal itself is busy with the game).
    #[arg(long)]
    log: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Re-runs a recording without a terminal and checks it ends in the same
    /// state.
    Replay { file: PathBuf },
    /// Lets the autopilot play every level without a terminal and prints how
    /// it went.
    Headless {
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// Seconds of play per level at most.
        #[arg(long, default_value_t = 300)]
        seconds: u32,
        /// Round (0 is the first time through; later rounds are quicker).
        #[arg(long, default_value_t = 0)]
        round: u32,
    },
    /// Lets the autopilot play a level for a while and writes a picture of
    /// the moment, as the terminal would show it.
    Snapshot {
        /// Level (1-based).
        #[arg(long, default_value_t = 1)]
        level: usize,
        /// Seconds of play before the picture.
        #[arg(long, default_value_t = 6.0)]
        at: f32,
        #[arg(long, default_value_t = 48)]
        tile_px: u32,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// The hen stands still instead of the autopilot playing (to see
        /// what comes for her).
        #[arg(long)]
        idle: bool,
        out: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Some(path) = &cli.log {
        if let Ok(file) = std::fs::File::create(path) {
            tracing_subscriber::fmt()
                .with_writer(std::sync::Mutex::new(file))
                .with_ansi(false)
                .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
                .init();
        }
    }
    let result = match cli.command {
        Some(Command::Replay { file }) => replay(&file),
        Some(Command::Headless { seed, seconds, round }) => {
            headless(seed, seconds, round);
            Ok(())
        }
        Some(Command::Snapshot { level, at, tile_px, seed, idle, out }) => picture(level, at, tile_px, seed, idle, &out),
        None => app::run(Options {
            seed: cli.seed,
            level: cli.level.saturating_sub(1),
            diagnostics: cli.diagnostics,
            kitty_only: cli.kitty_only,
            no_vector: cli.no_vector,
            no_latency: cli.no_latency,
            no_frame_feedback: cli.no_frame_feedback,
            no_haptics: cli.no_haptics,
            no_hdr: cli.no_hdr,
            no_tempfile: cli.no_tempfile,
            record: cli.record,
            report: cli.report,
            autoplay: cli.autoplay,
        }),
    };
    if let Err(e) = result {
        eprintln!("silkie-sprint: {e:#}");
        std::process::exit(1);
    }
}

fn replay(file: &std::path::Path) -> anyhow::Result<()> {
    let rec = Recording::load(file)?;
    let lvl = level::builtin(rec.level_index).ok_or_else(|| anyhow::anyhow!("recording names level {} which does not exist", rec.level_index))?;
    let g = rec.replay(lvl);
    let hash = g.state_hash();
    println!(
        "{}: {} ticks, {} inputs, score {}, final state {:016x}",
        rec.level_name,
        rec.ticks,
        rec.inputs.len(),
        g.session.score,
        hash
    );
    match rec.final_hash {
        Some(h) if h == hash => println!("replay matches the recorded game"),
        Some(h) => anyhow::bail!("replay diverged: recorded {h:016x}"),
        None => println!("(the recording has no final state to compare)"),
    }
    Ok(())
}

fn headless(seed: u64, seconds: u32, round: u32) {
    println!("| level | outcome | nests | hens lost | flights | cuddles | bugs | time (s) | score | mean tick (µs) | max tick (µs) |");
    println!("|---|---|---|---|---|---|---|---|---|---|---|");
    for i in 0..level::BUILTIN.len() {
        let lvl = app::level_for(i, round);
        let name = lvl.name.clone();
        let session = Session {
            lives: 99,
            ..Default::default()
        };
        let mut g = Game::new(lvl, i, session, seed);
        let mut bot = Autopilot::new();
        let mut total = std::time::Duration::ZERO;
        let mut max = std::time::Duration::ZERO;
        let mut ticks = 0u64;
        let mut deaths = 0;
        let mut cuddles = 0;
        let mut flights = 0;
        let mut bugs = 0;
        while g.play_time < seconds as f32 && g.outcome().is_none() {
            let input = bot.decide(&g);
            g.step(&input);
            for e in g.drain_events() {
                match e {
                    SimEvent::Died { .. } => deaths += 1,
                    SimEvent::KidCaught { .. } => cuddles += 1,
                    SimEvent::TakeOff { .. } => flights += 1,
                    SimEvent::BugCaught { .. } => bugs += 1,
                    _ => {}
                }
            }
            total += g.stats.last_tick;
            max = max.max(g.stats.last_tick);
            ticks += 1;
        }
        let outcome = match g.phase {
            Phase::Cleared { .. } => "cleared",
            Phase::GameOver => "game over",
            _ => "time up",
        };
        println!(
            "| {} {} | {} | {}/{} | {} | {} | {} | {} | {:.0} | {} | {:.1} | {:.1} |",
            i + 1,
            name,
            outcome,
            g.session.eggs,
            g.level.nests,
            deaths,
            flights,
            cuddles,
            bugs,
            g.play_time,
            g.session.score,
            total.as_secs_f64() * 1e6 / ticks.max(1) as f64,
            max.as_secs_f64() * 1e6
        );
    }
}

fn picture(level_no: usize, at: f32, tile_px: u32, seed: u64, idle: bool, out: &std::path::Path) -> anyhow::Result<()> {
    let index = level_no.saturating_sub(1);
    let lvl = level::builtin(index).ok_or_else(|| anyhow::anyhow!("there is no level {level_no}"))?;
    let mut g = Game::new(lvl, index, Session::default(), seed);
    g.session.lives = 99;
    let mut bot = Autopilot::new();
    let mut show = Show::new();
    while g.play_time < at {
        let input = if idle { Default::default() } else { bot.decide(&g) };
        g.step(&input);
        show.update(silkie_sprint::sim::rules::DT);
        for e in g.drain_events() {
            show.on_event(&e);
        }
    }
    let sprites = show.sprites(&g, 1.0);
    let pm = snapshot(&g.level, &Show::decals(&g, 1.0), &sprites, tile_px);
    pm.save_png(out)?;
    println!("{}: {} sprites at {:.1} s, phase {:?}", g.level.name, sprites.len(), g.play_time, g.phase);
    Ok(())
}
