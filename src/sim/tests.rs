//! Headless simulation tests with fixed seeds.

use super::autopilot::Autopilot;
use super::kids::{Kid, KidState};
use super::replay::Recording;
use super::*;
use crate::world::level::{self, parse};

fn game(text: &str) -> Game {
    let mut g = Game::new(parse(text).unwrap(), 0, Session::default(), 7);
    g.phase = Phase::Playing;
    g
}

fn run(g: &mut Game, input: TickInput, ticks: u32) {
    g.step(&input);
    for _ in 1..ticks {
        g.step(&TickInput::default());
    }
}

/// Runs until she comes to grief (or `ticks` pass), and says how.
fn fate(g: &mut Game, input: TickInput, ticks: u32) -> Option<Fate> {
    g.drain_events();
    g.step(&input);
    for _ in 0..ticks {
        for e in g.drain_events() {
            if let SimEvent::Died { fate, .. } = e {
                return Some(fate);
            }
        }
        g.step(&TickInput::default());
    }
    None
}

fn hop(d: Dir) -> TickInput {
    TickInput { hop: Some(d), flap: false }
}

fn flap() -> TickInput {
    TickInput { hop: None, flap: true }
}

const QUIET: &str = "width: 10\nchicks: no\nlanes:\nnests\nverge\nverge\nstart\n";

#[test]
fn hops_move_one_tile_and_score_new_rows() {
    let mut g = game(QUIET);
    let y0 = g.hen.pos.y;
    run(&mut g, hop(Dir::Up), 20);
    assert!((g.hen.pos.y - (y0 - 1.0)).abs() < 1e-4);
    assert_eq!(g.session.score, POINTS_ROW);
    // Going back and forth again scores nothing.
    run(&mut g, hop(Dir::Down), 20);
    run(&mut g, hop(Dir::Up), 20);
    assert_eq!(g.session.score, POINTS_ROW);
}

#[test]
fn a_queued_hop_follows_the_current_one() {
    let mut g = game(QUIET);
    let x0 = g.hen.pos.x;
    g.step(&hop(Dir::Left));
    g.step(&hop(Dir::Left));
    run(&mut g, TickInput::default(), 30);
    assert!((g.hen.pos.x - (x0 - 2.0)).abs() < 1e-4);
}

#[test]
fn the_hen_stays_on_the_field() {
    let mut g = game(QUIET);
    for _ in 0..20 {
        run(&mut g, hop(Dir::Right), 12);
    }
    assert!(g.hen.pos.x <= g.width() - EDGE_MARGIN + 1e-4);
    run(&mut g, hop(Dir::Down), 12);
    assert_eq!(g.hen.row(), g.level.start_row());
}

#[test]
fn cars_run_her_over_but_not_when_she_flies() {
    let text = "width: 10\nchicks: no\nlanes:\nnests\nroad speed=2 limo8@0\nstart\n";
    let mut g = game(text);
    run(&mut g, hop(Dir::Up), 200);
    assert!(matches!(g.phase, Phase::Dying { fate: Fate::Squashed(_), .. }), "{:?}", g.phase);

    let mut g = game(text);
    run(&mut g, flap(), 20);
    assert!(g.hen.airborne());
    run(&mut g, hop(Dir::Up), 30);
    assert_eq!(g.phase, Phase::Playing);
}

#[test]
fn the_meter_runs_out_and_refills() {
    let mut g = game(QUIET);
    run(&mut g, flap(), 10);
    let flight_ticks = (g.level.flight / DT) as u32;
    run(&mut g, TickInput::default(), flight_ticks + 30);
    assert!(!g.hen.flying && g.hen.alt == 0.0, "came down when the meter emptied");
    assert!(g.hen.meter < 0.1);
    // Too tired to take off again at once.
    run(&mut g, flap(), 5);
    assert!(!g.hen.flying);
    let n = ((g.level.recharge + RECHARGE_DELAY) / DT) as u32 + 10;
    run(&mut g, TickInput::default(), n);
    assert!(g.hen.meter >= 1.0);
}

