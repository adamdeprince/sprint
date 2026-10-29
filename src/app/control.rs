//! Keys, mouse, pen and touch.
//!
//! A movement key is a hop; a held key repeats (as the terminal repeats
//! it), so the game plays the same whether or not the terminal reports key
//! releases. Space takes off and comes down again. With a mouse, a pen or a
//! finger, tap the hen to take off or land, and tap anywhere else to hop
//! that way.

use super::{App, Screen};
use crate::term::input::{KeyCode, KeyEvent, KeyKind, MouseButton, MouseEvent, MouseKind};
use crate::term::osc777::Frame;
use crate::world::Dir;
use glam::Vec2;

pub const PAUSE_MENU: [&str; 4] = ["Resume", "Restart level", "Quit to title", "Quit game"];

fn direction(code: KeyCode) -> Option<Dir> {
    match code {
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Char('k') => Some(Dir::Up),
        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Char('j') => Some(Dir::Down),
        KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Char('h') => Some(Dir::Left),
        KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Char('l') => Some(Dir::Right),
        _ => None,
    }
}

impl App {
    pub fn on_key(&mut self, k: KeyEvent) {
        if k.kind == KeyKind::Release {
            return;
        }
        let ch = match k.code {
            KeyCode::Char(c) => Some(c.to_ascii_lowercase()),
            _ => None,
        };
        if k.mods.ctrl() && ch == Some('c') {
            self.quit = true;
            return;
        }
        if k.mods.ctrl() && ch == Some('z') {
            if let Err(e) = self.suspend() {
                tracing::error!("suspend: {e}");
            }
            return;
        }
        if k.code == KeyCode::F(2) || ch == Some('`') || ch == Some('~') {
            self.show_diag = !self.show_diag;
            return;
        }
        match &mut self.screen {
            Screen::Title { .. } => match (k.code, ch) {
                (KeyCode::Enter, _) | (_, Some(' ')) => self.start_game(),
                (KeyCode::Escape, _) | (_, Some('q')) => self.quit = true,
                (_, Some('c')) => self.screen = Screen::Report,
                _ => {}
            },
            Screen::Report => self.go_title(),
            Screen::Play => {
                if let Some(d) = direction(k.code) {
                    self.hops.push_back(d);
                    self.note_key();
                    return;
                }
                match (k.code, ch) {
                    (_, Some(' ')) | (KeyCode::Enter, _) | (_, Some('f')) => {
                        if k.kind == KeyKind::Press {
                            self.flap = true;
                            self.note_key();
                        }
                    }
                    (_, Some('p')) | (KeyCode::Escape, _) => self.pause(),
                    _ => {}
                }
            }
            Screen::Paused { menu } => match (k.code, ch) {
                (KeyCode::Up, _) | (_, Some('w')) => *menu = (*menu + PAUSE_MENU.len() - 1) % PAUSE_MENU.len(),
                (KeyCode::Down, _) | (_, Some('s')) => *menu = (*menu + 1) % PAUSE_MENU.len(),
                (_, Some('p')) | (KeyCode::Escape, _) => self.resume(),
                (KeyCode::Enter, _) | (_, Some(' ')) => {
                    let m = *menu;
                    self.pause_menu(m);
                }
                (_, Some('q')) => self.quit = true,
                _ => {}
            },
            Screen::GameOver { name, rank } => {
                if rank.is_some() {
                    match k.code {
                        KeyCode::Char(c) if (c.is_alphanumeric() || c == ' ' || c == '-') && name.chars().count() < 12 && !k.mods.ctrl() => {
                            name.push(c)
                        }
                        KeyCode::Backspace => {
                            name.pop();
                        }
                        KeyCode::Enter => self.save_score(),
                        KeyCode::Escape => self.go_title(),
                        _ => {}
                    }
                } else if matches!(k.code, KeyCode::Enter | KeyCode::Escape) || ch == Some(' ') {
                    self.go_title();
                }
            }
        }
    }

    pub fn pause(&mut self) {
        self.screen = Screen::Paused { menu: 0 };
        self.hops.clear();
        self.flap = false;
        self.set_low_latency(false);
    }

