mod constants;
mod entities;
mod systems;

use hecs::World;
use macroquad::prelude::*;
use std::collections::VecDeque;

use crate::constants::*;
use crate::entities::*;
use crate::systems::collision::{CollisionEvents, collision_system};
use crate::systems::input::input_system;
use crate::systems::particle::particle_system;
use crate::systems::physics::physics_system;
use crate::systems::render::render_system;

fn spawn_paddle(world: &mut World, x: f32, y: f32) {
    world.spawn((
        Position(vec2(x, y)),
        Velocity(Vec2::ZERO),
        RectComp(vec2(PADDLE_WIDTH, PADDLE_HEIGHT)),
        PaddleTag,
    ));
}

fn spawn_ball(world: &mut World, x: f32, y: f32) {
    world.spawn((
        Position(vec2(x, y)),
        Velocity(Vec2::ZERO),
        CircleComp(BALL_RADIUS),
        TrailComp {
            positions: VecDeque::with_capacity(TRAIL_SIZE),
            max_size: TRAIL_SIZE,
        },
        BallTag,
    ));
}

fn spawn_blocks(world: &mut World, width: f32, height: f32) {
    for row in 0..BLOCK_ROWS {
        for col in 0..BLOCK_COLS {
            let color = match row {
                0 => RED,
                1 => ORANGE,
                2 => YELLOW,
                3 => GREEN,
                _ => BLUE,
            };
            world.spawn((
                Position(vec2(
                    BLOCK_PADDING + col as f32 * (width + BLOCK_PADDING),
                    screen_height() - 50.0 - row as f32 * (height + BLOCK_PADDING),
                )),
                RectComp(vec2(width, height)),
                ColorComp(color),
                BlockTag,
            ));
        }
    }
}

fn _window_conf() -> Conf {
    Conf {
        window_title: "BreakerBlock".to_owned(),
        window_width: 1000,
        window_height: 800,
        ..Default::default()
    }
}