#[test]
fn water_drowns_and_floats_carry() {
    let text = "width: 12\nchicks: no\nlanes:\nnests\nriver speed=1 log12@0 loop=40\nstart\n";
    let mut g = game(text);
    // Wait for the log to be under her, then hop on and ride.
    let mut boarded = false;
    for _ in 0..2000 {
        let lane = &g.level.lanes[1];
        if crate::world::traffic::float_under(lane, 12, g.hen.pos.x, g.time + HOP_TIME).is_some_and(|s| s.x1 - g.hen.pos.x > 4.0) {
            boarded = true;
            break;
        }
        g.step(&TickInput::default());
    }
    assert!(boarded);
    run(&mut g, hop(Dir::Up), 12);
    let x = g.hen.pos.x;
    run(&mut g, TickInput::default(), 60);
    assert_eq!(g.phase, Phase::Playing);
    assert!((g.hen.pos.x - (x + 1.0)).abs() < 0.05, "carried a tile in a second");
    // Ridden off the edge.
    assert_eq!(fate(&mut g, TickInput::default(), 60 * 12), Some(Fate::SweptAway));

    let text = "width: 12\nchicks: no\nlanes:\nnests\nriver speed=1 log1@0 loop=60\nstart\n";
    let mut g = game(text);
    assert_eq!(fate(&mut g, hop(Dir::Up), 20), Some(Fate::Drowned));
}

#[test]
fn nests_take_one_egg_each_and_clear_the_level() {
    let text = "width: 4\nnests: 2\nchicks: no\nlanes:\nnests\nverge\nstart\n";
    let mut g = game(text);
    // Line up under the left nest (x = 1), then up twice.
    run(&mut g, hop(Dir::Left), 15);
    run(&mut g, hop(Dir::Up), 15);
    run(&mut g, hop(Dir::Up), 15);
    assert!(matches!(g.phase, Phase::Nested { nest: 0, .. }), "{:?} at {:?}", g.phase, g.hen.pos);
    assert_eq!(g.nests[0], Nest::Egg);
    run(&mut g, TickInput::default(), (NESTED_TIME / DT) as u32 + 2);
    assert_eq!(g.phase, Phase::Playing);
    // The same nest again: refused.
    run(&mut g, hop(Dir::Left), 15);
    run(&mut g, hop(Dir::Up), 15);
    run(&mut g, hop(Dir::Up), 15);
    assert_eq!(g.phase, Phase::Playing);
    assert_eq!(g.hen.row(), 1);
    // The other one.
    run(&mut g, hop(Dir::Right), 15);
    run(&mut g, hop(Dir::Right), 15);
    run(&mut g, hop(Dir::Up), 15);
    run(&mut g, TickInput::default(), (NESTED_TIME / DT) as u32 + 2);
    assert!(matches!(g.phase, Phase::Cleared { .. }), "{:?}", g.phase);
    assert!(g.nests.iter().all(|n| *n == Nest::Hatched));
}

#[test]
fn running_out_of_time_costs_a_life_and_lives_run_out() {
    let mut g = game("width: 6\ntime: 5\nchicks: no\nlanes:\nnests\nverge\nstart\n");
    g.session.lives = 2;
    run(&mut g, TickInput::default(), (5.0 / DT) as u32 + 2);
    assert!(matches!(g.phase, Phase::Dying { fate: Fate::TimeUp, .. }));
    run(&mut g, TickInput::default(), (DEATH_TIME / DT) as u32 + 2);
    assert_eq!(g.phase, Phase::Playing);
    assert_eq!(g.session.lives, 1);
    run(&mut g, TickInput::default(), ((5.0 + DEATH_TIME) / DT) as u32 + 4);
    assert_eq!(g.phase, Phase::GameOver);
}

#[test]
fn kites_only_catch_a_flying_hen() {
    let text = "width: 10\nchicks: no\nlanes:\nnests\nverge air=1 ^kite@0 ^kite@3 ^kite@6 ^kite@9 ^kite@12 airloop=14\nstart\n";
    let mut g = game(text);
    run(&mut g, hop(Dir::Up), 400);
    assert_eq!(g.phase, Phase::Playing);
    assert_eq!(fate(&mut g, flap(), 200), Some(Fate::Tangled(crate::world::level::MoverKind::Kite)));
}