    fn resume(&mut self) {
        self.screen = Screen::Play;
        self.set_low_latency(true);
    }

    fn pause_menu(&mut self, item: usize) {
        match item {
            0 => self.resume(),
            1 => {
                if let Some(g) = self.game.take() {
                    let mut s = g.session.clone();
                    s.lives = s.lives.saturating_sub(1).max(1);
                    self.load_level(g.level_index, s);
                }
                self.resume();
            }
            2 => self.go_title(),
            _ => self.quit = true,
        }
    }

    fn save_score(&mut self) {
        if let (Screen::GameOver { name, .. }, Some(g)) = (&self.screen, &self.game) {
            let name = if name.trim().is_empty() { "Hen".to_string() } else { name.trim().to_string() };
            let entry = super::scores::Entry {
                name,
                score: g.session.score,
                level: g.level_index as u32 + 1 + self.round * super::level_count() as u32,
                chicks: g.session.eggs,
            };
            if let Err(e) = self.scores.add(entry) {
                tracing::warn!("saving high scores: {e}");
            }
        }
        self.go_title();
    }

    fn world_at(&self, px: Option<(i32, i32)>, col: u32, row: u32) -> Option<Vec2> {
        let l = self.renderer.layout?;
        let (x, y) = match px {
            Some((x, y)) => (x as f32, y as f32),
            None => ((col as f32 + 0.5) * l.cell_w as f32, (row as f32 + 0.5) * l.cell_h as f32),
        };
        Some(l.px_to_world(x, y))
    }

    /// A tap on the field while playing: on the hen, fly (or land);
    /// elsewhere, hop towards it.
    fn tap(&mut self, p: Vec2) {
        let Some(g) = &self.game else { return };
        // Where the hen appears, not just where her feet are.
        let drawn = g.hen.pos - Vec2::new(0.0, 0.25 + g.hen.alt * 0.85);
        let d = p - drawn;
        if d.length() < 0.7 {
            self.flap = true;
        } else if let Some(dir) = Dir::from_vec(d) {
            self.hops.push_back(dir);
        }
        self.note_key();
    }

    pub fn on_mouse(&mut self, m: MouseEvent) {
        let pos = self.world_at(m.px, m.col, m.row);
        let press = matches!(m.kind, MouseKind::Press(MouseButton::Left));
        match &self.screen {
            Screen::Title { .. } if press => self.start_game(),
            Screen::Paused { .. } if press => self.resume(),
            Screen::Report if matches!(m.kind, MouseKind::Press(_)) => self.go_title(),
            Screen::Play => match (m.kind, pos) {
                (MouseKind::Press(MouseButton::Left), Some(p)) => self.tap(p),
                (MouseKind::Press(MouseButton::Right), _) => self.flap = true,
                _ => {}
            },
            _ => {}
        }
    }

    /// A report of the tablet pointer protocol: a pen (tools) or a finger
    /// on a touchscreen. Positions are 1/256 physical pixel. Touching down
    /// is a tap.
    pub fn on_pointer_report(&mut self, f: &Frame) {
        let kind = f.get_u64("kind").unwrap_or(0);
        if kind != 1 && kind != 2 {
            return;
        }
        if kind == 2 && f.get_u64("src") == Some(1) {
            // Fingers on a touchpad are positions on the pad, not the
            // screen.
            return;
        }
        if f.get_u64("phase") != Some(1) {
            return;
        }
        let x = f.get_i64("x").unwrap_or(0) as f32 / 256.0;
        let y = f.get_i64("y").unwrap_or(0) as f32 / 256.0;
        let pos = self.renderer.layout.map(|l| l.px_to_world(x, y));
        if let Some(t) = f.get_u64("t") {
            self.note_pen(t);
        }
        match (&self.screen, pos) {
            (Screen::Play, Some(p)) => self.tap(p),
            (Screen::Title { .. }, _) => self.start_game(),
            (Screen::Paused { .. }, _) => self.resume(),
            _ => {}
        }
    }
}
