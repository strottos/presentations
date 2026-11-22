//! Fish: steering, hunger, fear, play and courtship.
//!
//! Steering happens in "isotropic" space where one unit is one terminal row
//! and half as many columns, since cells are roughly twice as tall as wide.
//! That keeps circles round and distances honest.

use std::ops::{Add, Mul, Sub};

use rand::prelude::*;
use ratatui::style::Color;

use crate::art::{self, Sprite};
use crate::entity::{Entity, Kind};
use crate::world::{SKY, WAVE_ROWS};

/// Seconds for a fish to go from full to starving.
const HUNGER_SECS: f32 = 90.0;
/// Seconds for a newborn to grow into its adult design.
const GROW_SECS: f32 = 45.0;
const VISION: f32 = 22.0;
const HUNT_HUNGER: f32 = 0.7;
const HUNT_RANGE: f32 = 24.0;
/// A fish will only eat things this many times smaller than itself.
const PREY_RATIO: f32 = 1.8;
/// Adults at least this big count as "the big one".
pub const BIG_SIZE: f32 = 45.0;
const FLEE_SPEED: f32 = 5.0;
const HUNT_SPEED: f32 = 5.4;
const FOOD_SPEED: f32 = 4.2;
const COURT_RADIUS: f32 = 3.0;

// ---- tiny vector type --------------------------------------------------

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct V {
    pub x: f32,
    pub y: f32,
}

impl V {
    pub const ZERO: V = V { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> V {
        V { x, y }
    }
    pub fn len(self) -> f32 {
        self.x.hypot(self.y)
    }
    pub fn norm(self) -> V {
        let l = self.len();
        if l < 1e-4 { V::ZERO } else { self * (1.0 / l) }
    }
    pub fn dist(self, o: V) -> f32 {
        (self - o).len()
    }
    pub fn clamp_len(self, max: f32) -> V {
        let l = self.len();
        if l > max && l > 0.0 { self * (max / l) } else { self }
    }
}

impl Add for V {
    type Output = V;
    fn add(self, o: V) -> V {
        V::new(self.x + o.x, self.y + o.y)
    }
}
impl Sub for V {
    type Output = V;
    fn sub(self, o: V) -> V {
        V::new(self.x - o.x, self.y - o.y)
    }
}
impl Mul<f32> for V {
    type Output = V;
    fn mul(self, k: f32) -> V {
        V::new(self.x * k, self.y * k)
    }
}

/// Cell coordinates to isotropic space.
pub fn iso(x: f32, y: f32) -> V {
    V::new(x * 0.5, y)
}

// ---- state -------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mood {
    Cruise,
    Food,
    Hunt,
    Flee,
    Play,
    Court,
}

/// Two fish doing something together: playing tag or courting.
#[derive(Clone, Copy)]
pub struct Partner {
    pub id: u32,
    pub court: bool,
    pub timer: f32,
    /// Play only: who is "it" right now.
    pub chaser: bool,
    pub swap_cd: f32,
    pub heart_t: f32,
}

pub struct Fish {
    pub id: u32,
    pub e: Entity,
    pub species: usize,
    pub adult: bool,
    pub size: f32,
    pub vel: V,
    pub face_right: bool,
    pub hunger: f32,
    pub cruise: f32,
    pub mood: Mood,
    pub wander: V,
    wander_t: f32,
    pub partner: Option<Partner>,
    pub social_cd: f32,
    pub breed_cd: f32,
    pub panic: f32,
    pub panic_from: V,
    /// Slowed down after a meal.
    stun: f32,
    /// How long the current chase has lasted; predators give up eventually.
    chase_time: f32,
    tired: f32,
    bubble_t: f32,
    fx_cd: f32,
}

/// What a fish knows about another fish, captured at the start of the frame.
pub struct Snap {
    pub id: u32,
    pub c: V,
    pub vel: V,
    pub size: f32,
    pub species: usize,
    pub adult: bool,
    pub hunger: f32,
    pub rect: (i32, i32, i32, i32),
    pub partner: Option<u32>,
}