#[test]
fn bunting_keeps_her_on_foot() {
    let mut g = game("width: 10\nchicks: no\nlanes:\nnests\nverge bunting\nstart\n");
    run(&mut g, flap(), 20);
    run(&mut g, hop(Dir::Up), 20);
    assert_eq!(g.hen.row(), 2, "cannot fly into bunting");
    run(&mut g, flap(), 40);
    run(&mut g, hop(Dir::Up), 20);
    assert_eq!(g.hen.row(), 1, "walks under it");
    run(&mut g, flap(), 5);
    assert!(!g.hen.flying, "and cannot take off beneath it");
}

#[test]
fn the_hawk_takes_a_hen_who_stays_up_and_misses_one_who_lands() {
    let text = "width: 12\nflight: 20\nhawk: yes\nchicks: no\nlanes:\nnests\nverge\nverge\nstart\n";
    let mut g = game(text);
    g.hawk.as_mut().unwrap().state = hawk::HawkState::Away { t: 0.0 };
    run(&mut g, flap(), ((HAWK_WARN + HAWK_STOOP + 0.6) / DT) as u32);
    assert!(matches!(g.phase, Phase::Dying { fate: Fate::Snatched, .. }), "{:?}", g.phase);

    let mut g = game(text);
    g.hawk.as_mut().unwrap().state = hawk::HawkState::Away { t: 0.0 };
    run(&mut g, flap(), 30);
    assert!(g.hawk.as_ref().unwrap().shadow_visible());
    run(&mut g, flap(), ((HAWK_WARN + HAWK_STOOP) / DT) as u32);
    assert_eq!(g.phase, Phase::Playing);
}

#[test]
fn the_fox_bites_a_hen_who_dawdles_on_its_verge() {
    let mut g = game("width: 12\nchicks: no\nlanes:\nnests\nverge fox\nstart\n");
    g.foxes[0].x = 2.0;
    assert_eq!(fate(&mut g, hop(Dir::Up), 60 * 8), Some(Fate::Bitten));
}

const KIDS_FIELD: &str = "width: 16\nchicks: no\ntime: 90\nlanes:\nnests\nriver speed=1.5 log5@0 log5@9 log4@17\nroad speed=-2 hatchback@0 sofa@6 tractor@12\nroad speed=2.5 limo4@0 hatchback@10\nverge\nroad speed=1.2 tractor@0 tractor@9\nstart\n";

/// The test field with the given children on it, and no bugs to distract.
fn with_kids(kids: &str, seed: u64) -> Game {
    let text = format!("{kids}\n{KIDS_FIELD}");
    let mut g = Game::new(parse(&text).unwrap(), 0, Session::default(), seed);
    g.phase = Phase::Playing;
    g.bug_timer = f32::INFINITY;
    g
}

fn wait_to_be_caught(g: &mut Game) {
    for c in &mut g.kids {
        c.state = KidState::Chasing { t: 30.0 };
    }
    for _ in 0..60 * 20 {
        g.step(&TickInput::default());
        if g.hen.held {
            return;
        }
    }
    panic!("nobody caught a hen standing still: {:?}", g.kids);
}

/// Waits for the hen to be put down; returns where, who by, how many pets.
fn wait_for_drop(g: &mut Game) -> (glam::Vec2, Kid, u32) {
    let mut petted = 0;
    for _ in 0..60 * 15 {
        g.step(&TickInput::default());
        for e in g.drain_events() {
            match e {
                SimEvent::KidPet { .. } => petted += 1,
                SimEvent::KidDrop { kid, pos } => return (pos, kid, petted),
                _ => {}
            }
        }
    }
    panic!("never put down: {:?}", g.kids);
}

fn stays_safe(g: &mut Game, secs: f32, what: &str) {
    for _ in 0..(secs / DT) as u32 - 1 {
        g.step(&TickInput::default());
        assert_eq!(g.phase, Phase::Playing, "{what}");
    }
}

#[test]
fn each_child_puts_her_down_safe_for_their_own_time() {
    for (setting, kid, safe) in [("loki: yes", Kid::Loki, LOKI_SAFE_TIME), ("skadi: yes", Kid::Skadi, SKADI_SAFE_TIME)] {
        assert_eq!(kid.safe_time(), safe);
        for seed in 0..10u64 {
            let mut g = with_kids(setting, seed);
            let start = g.hen.pos;
            wait_to_be_caught(&mut g);
            let clock = g.time_left;
            let (at, by, petted) = wait_for_drop(&mut g);
            assert_eq!(by, kid);
            assert!(petted >= 3, "{petted} pets");
            assert!((g.time_left - clock).abs() < 0.01, "the clock stops for a cuddle");
            assert!(at.distance(start) > 1.0, "seed {seed}: somewhere else than {start:?}: {at:?}");
            assert!(!g.hen.held);
            stays_safe(&mut g, safe, &format!("{kid:?}, seed {seed}: put down at {at:?}"));
        }
    }
}

