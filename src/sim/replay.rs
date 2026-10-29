//! Input recordings: the seed, the level and every input, enough to
//! reproduce a game exactly with a headless run.

use super::{Game, Session, TickInput};
use crate::world::level::Level;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Recording {
    pub version: u32,
    pub seed: u64,
    pub level_index: usize,
    pub level_name: String,
    /// Speed-up of a later round, applied to the level before play.
    pub quicken: f32,
    pub session: Session,
    /// Ticks with any input (presses; nothing is held).
    pub inputs: Vec<(u64, TickInput)>,
    pub ticks: u64,
    pub final_hash: Option<u64>,
}

impl Recording {
    pub fn new(game: &Game, quicken: f32) -> Self {
        Recording {
            version: FORMAT_VERSION,
            seed: game.seed,
            level_index: game.level_index,
            level_name: game.level.name.clone(),
            quicken,
            session: game.session.clone(),
            inputs: Vec::new(),
            ticks: 0,
            final_hash: None,
        }
    }

    /// Records the input applied at `tick` (1-based, as `Game::tick` after
    /// the step).
    pub fn push(&mut self, tick: u64, input: &TickInput) {
        if !input.is_idle() {
            self.inputs.push((tick, input.clone()));
        }
        self.ticks = tick;
    }

    pub fn finish(&mut self, game: &Game) {
        self.final_hash = Some(game.state_hash());
    }

    /// Re-runs the recording, returning the finished game.
    pub fn replay(&self, mut level: Level) -> Game {
        if self.quicken != 1.0 {
            level.quicken(self.quicken);
        }
        let mut game = Game::new(level, self.level_index, self.session.clone(), self.seed);
        let mut next = 0;
        let idle = TickInput::default();
        for tick in 1..=self.ticks {
            let input = if next < self.inputs.len() && self.inputs[next].0 == tick {
                next += 1;
                &self.inputs[next - 1].1
            } else {
                &idle
            };
            game.step(input);
            game.drain_events();
        }
        game
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let text = serde_json::to_string(self).map_err(std::io::Error::other)?;
        std::fs::write(path, text)
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        serde_json::from_str(&text).map_err(std::io::Error::other)
    }
}
