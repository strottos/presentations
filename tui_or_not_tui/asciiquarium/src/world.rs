//! The aquarium: owns everything in the tank, steps the simulation, and draws it.

use std::cmp::Reverse;

use rand::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::art::{self, Sprite, TRANSPARENT};
use crate::entity::{Entity, Kind};
use crate::fish::{BIG_SIZE, Ctx, FoodSnap, Fish, Mood, Out, Partner, V, iso};

/// Rows of open air above the waterline (ships, ducks and whale spouts live here).
pub const SKY: i32 = 6;
/// Rows taken up by the waterline itself.
pub const WAVE_ROWS: i32 = 4;
pub const MIN_W: u16 = 40;
pub const MIN_H: u16 = 20;

const FISH_COLORS: [Color; 12] = [
    Color::Cyan,
    Color::LightCyan,
    Color::Red,
    Color::LightRed,
    Color::Yellow,
    Color::LightYellow,
    Color::Green,
    Color::LightGreen,
    Color::Magenta,
    Color::LightMagenta,
    Color::Blue,
    Color::LightBlue,
];

#[derive(Default, Clone, Copy)]
pub struct Stats {
    pub fed: u32,
    pub eaten: u32,
    pub born: u32,
}

pub struct World {
    w: i32,
    h: i32,
    time: f32,
    entities: Vec<Entity>,
    fish: Vec<Fish>,
    rng: ThreadRng,
    next_id: u32,
    /// Seconds until the next special visitor, only counts down while none is around.
    special_cooldown: f32,
    arrival_cd: f32,
    pub stats: Stats,
    pub paused: bool,
    pub show_help: bool,
}

impl World {
    pub fn new(w: u16, h: u16) -> Self {
        let mut world = World {
            w: w as i32,
            h: h as i32,
            time: 0.0,
            entities: Vec::new(),
            fish: Vec::new(),
            rng: rand::rng(),
            next_id: 0,
            special_cooldown: 3.0,
            arrival_cd: 0.0,
            stats: Stats::default(),
            paused: false,
            show_help: false,
        };
        world.populate();
        world
    }

    /// Rebuild the scene for a new terminal size.
    pub fn resize(&mut self, w: u16, h: u16) {
        self.w = w as i32;
        self.h = h as i32;
        self.entities.clear();
        self.fish.clear();
        self.stats = Stats::default();
        self.populate();
    }

    fn populate(&mut self) {
        self.add_castle();
        for _ in 0..self.seaweed_target() {
            self.add_seaweed();
        }
        for _ in 0..self.fish_target() {
            let species = self.pick_species();
            let (x, y) = (
                self.rng.random_range(0..self.w.max(1)) as f32,
                self.rng.random_range(0..self.h.max(1)) as f32,
            );
            let right = self.rng.random_bool(0.5);
            let i = self.add_fish_at(species, true, x, y, right);
            self.fish[i].hunger = self.rng.random_range(0.1..0.6);
        }
    }

    fn fish_target(&self) -> usize {
        let water = (self.h - SKY - WAVE_ROWS - 1).max(0) as usize;
        ((self.w as usize * water) / 220).max(4)
    }

    fn seaweed_target(&self) -> usize {
        (self.w as usize / 15).max(2)
    }

    // ---- spawning -------------------------------------------------------

    fn add_castle(&mut self) {
        let sprite = Sprite::new(art::CASTLE, false);
        let x = (self.w - sprite.w - 2).max(0) as f32;
        let y = (self.h - sprite.h) as f32;
        self.entities
            .push(Entity::new(Kind::Castle, vec![sprite], x, y, 22, Color::DarkGray));
    }