#[test]
fn both_children_at_once_is_a_tug_of_war() {
    let mut winners = Vec::new();
    for seed in 0..16u64 {
        let mut g = with_kids("loki: yes\nskadi: yes", seed);
        // One child next to her, the other close behind.
        let hen = g.hen.pos;
        g.kids[0].pos = hen + glam::Vec2::new(0.3, -0.2);
        g.kids[1].pos = hen + glam::Vec2::new(-1.6, -0.3);
        for c in &mut g.kids {
            c.prev_pos = c.pos;
            c.state = KidState::Chasing { t: 30.0 };
        }
        let mut tug = false;
        let mut won = None;
        for _ in 0..60 * 8 {
            g.step(&TickInput::default());
            for e in g.drain_events() {
                match e {
                    SimEvent::TugOfWar { .. } => tug = true,
                    SimEvent::TugWon { kid, .. } => won = Some(kid),
                    _ => {}
                }
            }
            if won.is_some() {
                break;
            }
        }
        assert!(tug, "seed {seed}: a tug of war");
        let winner = won.expect("somebody wins");
        winners.push(winner);
        let (_, by, _) = wait_for_drop(&mut g);
        assert_eq!(by, winner, "the winner runs off with her");
        stays_safe(&mut g, winner.safe_time(), &format!("seed {seed}: put down by {winner:?}"));
    }
    assert!(winners.contains(&Kid::Loki) && winners.contains(&Kid::Skadi), "either can win: {winners:?}");
}

#[test]
fn no_child_can_reach_a_flying_hen() {
    let mut g = with_kids("loki: yes\nskadi: yes", 3);
    let hen = g.hen.pos;
    for (i, c) in g.kids.iter_mut().enumerate() {
        c.state = KidState::Chasing { t: 30.0 };
        c.pos = hen + glam::Vec2::new(if i == 0 { 2.0 } else { -2.0 }, 0.0);
    }
    g.hen.meter = 1.0;
    let n = (g.level.flight / DT) as u32 - 10;
    run(&mut g, flap(), n);
    assert!(!g.hen.held);
}

#[test]
fn replays_reproduce_the_game() {
    let lvl = level::builtin(1).unwrap();
    let mut g = Game::new(lvl.clone(), 1, Session::default(), 99);
    let mut rec = Recording::new(&g, 1.0);
    let mut bot = Autopilot::new();
    for _ in 0..60 * 25 {
        let input = bot.decide(&g);
        g.step(&input);
        rec.push(g.tick, &input);
        g.drain_events();
    }
    rec.finish(&g);
    let json = serde_json::to_string(&rec).unwrap();
    let back: Recording = serde_json::from_str(&json).unwrap();
    let again = back.replay(lvl);
    assert_eq!(again.state_hash(), rec.final_hash.unwrap());
}

#[test]
fn the_autopilot_fills_a_nest_on_every_level() {
    for i in 0..level::BUILTIN.len() {
        let lvl = level::builtin(i).unwrap();
        let name = lvl.name.clone();
        let mut g = Game::new(lvl, i, Session::default(), 3);
        let mut bot = Autopilot::new();
        let mut nested = false;
        for _ in 0..60 * 90 {
            let input = bot.decide(&g);
            g.step(&input);
            if g.drain_events().iter().any(|e| matches!(e, SimEvent::Nested { .. })) {
                nested = true;
                break;
            }
        }
        assert!(nested, "level {} {name}: no nest in 90 s (lives {}, phase {:?})", i + 1, g.session.lives, g.phase);
    }
}