pub struct FoodSnap {
    /// Index into the world's entity list.
    pub idx: usize,
    pub cell: (i32, i32),
    pub c: V,
}

/// Everything a fish can perceive this frame.
pub struct Ctx {
    pub snaps: Vec<Snap>,
    pub foods: Vec<FoodSnap>,
    pub sharks: Vec<V>,
    pub w: i32,
    pub h: i32,
}

pub struct Fx {
    pub text: &'static str,
    pub x: f32,
    pub y: f32,
    pub color: Color,
    pub ttl: f32,
    pub vy: f32,
}

pub struct Birth {
    pub x: f32,
    pub y: f32,
    pub species: usize,
    pub color: Color,
}

/// Side effects of a frame of fish thinking, applied by the world afterwards.
#[derive(Default)]
pub struct Out {
    pub fx: Vec<Fx>,
    /// Cell coordinates.
    pub bubbles: Vec<(f32, f32)>,
    pub eaten: Vec<u32>,
    pub food: Vec<usize>,
    pub births: Vec<Birth>,
    pub swaps: Vec<u32>,
}

impl Out {
    fn fx(&mut self, text: &'static str, x: f32, y: f32, color: Color, ttl: f32, vy: f32) {
        self.fx.push(Fx { text, x, y, color, ttl, vy });
    }
}

fn overlaps(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3
}

impl Fish {
    pub fn new(
        id: u32,
        species: usize,
        adult: bool,
        pos: (f32, f32),
        color: Color,
        face_right: bool,
        rng: &mut ThreadRng,
    ) -> Self {
        let frames = Self::frames_for(species, adult);
        let size = frames[0].area() as f32;
        let mut e = Entity::new(Kind::Fish, frames, pos.0, pos.1, rng.random_range(3..=20), color);
        e.frame = if face_right { 0 } else { 1 };
        let mut cruise = 1.6 + 2.0 * rng.random_range(0.0..1.0);
        if size >= BIG_SIZE {
            cruise *= 0.7;
        }
        let wander = V::new(if face_right { 1.0 } else { -1.0 }, 0.0);
        Fish {
            id,
            e,
            species,
            adult,
            size,
            vel: wander * (cruise * 0.5),
            face_right,
            hunger: 0.0,
            cruise,
            mood: Mood::Cruise,
            wander,
            wander_t: rng.random_range(1.0..3.0),
            partner: None,
            social_cd: rng.random_range(3.0..10.0),
            breed_cd: 0.0,
            panic: 0.0,
            panic_from: V::ZERO,
            stun: 0.0,
            chase_time: 0.0,
            tired: 0.0,
            bubble_t: rng.random_range(2.0..10.0),
            fx_cd: 0.0,
        }
    }

    /// Right-facing sprite first, left-facing second.
    fn frames_for(species: usize, adult: bool) -> Vec<Sprite> {
        let art = if adult { art::FISH[species].0 } else { art::BABY };
        vec![Sprite::new(art, false), Sprite::new(art, true)]
    }

    pub fn species_size(species: usize) -> f32 {
        Sprite::new(art::FISH[species].0, false).area() as f32
    }

    pub fn center(&self) -> V {
        let s = self.e.sprite();
        iso(self.e.x + s.w as f32 / 2.0, self.e.y + s.h as f32 / 2.0)
    }

    /// Swimmable area as (min_x, max_x, min_y, max_y) for the sprite's top-left.
    pub fn bounds(&self, w: i32, h: i32) -> (f32, f32, f32, f32) {
        let s = self.e.sprite();
        let min_y = (SKY + WAVE_ROWS) as f32;
        (
            0.0,
            (w - s.w).max(0) as f32,
            min_y,
            ((h - s.h - 1) as f32).max(min_y),
        )
    }