#[macroquad::main("BreakerBlock")]
async fn main() {
    let mut game_state = GameState::Menu;
    let mut score = 0;
    let mut screenshake_time = 0.0;
    let mut world = World::new();

    let mut initial_dims = (screen_width(), screen_height());
    if initial_dims.0 == 0.0 {
        initial_dims = (1000.0, 800.0);
    }

    spawn_paddle(&mut world, initial_dims.0 / 2.0 - PADDLE_WIDTH / 2.0, 50.0);
    // spawn_ball moved to Ready state transition

    let block_w = (initial_dims.0 - (BLOCK_COLS as f32 + 1.0) * BLOCK_PADDING) / BLOCK_COLS as f32;
    spawn_blocks(&mut world, block_w, 25.0);

    loop {
        let dt = get_frame_time();
        clear_background(Color::from_rgba(15, 15, 25, 255));

        match game_state {
            GameState::Menu => {
                let text = "BREAKER BLOCK";
                let size = 80.0;
                let text_size = measure_text(text, None, size as u16, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_size.width / 2.0,
                    screen_height() / 2.0 - 50.0,
                    size,
                    SKYBLUE,
                );

                let subtext = "PRESS SPACE TO START";
                let subsize = 30.0;
                let subtext_size = measure_text(subtext, None, subsize as u16, 1.0);
                draw_text(
                    subtext,
                    screen_width() / 2.0 - subtext_size.width / 2.0,
                    screen_height() / 2.0 + 20.0,
                    subsize,
                    WHITE,
                );

                if is_key_pressed(KeyCode::Space) {
                    let dims = (screen_width(), screen_height());
                    spawn_ball(&mut world, dims.0 / 2.0, 50.0 + BALL_RADIUS);
                    game_state = GameState::Ready;
                }
            }
            GameState::Ready | GameState::Playing | GameState::GameOver | GameState::Win => {
                // Gameplay Logic
                if let GameState::Ready | GameState::Playing = game_state {
                    input_system(&mut world, dt);
                }

                if let GameState::Ready = game_state {
                    let mut paddle_x = 0.0;
                    let mut paddle_y = 0.0;
                    for (_id, (pos, _tag)) in world.query_mut::<(&Position, &PaddleTag)>() {
                        paddle_x = pos.0.x;
                        paddle_y = pos.0.y;
                    }
                    for (_id, (pos, vel, _tag)) in
                        world.query_mut::<(&mut Position, &mut Velocity, &BallTag)>()
                    {
                        pos.0 = vec2(
                            paddle_x + PADDLE_WIDTH / 2.0,
                            paddle_y + PADDLE_HEIGHT + BALL_RADIUS,
                        );
                        vel.0 = Vec2::ZERO;
                    }

                    if is_key_pressed(KeyCode::Space) {
                        for (_id, (vel, _tag)) in world.query_mut::<(&mut Velocity, &BallTag)>() {
                            vel.0 = vec2(0.5, 1.0).normalize() * BALL_SPEED;
                        }
                        game_state = GameState::Playing;
                    }
                }

                if let GameState::Playing = game_state {
                    const SUBSTEPS: usize = 10;
                    let sub_dt = dt / SUBSTEPS as f32;

                    for _ in 0..SUBSTEPS {
                        physics_system(&mut world, sub_dt);

                        let mut events = CollisionEvents {
                            screenshake_time: 0.0,
                            score_delta: 0,
                            game_over: false,
                            blocks_left: 0,
                        };
                        collision_system(&mut world, &mut events);

                        score += events.score_delta;
                        if events.screenshake_time > 0.0 {
                            screenshake_time = events.screenshake_time;
                        }
                        if events.game_over {
                            game_state = GameState::GameOver;
                            break;
                        }
                        if events.blocks_left == 0 {
                            let mut blocks_exist = false;
                            for (_id, _tag) in world.query_mut::<&BlockTag>() {
                                blocks_exist = true;
                                break;
                            }
                            if !blocks_exist {
                                game_state = GameState::Win;
                                break;
                            }
                        }
                    }
                    particle_system(&mut world, dt);
                }

                // Camera and Rendering (Persistent across gameplay and end-screens)
                if screenshake_time > 0.0 {
                    screenshake_time -= dt;
                }
                let camera_offset = if screenshake_time > 0.0 {
                    vec2(rand::gen_range(-5.0, 5.0), rand::gen_range(-5.0, 5.0))
                } else {
                    Vec2::ZERO
                };

                // Clear background logic: Only render if playing/ready OR shaking
                if game_state == GameState::Ready
                    || game_state == GameState::Playing
                    || screenshake_time > 0.0
                {
                    push_camera_state();
                    let mut camera = Camera2D {
                        target: vec2(screen_width() / 2.0, screen_height() / 2.0) + camera_offset,
                        ..Camera2D::from_display_rect(Rect::new(
                            0.0,
                            screen_height(),
                            screen_width(),
                            -screen_height(),
                        ))
                    };
                    camera.rotation = 180.0;
                    camera.zoom.x *= -1.0;
                    set_camera(&camera);

                    render_system(&mut world);

                    pop_camera_state();
                }

                draw_text(&format!("SCORE: {}", score), 20.0, 40.0, 40.0, WHITE);

                // End-Screen Overlays
                if (game_state == GameState::GameOver || game_state == GameState::Win)
                    && screenshake_time <= 0.0
                {
                    let (text, color) = if let GameState::Win = game_state {
                        ("VICTORY!", GREEN)
                    } else {
                        ("GAME OVER", RED)
                    };
                    let text_size = measure_text(text, None, 80, 1.0);
                    draw_text(
                        text,
                        screen_width() / 2.0 - text_size.width / 2.0,
                        screen_height() / 2.0,
                        80.0,
                        color,
                    );

                    let subtext = "PRESS SPACE TO RESTART";
                    let subtext_size = measure_text(subtext, None, 30, 1.0);
                    draw_text(
                        subtext,
                        screen_width() / 2.0 - subtext_size.width / 2.0,
                        screen_height() / 2.0 + 50.0,
                        30.0,
                        WHITE,
                    );

                    if is_key_pressed(KeyCode::Space) {
                        score = 0;
                        world.clear();
                        let dims = (screen_width(), screen_height());
                        spawn_paddle(&mut world, dims.0 / 2.0 - PADDLE_WIDTH / 2.0, 50.0);
                        spawn_ball(&mut world, dims.0 / 2.0, 50.0 + BALL_RADIUS);
                        let block_w = (dims.0 - (BLOCK_COLS as f32 + 1.0) * BLOCK_PADDING)
                            / BLOCK_COLS as f32;
                        spawn_blocks(&mut world, block_w, 25.0);
                        game_state = GameState::Ready;
                    }
                }
            }
        }

        next_frame().await
    }
}
