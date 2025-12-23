use crate::constants::*;
use crate::entities::*;
use hecs::World;
use macroquad::prelude::*;

pub fn render_system(world: &mut World) {
    // Draw trail first
    for (_id, (pos, trail, _tag)) in world.query_mut::<(&Position, &mut TrailComp, &BallTag)>() {
        // Update trail data (this could be in its own system but for simplicity...)
        trail.positions.push_front(pos.0);
        if trail.positions.len() > trail.max_size {
            trail.positions.pop_back();
        }

        for (i, p) in trail.positions.iter().enumerate() {
            let alpha = 1.0 - (i as f32 / trail.max_size as f32);
            draw_circle(
                p.x,
                p.y,
                10.0 * alpha * 0.8, // Ball radius approx
                Color::from_rgba(255, 255, 255, (alpha * 100.0) as u8),
            );
        }
    }

    // Draw blocks
    for (_id, (pos, rect, color, _tag)) in
        world.query_mut::<(&Position, &RectComp, &ColorComp, &BlockTag)>()
    {
        draw_rectangle(pos.0.x, pos.0.y, rect.0.x, rect.0.y, color.0);
        draw_rectangle_lines(
            pos.0.x,
            pos.0.y,
            rect.0.x,
            rect.0.y,
            2.0,
            Color::from_rgba(0, 0, 0, 50),
        );
    }

    // Draw paddle
    for (_id, (pos, rect, _tag)) in world.query_mut::<(&Position, &RectComp, &PaddleTag)>() {
        draw_rectangle(pos.0.x, pos.0.y, rect.0.x, rect.0.y, SKYBLUE);
    }

    // Draw ball
    for (_id, (pos, _tag)) in world.query_mut::<(&Position, &BallTag)>() {
        // Bloom / Glow effect (layered circles)
        for i in 1..=5 {
            let alpha = 0.15 / (i as f32);
            let radius = BALL_RADIUS + (i as f32 * 4.0);
            draw_circle(
                pos.0.x,
                pos.0.y,
                radius,
                Color::from_rgba(35, 206, 250, (alpha * 255.0) as u8), // SKYBLUE
            );
        }
        draw_poly(pos.0.x, pos.0.y, 20, BALL_RADIUS, 0.0, WHITE);
    }

    // Draw particles
    for (_id, (pos, size, color, lifetime, rot, _tag)) in world.query_mut::<(
        &Position,
        &ParticleSize,
        &ColorComp,
        &ParticleLifetime,
        &ParticleRotation,
        &ParticleTag,
    )>() {
        let alpha = (lifetime.0 * 2.0).clamp(0.0, 1.0);
        let mut c = color.0;
        c.a = alpha;

        draw_rectangle_ex(
            pos.0.x,
            pos.0.y,
            size.0,
            size.0 * 2.5, // Shard / Rect shape
            DrawRectangleParams {
                rotation: rot.0,
                color: c,
                ..Default::default()
            },
        );
    }
}
