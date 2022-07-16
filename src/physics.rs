use std::default;

use glam::Vec2;

#[derive(Default,Clone)]
pub struct Collider{
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32
}

pub struct PhysicsState{
    pub dt: f32,
    pub dt_timer: std::time::Instant,
    pub colliders: Vec<Collider>,
}

impl Default for PhysicsState{
    fn default() -> Self {
        Self { dt: 0.0, dt_timer: std::time::Instant::now(), colliders: vec![] }
    }
}