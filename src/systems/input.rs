use crate::entities::{PaddleTag, Position, RectComp, Velocity};
use hecs::World;
use macroquad::prelude::*;

pub fn input_system(world: &mut World, _dt: f32) {
    let speed = 500.0;
    for (_id, (pos, vel, rect, _)) in
        world.query_mut::<(&mut Position, &mut Velocity, &RectComp, &PaddleTag)>()
    {
        let mut move_dir = 0.0;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            move_dir -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            move_dir += 1.0;
        }

        vel.0.x = move_dir * speed;

        // Predictive clamping to prevent crossing the edge
        let new_x = pos.0.x + vel.0.x * get_frame_time();
        if new_x < 0.0 {
            pos.0.x = 0.0;
            vel.0.x = 0.0;
        } else if new_x > screen_width() - rect.0.x {
            pos.0.x = screen_width() - rect.0.x;
            vel.0.x = 0.0;
        }
    }
}
