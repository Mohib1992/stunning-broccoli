use crate::entities::{Position, Velocity};
use hecs::World;

pub fn physics_system(world: &mut World, sub_dt: f32) {
    for (_id, (pos, vel)) in world.query_mut::<(&mut Position, &Velocity)>() {
        pos.0 += vel.0 * sub_dt;
    }
}
