//! A drawable thing in the tank: scenery, bubbles, food, visitors, effects.

use ratatui::style::Color;

use crate::art::Sprite;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Castle,
    Seaweed,
    Fish,
    Bubble,
    Food,
    /// A short-lived glyph: hearts, sparkles, alarm marks, ripples.
    Effect,
    Shark,
    Whale,
    Ship,
    Ducks,
}

impl Kind {
    pub fn is_special(self) -> bool {
        matches!(self, Kind::Shark | Kind::Whale | Kind::Ship | Kind::Ducks)
    }
}

pub struct Entity {
    pub kind: Kind,
    pub x: f32,
    pub y: f32,
    /// Cells per second.
    pub vx: f32,
    pub vy: f32,
    pub frames: Vec<Sprite>,
    pub frame: usize,
    pub frame_dur: f32,
    pub frame_t: f32,
    pub color: Color,
    pub bold: bool,
    /// Higher is further away and drawn first.
    pub depth: u8,
    pub age: f32,
    /// General purpose countdown/lifetime, meaning depends on `kind`.
    pub timer: f32,
    pub phase: f32,
    pub alive: bool,
}

impl Entity {
    pub fn new(kind: Kind, frames: Vec<Sprite>, x: f32, y: f32, depth: u8, color: Color) -> Self {
        Entity {
            kind,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            frames,
            frame: 0,
            frame_dur: 0.0,
            frame_t: 0.0,
            color,
            bold: false,
            depth,
            age: 0.0,
            timer: 0.0,
            phase: 0.0,
            alive: true,
        }
    }

    /// A glyph that drifts vertically for `ttl` seconds and vanishes.
    pub fn effect(text: &str, x: f32, y: f32, color: Color, ttl: f32, vy: f32) -> Self {
        let mut e = Entity::new(Kind::Effect, vec![Sprite::new(&[text], false)], x, y, 1, color);
        e.bold = true;
        e.timer = ttl;
        e.vy = vy;
        e
    }

    pub fn sprite(&self) -> &Sprite {
        &self.frames[self.frame]
    }

    pub fn facing_right(&self) -> bool {
        self.vx >= 0.0
    }

    pub fn advance(&mut self, dt: f32) {
        self.age += dt;
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        if self.frames.len() > 1 && self.frame_dur > 0.0 {
            self.frame_t += dt;
            while self.frame_t >= self.frame_dur {
                self.frame_t -= self.frame_dur;
                self.frame = (self.frame + 1) % self.frames.len();
            }
        }
    }

    /// Has this entity swum/sailed off the far side of the screen?
    pub fn left_screen(&self, w: i32) -> bool {
        let s = self.sprite();
        (self.vx > 0.0 && self.x >= w as f32) || (self.vx < 0.0 && self.x + s.w as f32 <= 0.0)
    }

    /// Bounding box in cells as (x0, y0, x1, y1), end-exclusive.
    pub fn rect(&self) -> (i32, i32, i32, i32) {
        let s = self.sprite();
        let (x, y) = (self.x.round() as i32, self.y.round() as i32);
        (x, y, x + s.w, y + s.h)
    }
}