#[test]
fn every_level_can_be_crossed_on_foot() {
    for i in 0..level::BUILTIN.len() {
        let lvl = level::builtin(i).unwrap();
        let name = lvl.name.clone();
        let mut g = Game::new(lvl, i, Session::default(), 5);
        let mut bot = Autopilot::new();
        bot.may_fly = false;
        let mut nested = false;
        for _ in 0..60 * 120 {
            let input = bot.decide(&g);
            assert!(!input.flap);
            g.step(&input);
            if g.drain_events().iter().any(|e| matches!(e, SimEvent::Nested { .. })) {
                nested = true;
                break;
            }
        }
        assert!(nested, "level {} {name}: no nest on foot in 120 s (phase {:?})", i + 1, g.phase);
    }
}


#[test]
fn a_key_skips_the_countdown() {
    let mut g = Game::new(parse(QUIET).unwrap(), 0, Session::default(), 7);
    assert!(matches!(g.phase, Phase::Ready { .. }));
    g.step(&hop(Dir::Up));
    assert_eq!(g.phase, Phase::Playing);
    assert!(g.hen.hop.is_some(), "and the key counts");
}

fn bug(g: &mut Game, kind: bugs::BugKind, pos: glam::Vec2) {
    let id = g.alloc_id();
    g.bugs.push(bugs::Bug {
        id,
        kind,
        pos,
        prev_pos: pos,
        vel: glam::Vec2::ZERO,
        life: 60.0,
        timer: 60.0,
        leap: None,
        tired: 0.0,
    });
}

/// A level with no bugs of its own turning up (they are placed by hand).
fn bugless(text: &str) -> Game {
    let mut g = game(text);
    g.bug_timer = f32::INFINITY;
    g
}

#[test]
fn cars_splat_bugs_but_sidewalk_traffic_misses_them() {
    let text = "width: 10\nchicks: no\nlanes:\nnests\nroad speed=3 limo9@0 loop=14\nsidewalk speed=3 bicycle@0 stroller@3 wagon@6 bicycle@9 loop=12\nstart\n";
    let mut g = bugless(text);
    bug(&mut g, bugs::BugKind::Snail, glam::Vec2::new(5.0, 1.5));
    bug(&mut g, bugs::BugKind::Snail, glam::Vec2::new(5.0, 2.5));
    let mut splats = 0;
    for _ in 0..60 * 6 {
        g.step(&TickInput::default());
        splats += g.drain_events().iter().filter(|e| matches!(e, SimEvent::BugSplat { .. })).count();
    }
    assert_eq!(splats, 1, "the snail on the road, not the one on the sidewalk");
    assert_eq!(g.bugs.len(), 1);
    assert_eq!(crate::world::row_of(g.bugs[0].pos.y), 2);
}

#[test]
fn bugs_caught_quickly_make_a_combo() {
    let mut g = bugless(QUIET);
    let at = g.hen.pos;
    for k in 1..=3 {
        bug(&mut g, bugs::BugKind::Snail, at - glam::Vec2::new(k as f32, 0.0));
    }
    let mut caught = Vec::new();
    for _ in 0..3 {
        g.step(&hop(Dir::Left));
        for _ in 0..12 {
            g.step(&TickInput::default());
            for e in g.drain_events() {
                if let SimEvent::BugCaught { points, combo, .. } = e {
                    caught.push((points, combo));
                }
            }
        }
    }
    assert_eq!(caught, vec![(40, 1), (80, 2), (120, 3)]);
    run(&mut g, TickInput::default(), (COMBO_WINDOW / DT) as u32 + 2);
    assert_eq!(g.combo, 0, "the combo lapses");
}

#[test]
fn grasshoppers_leap_away_and_fliers_need_flight() {
    let mut g = bugless(QUIET);
    let at = g.hen.pos - glam::Vec2::new(1.4, 0.0);
    bug(&mut g, bugs::BugKind::Grasshopper, at);
    g.step(&TickInput::default());
    assert!(g.bugs[0].aloft(), "a grasshopper this close leaps");
    run(&mut g, TickInput::default(), 30);
    assert!(g.bugs[0].pos.distance(g.hen.pos) > 2.0, "away from her");

    let mut g = bugless(QUIET);
    let at = g.hen.pos;
    bug(&mut g, bugs::BugKind::Butterfly, at);
    g.bugs[0].timer = 60.0;
    g.step(&TickInput::default());
    assert_eq!(g.bugs.len(), 1, "out of reach on foot");
    let mut caught = false;
    g.step(&flap());
    for _ in 0..30 {
        g.step(&TickInput::default());
        caught |= g.drain_events().iter().any(|e| matches!(e, SimEvent::BugCaught { .. }));
    }
    assert!(caught, "caught in the air");
}

