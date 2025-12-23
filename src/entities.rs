use macroquad::prelude::*;
use std::collections::VecDeque;

// --- Components ---

pub struct Position(pub Vec2);
pub struct Velocity(pub Vec2);

pub struct ColorComp(pub Color);
#[allow(dead_code)]
pub struct CircleComp(pub f32);
pub struct RectComp(pub Vec2);

pub struct TrailComp {
    pub positions: VecDeque<Vec2>,
    pub max_size: usize,
}

pub struct ParticleLifetime(pub f32);
pub struct ParticleSize(pub f32);
pub struct ParticleRotation(pub f32, pub f32); // (rotation, rotation_speed)
pub struct ParticleDamping(pub f32);

// --- Marker Components (Tags) ---

pub struct BallTag;
pub struct PaddleTag;
pub struct BlockTag;
pub struct ParticleTag;

// --- Enums ---

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Menu,
    Ready,
    Playing,
    GameOver,
    Win,
}