    fn add_seaweed(&mut self) {
        let max_height = (self.h - SKY - WAVE_ROWS - 4).clamp(3, 8);
        let height = self.rng.random_range(3..=max_height) as usize;
        let make = |shift: usize| {
            let lines: Vec<String> = (0..height)
                .map(|i| if (i + shift).is_multiple_of(2) { "( ".into() } else { " )".into() })
                .collect();
            let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
            Sprite::new(&refs, false)
        };
        // Keep clear of the castle so it stays readable.
        let (castle_x, castle_w) = self
            .entities
            .iter()
            .find(|e| e.kind == Kind::Castle)
            .map_or((self.w, 0), |c| (c.x as i32, c.sprite().w));
        let mut x = self.rng.random_range(0..(self.w - 2).max(1));
        for _ in 0..10 {
            if x + 2 <= castle_x || x >= castle_x + castle_w {
                break;
            }
            x = self.rng.random_range(0..(self.w - 2).max(1));
        }
        let y = (self.h - height as i32) as f32;
        let mut e = Entity::new(Kind::Seaweed, vec![make(0), make(1)], x as f32, y, 21, Color::Green);
        e.frame = self.rng.random_range(0..2);
        e.frame_dur = self.rng.random_range(0.5..1.2);
        self.entities.push(e);
    }

    /// Choose a design by weight, keeping "the big one" rare.
    fn pick_species(&mut self) -> usize {
        let big_cap = (self.w as usize / 90).max(1);
        let bigs = self.fish.iter().filter(|f| f.size >= BIG_SIZE).count();

        // Often join an existing school, which makes flocks and gives fish
        // someone of their own kind to court.
        if !self.fish.is_empty() && self.rng.random_bool(0.5) {
            let pick = self.fish[self.rng.random_range(0..self.fish.len())].species;
            if Fish::species_size(pick) < BIG_SIZE || bigs < big_cap {
                return pick;
            }
        }

        let total: u32 = art::FISH.iter().map(|f| f.1).sum();
        for _ in 0..8 {
            let mut r = self.rng.random_range(0..total);
            let mut pick = 0;
            for (i, (_, weight)) in art::FISH.iter().enumerate() {
                if r < *weight {
                    pick = i;
                    break;
                }
                r -= weight;
            }
            if Fish::species_size(pick) < BIG_SIZE || bigs < big_cap {
                return pick;
            }
        }
        0
    }

    fn random_color(&mut self) -> Color {
        FISH_COLORS[self.rng.random_range(0..FISH_COLORS.len())]
    }

    fn add_fish_at(&mut self, species: usize, adult: bool, x: f32, y: f32, right: bool) -> usize {
        let color = self.random_color();
        self.add_colored_fish(species, adult, x, y, right, color)
    }

    fn add_colored_fish(
        &mut self,
        species: usize,
        adult: bool,
        x: f32,
        y: f32,
        right: bool,
        color: Color,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let mut fish = Fish::new(id, species, adult, (x, y), color, right, &mut self.rng);
        let (min_x, max_x, min_y, max_y) = fish.bounds(self.w, self.h);
        fish.e.x = fish.e.x.clamp(min_x, max_x);
        fish.e.y = fish.e.y.clamp(min_y, max_y);
        self.fish.push(fish);
        self.fish.len() - 1
    }

    /// A new fish swims in from one of the tank walls.
    fn add_arrival(&mut self) {
        let species = self.pick_species();
        let right = self.rng.random_bool(0.5);
        let x = if right { 0.0 } else { self.w as f32 };
        let y = self.rng.random_range(SKY + WAVE_ROWS..self.h.max(SKY + WAVE_ROWS + 1)) as f32;
        let i = self.add_fish_at(species, true, x, y, right);
        self.fish[i].hunger = self.rng.random_range(0.2..0.5);
    }

    fn add_baby(&mut self, x: f32, y: f32, species: usize, parent_color: Color) {
        // Mostly the parent's colour, occasionally a mutation.
        let color = if self.rng.random_bool(0.3) { self.random_color() } else { parent_color };
        let right = self.rng.random_bool(0.5);
        let i = self.add_colored_fish(species, false, x, y, right, color);
        self.fish[i].hunger = 0.3;
        self.stats.born += 1;
    }

    fn add_bubble(&mut self, x: f32, y: f32) {
        let frames = art::BUBBLE_FRAMES
            .iter()
            .map(|b| Sprite::new(&[b], false))
            .collect();
        let mut e = Entity::new(Kind::Bubble, frames, x, y, 2, Color::LightCyan);
        e.vy = -self.rng.random_range(3.0..5.0);
        self.entities.push(e);
    }

