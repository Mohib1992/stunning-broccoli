use crate::constants::*;
use crate::entities::*;
use hecs::World;
use macroquad::prelude::*;

pub struct CollisionEvents {
    pub screenshake_time: f32,
    pub score_delta: i32,
    pub game_over: bool,
    pub blocks_left: usize,
}

pub fn collision_system(world: &mut World, events: &mut CollisionEvents) {
    let mut to_destroy = Vec::new();
    let mut particles_to_spawn = Vec::new();

    // Collect paddle data
    let mut paddles = Vec::new();
    for (id, (pos, rect, _tag)) in world.query_mut::<(&Position, &RectComp, &PaddleTag)>() {
        paddles.push((id, pos.0, rect.0));
    }

    // Collect block data
    let mut blocks = Vec::new();
    events.blocks_left = 0;
    for (id, (pos, rect, color, _tag)) in
        world.query_mut::<(&Position, &RectComp, &ColorComp, &BlockTag)>()
    {
        blocks.push((id, pos.0, rect.0, color.0));
        events.blocks_left += 1;
    }

    // Ball-Wall and Ball-Paddle and Ball-Block
    for (_ball_id, (ball_pos, ball_vel, _ball_tag)) in
        world.query_mut::<(&mut Position, &mut Velocity, &BallTag)>()
    {
        // Wall collisions
        if ball_pos.0.x - BALL_RADIUS < 0.0 {
            ball_pos.0.x = BALL_RADIUS;
            ball_vel.0.x *= -1.0;
        } else if ball_pos.0.x + BALL_RADIUS > screen_width() {
            ball_pos.0.x = screen_width() - BALL_RADIUS;
            ball_vel.0.x *= -1.0;
        }

        // Flipped Y Logic (0 is bottom, height is top)
        if ball_pos.0.y + BALL_RADIUS > screen_height() {
            ball_pos.0.y = screen_height() - BALL_RADIUS;
            ball_vel.0.y *= -1.0;
        }

        if ball_pos.0.y - BALL_RADIUS + 50.0 < 0.0 {
            events.game_over = true;
            events.screenshake_time = 0.3;
        }

        // Paddle collision (Ball falling onto paddle)
        for (_id, p_pos, p_rect) in &paddles {
            let closest_paddle_point = vec2(
                ball_pos.0.x.clamp(p_pos.x, p_pos.x + p_rect.x),
                ball_pos.0.y.clamp(p_pos.y, p_pos.y + p_rect.y),
            );
            let dist_paddle = ball_pos.0.distance(closest_paddle_point);

            if dist_paddle < BALL_RADIUS && ball_vel.0.y < 0.0 {
                let collision_normal = (ball_pos.0 - closest_paddle_point).normalize_or_zero();
                let penetration = BALL_RADIUS - dist_paddle;
                ball_pos.0 += collision_normal * penetration;

                ball_vel.0.y *= -1.0;
                let hit_factor = (ball_pos.0.x - (p_pos.x + p_rect.x / 2.0)) / (p_rect.x / 2.0);
                ball_vel.0.x = hit_factor * BALL_SPEED * 1.5;
                ball_vel.0 = ball_vel.0.normalize() * BALL_SPEED;
            }
        }

        // Block collisions
        let mut collisions_this_substep = 0;
        for (id, b_pos, b_rect, b_color) in &blocks {
            if to_destroy.contains(id) {
                continue;
            }

            let closest_point = vec2(
                ball_pos.0.x.clamp(b_pos.x, b_pos.x + b_rect.x),
                ball_pos.0.y.clamp(b_pos.y, b_pos.y + b_rect.y),
            );
            let dist = ball_pos.0.distance(closest_point);

            if dist < BALL_RADIUS {
                to_destroy.push(*id);
                events.score_delta += 10;

                // Prepare particles
                let particle_count = rand::gen_range(12, 50);
                for _ in 0..particle_count {
                    let is_spark = rand::gen_range(0, 5) == 0;
                    let p_color = if is_spark {
                        if rand::gen_range(0, 2) == 0 {
                            WHITE
                        } else {
                            YELLOW
                        }
                    } else {
                        *b_color
                    };

                    particles_to_spawn.push((
                        closest_point,
                        vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)).normalize()
                            * rand::gen_range(100.0, 250.0),
                        p_color,
                    ));
                }

                let collision_normal = (ball_pos.0 - closest_point).normalize_or_zero();
                let penetration = BALL_RADIUS - dist;

                if collision_normal == Vec2::ZERO {
                    ball_vel.0 *= -1.0;
                } else {
                    ball_pos.0 += collision_normal * penetration;
                    ball_vel.0 =
                        ball_vel.0 - 2.0 * ball_vel.0.dot(collision_normal) * collision_normal;
                }

                collisions_this_substep += 1;
                if collisions_this_substep >= 2 {
                    break;
                }
            }
        }
    }

    // Process destruction and spawning outside of queries to avoid borrow checker issues
    for entity in to_destroy {
        let _ = world.despawn(entity);
        events.blocks_left -= 1;
    }

    for (pos, vel, color) in particles_to_spawn {
        world.spawn((
            Position(pos),
            Velocity(vel),
            ColorComp(color),
            ParticleLifetime(rand::gen_range(0.4, 0.8)),
            ParticleSize(rand::gen_range(2.0, 6.0)),
            ParticleRotation(0.0, rand::gen_range(-5.0, 5.0)),
            ParticleDamping(0.96),
            ParticleTag,
        ));
    }

    // Update particles lifetime
    let dt = get_frame_time();
    let mut expired = Vec::new();
    for (id, (lifetime, _tag)) in world.query_mut::<(&mut ParticleLifetime, &ParticleTag)>() {
        lifetime.0 -= dt;
        if lifetime.0 <= 0.0 {
            expired.push(id);
        }
    }
    for id in expired {
        let _ = world.despawn(id);
    }
}