#[test]
fn the_hawk_takes_a_loiterer_unless_she_moves_or_hides() {
    let text = "width: 12\nhawk: yes\nchicks: no\ntime: 90\nlanes:\nnests\nverge\nverge bunting\nverge\nstart\n";
    // Standing about on the grass: taken.
    let mut g = bugless(text);
    g.hawk.as_mut().unwrap().state = hawk::HawkState::Away { t: 0.0 };
    run(&mut g, hop(Dir::Up), 12);
    assert_eq!(fate(&mut g, TickInput::default(), 60 * 8), Some(Fate::Snatched));

    // Hopping away once the shadow comes: missed.
    let mut g = bugless(text);
    g.hawk.as_mut().unwrap().state = hawk::HawkState::Away { t: 0.0 };
    run(&mut g, hop(Dir::Up), 12);
    while !g.hawk.as_ref().unwrap().shadow_visible() {
        g.step(&TickInput::default());
    }
    for _ in 0..3 {
        run(&mut g, hop(Dir::Right), 12);
    }
    assert_eq!(fate(&mut g, TickInput::default(), ((HAWK_WARN + HAWK_STOOP) / DT) as u32), None);

    // Under the bunting it never comes at all.
    let mut g = bugless(text);
    g.hawk.as_mut().unwrap().state = hawk::HawkState::Away { t: 0.0 };
    run(&mut g, hop(Dir::Up), 12);
    run(&mut g, hop(Dir::Up), 12);
    assert_eq!(g.hen.row(), 2);
    assert_eq!(fate(&mut g, TickInput::default(), 60 * 8), None);
    assert!(!g.hawk.as_ref().unwrap().shadow_visible());
}

#[test]
fn the_meter_carries_over_from_one_crossing_to_the_next() {
    let text = "width: 4\nnests: 2\nchicks: no\nflight: 2\nrecharge: 20\nlanes:\nnests\nverge\nstart\n";
    let mut g = bugless(text);
    // Fly up into the left nest, using most of the meter.
    run(&mut g, flap(), 20);
    run(&mut g, hop(Dir::Left), 15);
    run(&mut g, hop(Dir::Up), 15);
    run(&mut g, hop(Dir::Up), 15);
    assert!(matches!(g.phase, Phase::Nested { .. }), "{:?}", g.phase);
    let left = g.hen.meter;
    assert!(left < 0.8, "{left}");
    run(&mut g, TickInput::default(), (NESTED_TIME / DT) as u32 + 2);
    assert_eq!(g.phase, Phase::Playing);
    assert!((g.hen.meter - left).abs() < 0.02, "the next hen starts with what was left, {} vs {left}", g.hen.meter);
    // And losing a hen does not refill it either.
    g.time_left = 0.01;
    run(&mut g, TickInput::default(), (DEATH_TIME / DT) as u32 + 5);
    assert_eq!(g.phase, Phase::Playing);
    assert!(g.hen.meter < 0.85, "{}", g.hen.meter);
}

#[test]
fn the_children_are_always_on_the_field() {
    for seed in 0..4u64 {
        let mut g = Game::new(level::builtin(1).unwrap(), 1, Session { lives: 99, ..Default::default() }, seed);
        assert_eq!(g.kids.len(), 2, "Loki and Skaði");
        let mut bot = Autopilot::new();
        let (w, h) = (g.width(), g.level.height() as f32);
        let mut chases = 0;
        for _ in 0..60 * 90 {
            let input = bot.decide(&g);
            g.step(&input);
            chases += g.drain_events().iter().filter(|e| matches!(e, SimEvent::KidChasing { .. })).count();
            for c in &g.kids {
                assert!(c.pos.x >= 0.0 && c.pos.x <= w && c.pos.y >= 1.0 && c.pos.y <= h, "seed {seed}: {:?} at {:?}", c.kid, c.pos);
            }
            if g.outcome().is_some() {
                break;
            }
        }
        assert!(chases >= 1, "seed {seed}: they come after her");
    }
}