    pub fn snap(&self) -> Snap {
        Snap {
            id: self.id,
            c: self.center(),
            vel: self.vel,
            size: self.size,
            species: self.species,
            adult: self.adult,
            hunger: self.hunger,
            rect: self.e.rect(),
            partner: self.partner.map(|p| p.id),
        }
    }

    fn mouth_rect(&self) -> (i32, i32, i32, i32) {
        let (x0, y0, x1, y1) = self.e.rect();
        if self.face_right {
            (x1 - 3, y0, x1 + 1, y1)
        } else {
            (x0 - 1, y0, x0 + 3, y1)
        }
    }

    fn mouth_cell(&self) -> (f32, f32) {
        let s = self.e.sprite();
        let x = if self.face_right { self.e.x + s.w as f32 } else { self.e.x - 1.0 };
        (x, self.e.y + (s.h / 2) as f32)
    }

    fn grow_up(&mut self, out: &mut Out) {
        self.adult = true;
        self.e.frames = Self::frames_for(self.species, true);
        self.size = self.e.frames[0].area() as f32;
        let (x, y) = (self.e.x, self.e.y);
        out.fx("*", x, y, Color::LightYellow, 0.8, -1.0);
    }

    fn wall_push(&self, w: i32, h: i32) -> V {
        let (min_x, max_x, min_y, max_y) = self.bounds(w, h);
        let (mx, my) = (8.0, 2.5);
        let mut p = V::ZERO;
        let (dl, dr) = (self.e.x - min_x, max_x - self.e.x);
        let (dt, db) = (self.e.y - min_y, max_y - self.e.y);
        if dl < mx {
            p.x += 1.0 - dl / mx;
        }
        if dr < mx {
            p.x -= 1.0 - dr / mx;
        }
        if dt < my {
            p.y += 1.0 - dt / my;
        }
        if db < my {
            p.y -= 1.0 - db / my;
        }
        p
    }

    fn nearest_threat(&self, me: V, ctx: &Ctx) -> Option<V> {
        if self.panic > 0.0 {
            return Some(self.panic_from);
        }
        let mut best: Option<(f32, V)> = None;
        let mut consider = |d: f32, at: V| {
            if best.is_none_or(|b| d < b.0) {
                best = Some((d, at));
            }
        };
        for &s in &ctx.sharks {
            let d = me.dist(s);
            if d < 16.0 {
                consider(d, s);
            }
        }
        for p in &ctx.snaps {
            if p.id != self.id && p.size >= self.size * PREY_RATIO && p.hunger > HUNT_HUNGER - 0.1 {
                let d = me.dist(p.c);
                if d < 10.0 {
                    consider(d, p.c);
                }
            }
        }
        best.map(|b| b.1)
    }