    /// Sprinkle food flakes around a column. Clicking in the water drops them
    /// at that depth; otherwise they fall from the sky.
    pub fn drop_food(&mut self, col: i32, row: i32, count: usize) {
        for _ in 0..count {
            let x = (col + self.rng.random_range(-2..=2)).clamp(0, (self.w - 1).max(0));
            let y = if row >= SKY + WAVE_ROWS {
                row - self.rng.random_range(0..2)
            } else {
                SKY - 2 - self.rng.random_range(0..4)
            };
            let frames = [",", "."].iter().map(|c| Sprite::new(&[c], false)).collect();
            let mut e = Entity::new(Kind::Food, frames, x as f32, y.max(0) as f32, 6, Color::Yellow);
            e.bold = true;
            e.vy = self.rng.random_range(1.4..2.4);
            e.frame_dur = 0.5;
            e.phase = self.rng.random_range(0.0..std::f32::consts::TAU);
            self.entities.push(e);
        }
    }

    /// Feed from the keyboard: at the mouse column if we know it, else anywhere.
    pub fn feed(&mut self, col: Option<i32>) {
        let col = col.unwrap_or_else(|| self.rng.random_range(4..(self.w - 4).max(5)));
        self.drop_food(col, 0, 6);
    }

    /// Bang on the glass: every fish panics and bolts away from the spot.
    pub fn tap(&mut self, col: i32, row: i32) {
        let from = iso(col as f32, row as f32);
        for f in &mut self.fish {
            f.panic = self.rng.random_range(1.5..2.5);
            f.panic_from = from;
        }
        self.entities.push(Entity::effect(
            "(o)",
            (col - 1) as f32,
            row as f32,
            Color::White,
            0.5,
            0.0,
        ));
    }

    pub fn tap_random(&mut self) {
        let col = self.rng.random_range(0..self.w.max(1));
        let row = self.rng.random_range(SKY + WAVE_ROWS..self.h.max(SKY + WAVE_ROWS + 1));
        self.tap(col, row);
    }

    /// Where a special visitor enters: just off-screen, heading inwards.
    fn entry_dir(&mut self) -> bool {
        self.rng.random_bool(0.5)
    }

    fn add_shark(&mut self) {
        let right = self.entry_dir();
        let sprite = Sprite::new(art::SHARK, !right);
        let max_y = (self.h - sprite.h - 2).max(SKY + WAVE_ROWS + 1);
        let y = self.rng.random_range(SKY + WAVE_ROWS + 1..=max_y) as f32;
        let x = if right { -sprite.w as f32 } else { self.w as f32 };
        let mut e = Entity::new(Kind::Shark, vec![sprite], x, y, 2, Color::Gray);
        e.vx = if right { 11.0 } else { -11.0 };
        self.entities.push(e);
    }

    fn add_whale(&mut self) {
        let right = self.entry_dir();
        let frames: Vec<Sprite> = art::WHALE_SPOUT
            .iter()
            .map(|spout| {
                let lines: Vec<&str> = spout.iter().chain(art::WHALE_BODY).copied().collect();
                Sprite::new(&lines, !right)
            })
            .collect();
        let x = if right { -frames[0].w as f32 } else { self.w as f32 };
        let mut e = Entity::new(Kind::Whale, frames, x, 0.0, 10, Color::LightBlue);
        e.vx = if right { 6.0 } else { -6.0 };
        e.frame_dur = 0.3;
        self.entities.push(e);
    }

    fn add_ship(&mut self) {
        let right = self.entry_dir();
        let sprite = Sprite::new(art::SHIP, !right);
        let y = (SKY - sprite.h + 1).max(0) as f32;
        let x = if right { -sprite.w as f32 } else { self.w as f32 };
        let mut e = Entity::new(Kind::Ship, vec![sprite], x, y, 8, Color::Yellow);
        e.vx = if right { 5.0 } else { -5.0 };
        self.entities.push(e);
    }

