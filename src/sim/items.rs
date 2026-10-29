//! Treats that turn up on the verges (corn, and golden feathers that fill
//! the wings meter), and stray chicks that drift down
//! the river on whatever they could climb onto. A chick rides home on the
//! hen's back for a bonus when she reaches a nest.

use super::rules::*;
use super::{Emote, Game, SimEvent};
use crate::world::level::{LaneKind, MoverKind};
use crate::world::{row_center, traffic};
use glam::Vec2;
use rand::RngExt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    Corn,
    /// Fills the flight meter.
    Feather,
}

impl ItemKind {
    pub fn points(self) -> u32 {
        match self {
            ItemKind::Corn => POINTS_CORN,
            ItemKind::Feather => POINTS_FEATHER,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub kind: ItemKind,
    pub pos: Vec2,
    pub life: f32,
}

/// A chick riding a float: which lane, which mover, and its plumage.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrayChick {
    pub id: u32,
    pub row: i32,
    pub mover: usize,
    pub color: u8,
}

impl StrayChick {
    pub fn pos(&self, g: &Game) -> Option<Vec2> {
        let lane = g.level.lane(self.row)?;
        let s = traffic::ground(lane, g.level.width, g.time).nth(self.mover)?;
        Some(Vec2::new(s.center(), row_center(self.row)))
    }
}

pub fn update(g: &mut Game) {
    update_items(g);
    update_stray(g);
}

fn update_items(g: &mut Game) {
    let hen = g.hen.pos;
    let can_pick = !g.hen.airborne() && !g.hen.held;
    let mut i = 0;
    while i < g.items.len() {
        g.items[i].life -= DT;
        let it = g.items[i].clone();
        if can_pick && it.pos.distance(hen) < PICKUP_DIST {
            g.items.remove(i);
            let points = it.kind.points();
            g.add_score(points);
            if it.kind == ItemKind::Feather {
                g.hen.meter = 1.0;
            }
            g.emit(SimEvent::ItemCollected {
                item: it.id,
                kind: it.kind,
                points,
                pos: it.pos,
            });
            continue;
        }
        if it.life <= 0.0 {
            g.items.remove(i);
            g.emit(SimEvent::ItemExpired { item: it.id });
            continue;
        }
        i += 1;
    }
    g.item_timer -= DT;
    if g.item_timer > 0.0 {
        return;
    }
    g.item_timer = g.rng.random_range(ITEM_EVERY_MIN..ITEM_EVERY_MAX);
    let rows: Vec<i32> = g
        .level
        .lanes
        .iter()
        .enumerate()
        .filter(|(_, l)| l.items)
        .map(|(r, _)| r as i32)
        .collect();
    if rows.is_empty() {
        return;
    }
    let row = rows[g.rng.random_range(0..rows.len())];
    let x = g.rng.random_range(0.0..g.level.width as f32).floor() + 0.5;
    let pos = Vec2::new(x, row_center(row));
    if pos.distance(hen) < 1.5 || g.items.iter().any(|it| it.pos.distance(pos) < 0.5) {
        return;
    }
    let kind = if g.rng.random_bool(0.6) { ItemKind::Corn } else { ItemKind::Feather };
    let id = g.alloc_id();
    g.items.push(Item {
        id,
        kind,
        pos,
        life: ITEM_LIFE,
    });
    g.emit(SimEvent::ItemSpawned { item: id, pos });
}

fn update_stray(g: &mut Game) {
    if let Some(s) = g.stray.clone() {
        let Some(pos) = s.pos(g) else {
            g.stray = None;
            return;
        };
        let hen = g.hen.pos;
        let on_field = pos.x > 0.3 && pos.x < g.width() - 0.3;
        if on_field && !g.hen.airborne() && !g.hen.held && g.hen.carrying.is_none() && pos.distance(hen) < 0.7 {
            g.hen.carrying = Some(s.color);
            g.stray = None;
            g.stray_timer = g.rng.random_range(CHICK_EVERY_MIN..CHICK_EVERY_MAX);
            g.emit(SimEvent::ChickPicked { pos });
            g.emit(SimEvent::Emote {
                entity: g.hen.id,
                emote: Emote::Heart,
            });
        }
        return;
    }
    if !g.level.chicks || g.hen.carrying.is_some() {
        return;
    }
    g.stray_timer -= DT;
    if g.stray_timer > 0.0 {
        return;
    }
    g.stray_timer = g.rng.random_range(CHICK_EVERY_MIN..CHICK_EVERY_MAX);
    // Anything floating that does not dive will do.
    let mut rides: Vec<(i32, usize)> = Vec::new();
    for (r, lane) in g.level.lanes.iter().enumerate() {
        if lane.kind != LaneKind::River {
            continue;
        }
        for (i, m) in lane.movers.iter().enumerate() {
            if m.kind != MoverKind::Divers {
                rides.push((r as i32, i));
            }
        }
    }
    if rides.is_empty() {
        return;
    }
    let (row, mover) = rides[g.rng.random_range(0..rides.len())];
    let color = g.rng.random_range(0..4u8);
    let id = g.alloc_id();
    let s = StrayChick { id, row, mover, color };
    if let Some(pos) = s.pos(g) {
        g.emit(SimEvent::ChickAppeared { pos });
    }
    g.stray = Some(s);
}