    pub fn update(&mut self, dt: f32, ctx: &Ctx, rng: &mut ThreadRng, out: &mut Out) {
        self.e.age += dt;
        self.hunger = (self.hunger + dt / HUNGER_SECS).min(1.0);
        for t in [
            &mut self.social_cd,
            &mut self.breed_cd,
            &mut self.panic,
            &mut self.stun,
            &mut self.tired,
            &mut self.bubble_t,
            &mut self.fx_cd,
        ] {
            *t -= dt;
        }
        if !self.adult && self.e.age >= GROW_SECS {
            self.grow_up(out);
        }

        let me = self.center();
        let mut desired = V::ZERO;
        let mut speed = self.cruise;
        let mut accel = 5.0;
        let mut arrive = 1.0;
        let mut mood = Mood::Cruise;
        let mut busy = false;
        let mut wall_weight = 2.5;

        // 1. Fear beats everything.
        if let Some(from) = self.nearest_threat(me, ctx) {
            desired = (me - from).norm();
            speed = FLEE_SPEED;
            accel = 14.0;
            mood = Mood::Flee;
            busy = true;
            wall_weight = 1.2;
            if self.mood != Mood::Flee && self.fx_cd <= 0.0 {
                self.fx_cd = 3.0;
                out.fx("!", self.e.x, self.e.y - 1.0, Color::LightRed, 0.8, -1.5);
            }
        }

        // 2. Hungry predators hunt anything small enough.
        if !busy && self.hunger > HUNT_HUNGER && self.tired <= 0.0 {
            let partner_id = self.partner.map(|p| p.id);
            let prey = ctx
                .snaps
                .iter()
                .filter(|p| {
                    p.id != self.id
                        && Some(p.id) != partner_id
                        && p.size * PREY_RATIO <= self.size
                        && me.dist(p.c) < HUNT_RANGE
                })
                .min_by(|a, b| me.dist(a.c).total_cmp(&me.dist(b.c)));
            if let Some(p) = prey {
                desired = (p.c + p.vel * 0.4 - me).norm();
                speed = HUNT_SPEED;
                accel = 12.0;
                mood = Mood::Hunt;
                busy = true;
                self.chase_time += dt;
                if self.chase_time > 7.0 {
                    self.tired = 8.0;
                    self.chase_time = 0.0;
                }
                if overlaps(self.mouth_rect(), p.rect) && !out.eaten.contains(&p.id) {
                    out.eaten.push(p.id);
                    self.hunger = 0.0;
                    self.stun = 0.9;
                    self.chase_time = 0.0;
                    out.fx("*", p.c.x * 2.0, p.c.y, Color::LightYellow, 0.6, -1.0);
                    let (bx, by) = self.mouth_cell();
                    out.bubbles.push((bx, by));
                    out.bubbles.push((bx + 1.0, by - 1.0));
                }
            } else {
                self.chase_time = (self.chase_time - dt).max(0.0);
            }
        }

        // 3. Hungry fish go after food.
        if !busy && self.hunger > 0.1 {
            let food = ctx
                .foods
                .iter()
                .filter(|f| me.dist(f.c) < VISION)
                .min_by(|a, b| me.dist(a.c).total_cmp(&me.dist(b.c)));
            if let Some(f) = food {
                let d = me.dist(f.c);
                desired = (f.c - me).norm();
                speed = FOOD_SPEED;
                arrive = (d / 2.5).clamp(0.35, 1.0);
                mood = Mood::Food;
                busy = true;
                let (x0, y0, x1, y1) = self.mouth_rect();
                let inside = f.cell.0 >= x0 && f.cell.0 < x1 && f.cell.1 >= y0 && f.cell.1 < y1;
                if inside && !out.food.contains(&f.idx) {
                    out.food.push(f.idx);
                    self.hunger = (self.hunger - 0.4).max(0.0);
                }
            }
        }

        // 4. Playing tag or courting with a partner.
        if !busy {
            if let Some(mut p) = self.partner {
                match ctx.snaps.iter().find(|s| s.id == p.id) {
                    Some(o) if o.partner == Some(self.id) => {
                        p.timer -= dt;
                        p.swap_cd -= dt;
                        p.heart_t -= dt;
                        let d = me.dist(o.c);
                        let mid = (self.e.x + o.rect.0 as f32) / 2.0;
                        let mid_y = (self.e.y + o.rect.1 as f32) / 2.0;
                        if p.timer <= 0.0 {
                            self.partner = None;
                            self.social_cd = rng.random_range(10.0..25.0);
                            if p.court {
                                self.breed_cd = 45.0;
                                if self.id < p.id {
                                    out.births.push(Birth {
                                        x: mid,
                                        y: mid_y,
                                        species: self.species,
                                        color: self.e.color,
                                    });
                                    out.fx("♥", mid, mid_y - 1.0, Color::LightRed, 1.4, -2.0);
                                }
                            }
                        } else {
                            busy = true;
                            if p.court {
                                mood = Mood::Court;
                                let r = me - o.c;
                                let tangent = V::new(-r.y, r.x).norm();
                                desired = tangent + r.norm() * ((COURT_RADIUS - d.max(0.01)) * 0.5);
                                speed = 2.4;
                                accel = 8.0;
                                if self.id < p.id && p.heart_t <= 0.0 {
                                    p.heart_t = 0.6;
                                    out.fx("♥", mid, mid_y - 1.0, Color::LightRed, 1.6, -2.0);
                                }
                            } else if p.chaser {
                                mood = Mood::Play;
                                desired = (o.c + o.vel * 0.2 - me).norm();
                                speed = 4.2;
                                accel = 10.0;
                                if d < 2.0 && p.swap_cd <= 0.0 {
                                    p.chaser = false;
                                    p.swap_cd = 1.0;
                                    out.swaps.push(p.id);
                                    out.fx("*", mid, mid_y - 1.0, Color::LightYellow, 0.6, -1.0);
                                }
                            } else {
                                mood = Mood::Play;
                                let away = (me - o.c).norm();
                                let side = V::new(-away.y, away.x) * (self.e.age * 5.0).sin();
                                desired = away + side * 0.8;
                                speed = 4.0;
                                accel = 10.0;
                            }
                            self.partner = Some(p);
                        }
                    }
                    _ => self.partner = None,
                }
            }
        }

        // 5. Otherwise wander, and stick with the school.
        if !busy {
            self.wander_t -= dt;
            if self.wander_t <= 0.0 {
                self.wander_t = rng.random_range(1.5..4.5);
                let keep = rng.random_bool(0.75);
                let dir = if self.face_right == keep { 1.0 } else { -1.0 };
                self.wander = V::new(dir, rng.random_range(-0.35..0.35));
            }
            let (mut coh, mut align, mut sep) = (V::ZERO, V::ZERO, V::ZERO);
            let mut n = 0.0;
            for p in &ctx.snaps {
                if p.id == self.id {
                    continue;
                }
                let d = me.dist(p.c);
                if p.species == self.species && p.adult == self.adult && d < 14.0 {
                    coh = coh + p.c;
                    align = align + p.vel;
                    n += 1.0;
                }
                if d < 3.0 {
                    sep = sep + (me - p.c).norm() * ((3.0 - d) / 3.0);
                }
            }
            desired = self.wander;
            if n > 0.0 {
                desired = desired + (coh * (1.0 / n) - me).norm() * 0.5 + align.norm() * 0.6;
            }
            desired = desired + sep * 1.5;
        }

        desired = desired + self.wall_push(ctx.w, ctx.h) * wall_weight;

        // Steer and move.
        let mut target = desired.norm() * speed * arrive;
        if self.stun > 0.0 {
            target = target * 0.15;
        }
        self.vel = self.vel + (target - self.vel).clamp_len(accel * dt);
        self.e.x += self.vel.x * 2.0 * dt;
        self.e.y += self.vel.y * dt;

        let (min_x, max_x, min_y, max_y) = self.bounds(ctx.w, ctx.h);
        if self.e.x < min_x {
            self.e.x = min_x;
            self.vel.x = self.vel.x.max(0.0);
        } else if self.e.x > max_x {
            self.e.x = max_x;
            self.vel.x = self.vel.x.min(0.0);
        }
        if self.e.y < min_y {
            self.e.y = min_y;
            self.vel.y = self.vel.y.max(0.0);
        } else if self.e.y > max_y {
            self.e.y = max_y;
            self.vel.y = self.vel.y.min(0.0);
        }

        if self.vel.x > 0.3 {
            self.face_right = true;
        } else if self.vel.x < -0.3 {
            self.face_right = false;
        }
        self.e.frame = if self.face_right { 0 } else { 1 };
        self.mood = mood;

        if self.bubble_t <= 0.0 {
            self.bubble_t = rng.random_range(4.0..12.0);
            out.bubbles.push(self.mouth_cell());
        }
    }
}