    fn add_ducks(&mut self) {
        let right = self.entry_dir();
        let frames: Vec<Sprite> = art::DUCKS.iter().map(|f| Sprite::new(f, !right)).collect();
        let x = if right { -frames[0].w as f32 } else { self.w as f32 };
        let y = (SKY - frames[0].h) as f32;
        let mut e = Entity::new(Kind::Ducks, frames, x, y, 8, Color::LightYellow);
        e.vx = if right { 7.0 } else { -7.0 };
        e.frame_dur = 0.4;
        self.entities.push(e);
    }

    /// Bring in a random special visitor. Public so the UI can trigger one on demand.
    pub fn add_special(&mut self) {
        match self.rng.random_range(0..4) {
            0 => self.add_shark(),
            1 => self.add_whale(),
            2 => self.add_ship(),
            _ => self.add_ducks(),
        }
    }

    pub fn add_extra_fish(&mut self) {
        self.add_arrival();
    }

    // ---- simulation -----------------------------------------------------

    pub fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }
        self.time += dt;
        self.update_entities(dt);
        self.update_fish(dt);
        self.match_social(dt);
        self.entities.retain(|e| e.alive);
        self.fish.retain(|f| f.e.alive);

        // Keep the tank stocked; new fish swim in from the walls.
        self.arrival_cd -= dt;
        if self.fish.len() < self.fish_target() && self.arrival_cd <= 0.0 {
            self.arrival_cd = self.rng.random_range(2.0..6.0);
            self.add_arrival();
        }

        // One special visitor at a time, with a pause in between.
        if !self.entities.iter().any(|e| e.kind.is_special()) {
            self.special_cooldown -= dt;
            if self.special_cooldown <= 0.0 {
                self.special_cooldown = self.rng.random_range(6.0..16.0);
                self.add_special();
            }
        }
    }

    /// Scenery, bubbles, food, effects and visitors.
    fn update_entities(&mut self, dt: f32) {
        let (w, h) = (self.w, self.h);
        let fish_centers: Vec<V> = self.fish.iter().map(Fish::center).collect();

        for e in &mut self.entities {
            e.advance(dt);
            match e.kind {
                Kind::Bubble => {
                    e.frame = if e.age < 1.0 {
                        0
                    } else if e.age < 2.5 {
                        1
                    } else {
                        2
                    };
                    if e.y <= SKY as f32 {
                        e.alive = false;
                    }
                }
                Kind::Food => {
                    let floor = (h - 2) as f32;
                    if e.y >= floor {
                        // Resting on the sand until it dissolves.
                        e.y = floor;
                        e.vy = 0.0;
                        e.vx = 0.0;
                        e.timer += dt;
                        if e.timer > 14.0 {
                            e.alive = false;
                        }
                    } else {
                        e.vx = (e.age * 2.5 + e.phase).sin() * 1.2;
                    }
                    e.x = e.x.clamp(0.0, (w - 1).max(0) as f32);
                }
                Kind::Effect => {
                    if e.age >= e.timer {
                        e.alive = false;
                    }
                }
                Kind::Shark => {
                    // The shark drifts up or down towards the nearest fish ahead of it.
                    let s = e.sprite();
                    let (sw, sh) = (s.w as f32, s.h as f32);
                    let dir = if e.facing_right() { 1.0 } else { -1.0 };
                    let me = iso(e.x + sw / 2.0, e.y + sh / 2.0);
                    let target = fish_centers
                        .iter()
                        .filter(|c| (c.x - me.x) * dir > 0.0 && me.dist(**c) < 30.0)
                        .min_by(|a, b| me.dist(**a).total_cmp(&me.dist(**b)));
                    e.vy = match target {
                        Some(c) => ((c.y - me.y) * 0.5).clamp(-1.8, 1.8),
                        None => e.vy * 0.9,
                    };
                    let min_y = (SKY + WAVE_ROWS) as f32;
                    e.y = e.y.clamp(min_y, (h as f32 - sh - 1.0).max(min_y));
                    if e.left_screen(w) {
                        e.alive = false;
                    }
                }
                k if k.is_special() => {
                    if e.left_screen(w) {
                        e.alive = false;
                    }
                }
                _ => {}
            }
        }
    }

    /// Let every fish look around and act, then apply what happened.
    fn update_fish(&mut self, dt: f32) {
        let ctx = Ctx {
            snaps: self.fish.iter().map(Fish::snap).collect(),
            foods: self
                .entities
                .iter()
                .enumerate()
                .filter(|(_, e)| e.kind == Kind::Food && e.alive)
                .map(|(idx, e)| {
                    let cell = (e.x.round() as i32, e.y.round() as i32);
                    FoodSnap {
                        idx,
                        cell,
                        c: iso(cell.0 as f32 + 0.5, cell.1 as f32 + 0.5),
                    }
                })
                .collect(),
            sharks: self
                .entities
                .iter()
                .filter(|e| e.kind == Kind::Shark)
                .map(|e| {
                    let s = e.sprite();
                    iso(e.x + s.w as f32 / 2.0, e.y + s.h as f32 / 2.0)
                })
                .collect(),
            w: self.w,
            h: self.h,
        };

        let mut out = Out::default();
        for f in &mut self.fish {
            f.update(dt, &ctx, &mut self.rng, &mut out);
        }

        for &idx in &out.food {
            self.entities[idx].alive = false;
            self.stats.fed += 1;
        }
        for &id in &out.eaten {
            if let Some(f) = self.fish.iter_mut().find(|f| f.id == id) {
                f.e.alive = false;
                self.stats.eaten += 1;
            }
        }
        for &id in &out.swaps {
            // The chaser caught its partner, so the partner is "it" now.
            if let Some(p) = self
                .fish
                .iter_mut()
                .find(|f| f.id == id)
                .and_then(|f| f.partner.as_mut())
            {
                p.chaser = true;
                p.swap_cd = 1.0;
            }
        }
        for fx in out.fx {
            self.entities
                .push(Entity::effect(fx.text, fx.x, fx.y, fx.color, fx.ttl, fx.vy));
        }
        for (x, y) in out.bubbles {
            self.add_bubble(x, y);
        }
        for b in out.births {
            self.add_baby(b.x, b.y, b.species, b.color);
        }

        self.sharks_eat_fish();
    }

    /// Any fish that swims into a shark's mouth is eaten.
    fn sharks_eat_fish(&mut self) {
        let mouths: Vec<(i32, i32, i32, i32)> = self
            .entities
            .iter()
            .filter(|e| e.kind == Kind::Shark)
            .map(|e| {
                let (x0, y0, x1, y1) = e.rect();
                if e.facing_right() { (x1 - 6, y0, x1, y1) } else { (x0, y0, x0 + 6, y1) }
            })
            .collect();

        let mut splashes = Vec::new();
        for fish in self.fish.iter_mut().filter(|f| f.e.alive) {
            let r = fish.e.rect();
            let hit = mouths
                .iter()
                .any(|m| r.0 < m.2 && m.0 < r.2 && r.1 < m.3 && m.1 < r.3);
            if hit {
                fish.e.alive = false;
                self.stats.eaten += 1;
                splashes.push(((r.0 + r.2) as f32 / 2.0, r.1 as f32));
            }
        }
        for (x, y) in splashes {
            self.entities
                .push(Entity::effect("*", x, y, Color::LightRed, 0.7, -1.0));
            self.add_bubble(x, y);
        }
    }

    /// Fed, unoccupied fish occasionally pair up: similar sizes play tag,
    /// same-species adults court and have a baby.
    fn match_social(&mut self, dt: f32) {
        fn free(f: &Fish) -> bool {
            f.adult && f.partner.is_none() && f.social_cd <= 0.0 && f.mood == Mood::Cruise && f.panic <= 0.0
        }

        let n = self.fish.len();
        let can_breed = n < self.fish_target() * 3 / 2;
        let chance = (dt * 0.2).min(1.0) as f64;

        for i in 0..n {
            if !free(&self.fish[i]) || !self.rng.random_bool(chance) {
                continue;
            }
            let me = self.fish[i].center();
            let best = (0..n)
                .filter(|&j| j != i && free(&self.fish[j]))
                .map(|j| (j, me.dist(self.fish[j].center())))
                .filter(|&(_, d)| d < 22.0)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            let Some((j, _)) = best else { continue };

            let (a, b) = (&self.fish[i], &self.fish[j]);
            let ratio = a.size.max(b.size) / a.size.min(b.size);
            let court = can_breed
                && a.species == b.species
                && a.hunger < 0.5
                && b.hunger < 0.5
                && a.breed_cd <= 0.0
                && b.breed_cd <= 0.0;
            if !court && (ratio > 1.6 || a.hunger > 0.6 || b.hunger > 0.6) {
                continue;
            }

            let (ida, idb) = (a.id, b.id);
            let timer = if court { 7.0 } else { self.rng.random_range(7.0..12.0) };
            let make = |id, chaser| Partner {
                id,
                court,
                timer,
                chaser,
                swap_cd: 0.0,
                heart_t: 0.0,
            };
            self.fish[i].partner = Some(make(idb, true));
            self.fish[j].partner = Some(make(ida, false));
        }
    }
}

// ---- rendering ----------------------------------------------------------

impl Widget for &World {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        if area.width < MIN_W || area.height < MIN_H {
            let msg = format!("Terminal too small: need at least {MIN_W}x{MIN_H}");
            buf.set_stringn(area.x, area.y, msg, area.width as usize, Style::default().fg(Color::Red));
            return;
        }

        let mut order: Vec<&Entity> = self
            .entities
            .iter()
            .chain(self.fish.iter().map(|f| &f.e))
            .collect();
        order.sort_by_key(|e| Reverse(e.depth)); // stable: far things first
        for e in order {
            draw_sprite(buf, area, e, e.x.round() as i32, e.y.round() as i32);
        }

        self.draw_waves(buf, area);
        self.draw_hud(buf, area);
    }
}

impl World {
    fn draw_waves(&self, buf: &mut Buffer, area: Rect) {
        const SPEEDS: [f32; 4] = [2.0, -1.5, 1.0, -2.5];
        let style = Style::default().fg(Color::Cyan);
        for (row, pattern) in art::WAVES.iter().enumerate() {
            let pat = pattern.as_bytes();
            let offset = (self.time * SPEEDS[row]) as i32;
            let y = area.y + SKY as u16 + row as u16;
            for x in 0..area.width {
                let ch = pat[(x as i32 + offset).rem_euclid(pat.len() as i32) as usize] as char;
                if ch != ' ' {
                    buf[(area.x + x, y)].set_char(ch).set_style(style);
                }
            }
        }
    }

    fn draw_hud(&self, buf: &mut Buffer, area: Rect) {
        let dim = Style::default().fg(Color::Gray);
        let stats = format!(
            " fish {}  fed {}  eaten {}  born {}{}",
            self.fish.len(),
            self.stats.fed,
            self.stats.eaten,
            self.stats.born,
            if self.paused { "  [PAUSED]" } else { "" },
        );
        buf.set_stringn(area.x, area.y, stats, area.width as usize, dim);

        if self.show_help || self.time < 10.0 {
            let help = " space/f feed · click drop food · right-click/t tap glass · s visitor · a add fish · p pause · h help · q quit";
            buf.set_stringn(area.x, area.y + 1, help, area.width as usize, dim);
        }
    }
}

fn draw_sprite(buf: &mut Buffer, area: Rect, e: &Entity, x: i32, y: i32) {
    let mut body = Style::default().fg(e.color);
    if e.bold {
        body = body.add_modifier(Modifier::BOLD);
    }
    let eye = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);
    for (r, row) in e.sprite().rows.iter().enumerate() {
        let py = y + r as i32;
        if py < 0 || py >= area.height as i32 {
            continue;
        }
        for (c, &ch) in row.iter().enumerate() {
            let px = x + c as i32;
            if ch == TRANSPARENT || px < 0 || px >= area.width as i32 {
                continue;
            }
            let (ch, style) = if ch == '@' { ('o', eye) } else { (ch, body) };
            buf[(area.x + px as u16, area.y + py as u16)]
                .set_char(ch)
                .set_style(style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dump(world: &World, w: u16, h: u16) {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        (&*world).render(area, &mut buf);
        for y in 0..h {
            let line: String = (0..w).map(|x| buf[(x, y)].symbol().to_string()).collect();
            println!("{}", line.trim_end());
        }
        println!("{}", "-".repeat(w as usize));
    }

    /// A world with no fish, no random visitors, and no restocking.
    fn empty_world() -> World {
        let mut world = World::new(110, 32);
        world.fish.clear();
        world.entities.retain(|e| e.kind == Kind::Castle);
        world.special_cooldown = 1e9;
        world.arrival_cd = 1e9;
        world
    }

    fn assert_in_tank(world: &World) {
        for f in &world.fish {
            let (min_x, max_x, min_y, max_y) = f.bounds(world.w, world.h);
            assert!(f.e.x >= min_x && f.e.x <= max_x, "fish escaped horizontally: {}", f.e.x);
            assert!(f.e.y >= min_y && f.e.y <= max_y, "fish escaped vertically: {}", f.e.y);
            assert!(f.e.x.is_finite() && f.e.y.is_finite());
        }
    }

    #[test]
    fn fish_eat_food_in_reach() {
        let mut world = empty_world();
        let i = world.add_fish_at(0, true, 40.0, 20.0, true);
        world.fish[i].hunger = 0.8;
        // Right in front of the mouth of a right-facing fish.
        world.drop_food(0, 0, 0);
        let mut e = Entity::new(Kind::Food, vec![Sprite::new(&[","], false)], 45.0, 20.0, 6, Color::Yellow);
        e.vy = 0.0;
        world.entities.push(e);
        world.update(0.02);
        assert_eq!(world.stats.fed, 1);
        assert!(world.fish[0].hunger < 0.6);
    }

    #[test]
    fn full_fish_ignore_food() {
        let mut world = empty_world();
        let i = world.add_fish_at(0, true, 40.0, 20.0, true);
        world.fish[i].hunger = 0.0;
        let mut e = Entity::new(Kind::Food, vec![Sprite::new(&[","], false)], 45.0, 20.0, 6, Color::Yellow);
        e.vy = 0.0;
        world.entities.push(e);
        world.update(0.02);
        assert_eq!(world.stats.fed, 0);
    }

    #[test]
    fn hungry_big_fish_eats_small_fish() {
        let mut world = empty_world();
        let big = world.add_fish_at(6, true, 40.0, 20.0, true);
        world.fish[big].hunger = 1.0;
        // Overlapping the big fish's mouth.
        let small = world.add_fish_at(0, true, 55.0, 21.0, true);
        world.fish[small].hunger = 0.0;
        world.update(0.02);
        assert_eq!(world.stats.eaten, 1);
        assert_eq!(world.fish.len(), 1);
        assert!(world.fish[0].size >= BIG_SIZE);
        assert_eq!(world.fish[0].hunger, 0.0);
    }

    #[test]
    fn well_fed_big_fish_leaves_small_fish_alone() {
        let mut world = empty_world();
        let big = world.add_fish_at(6, true, 40.0, 20.0, true);
        world.fish[big].hunger = 0.1;
        world.add_fish_at(0, true, 55.0, 21.0, true);
        for _ in 0..20 {
            world.update(0.02);
        }
        assert_eq!(world.stats.eaten, 0);
        assert_eq!(world.fish.len(), 2);
    }

    #[test]
    fn courtship_ends_in_a_baby() {
        let mut world = empty_world();
        let a = world.add_fish_at(2, true, 40.0, 20.0, true);
        let b = world.add_fish_at(2, true, 46.0, 20.0, false);
        let (ida, idb) = (world.fish[a].id, world.fish[b].id);
        let make = |id| Partner { id, court: true, timer: 0.01, chaser: false, swap_cd: 0.0, heart_t: 0.0 };
        world.fish[a].partner = Some(make(idb));
        world.fish[b].partner = Some(make(ida));
        world.update(0.05);
        assert_eq!(world.stats.born, 1);
        assert_eq!(world.fish.len(), 3);
        assert!(world.fish.iter().any(|f| !f.adult));
    }

    #[test]
    fn babies_grow_up() {
        let mut world = empty_world();
        let i = world.add_fish_at(3, false, 40.0, 20.0, true);
        let baby_size = world.fish[i].size;
        world.fish[i].e.age = 44.99;
        world.update(0.05);
        assert!(world.fish[0].adult);
        assert!(world.fish[0].size > baby_size);
    }

    #[test]
    fn tapping_the_glass_scatters_fish() {
        let mut world = empty_world();
        let i = world.add_fish_at(0, true, 50.0, 20.0, true);
        world.tap(40, 20);
        for _ in 0..10 {
            world.update(0.05);
        }
        assert_eq!(world.fish[i].mood, Mood::Flee);
        // Bolted away from the tap point, to the right.
        assert!(world.fish[i].vel.x > 0.0);
    }

    #[test]
    fn tiny_or_resized_terminals_do_not_panic() {
        for w in [0u16, 1, 5, 20, 39, 40, 60] {
            for h in [0u16, 1, 5, 12, 19, 20, 24] {
                let mut world = World::new(w, h);
                for step in 0..200 {
                    world.update(0.05);
                    if step == 20 {
                        world.feed(None);
                        world.tap_random();
                        world.add_special();
                        world.add_extra_fish();
                    }
                }
                world.resize(w.max(3) / 2, h.max(3) / 2);
                world.update(0.05);
                let area = Rect::new(0, 0, w, h);
                let mut buf = Buffer::empty(area);
                (&world).render(area, &mut buf);
            }
        }
    }

    #[test]
    fn soak_test_stays_sane() {
        let mut world = World::new(110, 32);
        for step in 0..6000 {
            world.update(0.05);
            match step % 400 {
                50 => world.feed(Some(50)),
                200 => world.tap_random(),
                300 => world.add_special(),
                _ => {}
            }
            assert_in_tank(&world);
        }
        assert!(!world.fish.is_empty());
        println!(
            "after 5 minutes: fish={} fed={} eaten={} born={}",
            world.fish.len(),
            world.stats.fed,
            world.stats.eaten,
            world.stats.born
        );
    }

    #[test]
    fn dump_feeding_frenzy() {
        let mut world = World::new(110, 32);
        world.special_cooldown = 1e9;
        for _ in 0..40 {
            world.update(0.05);
        }
        world.feed(Some(40));
        for _ in 0..50 {
            world.update(0.05);
        }
        dump(&world, 110, 32);
    }
}

#[cfg(test)]
mod balance {
    use super::*;

    fn run(label: &str, sharks: bool, feed_every: usize) {
        let mut tot = (0.0, 0u32, 0u32, 0u32, 0u32);
        let trials = 20;
        for _ in 0..trials {
            let mut world = World::new(110, 32);
            if !sharks {
                world.special_cooldown = 1e9;
            }
            for step in 0..6000 {
                world.update(0.05);
                if feed_every > 0 && step % feed_every == 50 {
                    world.feed(Some(50));
                }
            }
            tot.0 += world.fish.len() as f32;
            tot.1 += world.stats.fed;
            tot.2 += world.stats.eaten;
            tot.3 += world.stats.born;
        }
        let t = trials as f32;
        println!(
            "{label:<28} fish={:.1} fed={:.0} eaten={:.1} born={:.1}",
            tot.0 / t,
            tot.1 as f32 / t,
            tot.2 as f32 / t,
            tot.3 as f32 / t
        );
    }

    #[test]
    #[ignore = "prints balance statistics, run with --ignored --nocapture"]
    fn report() {
        run("no shark, never fed", false, 0);
        run("no shark, fed every 20s", false, 400);
        run("shark visits, fed every 20s", true, 400);
    }
}
