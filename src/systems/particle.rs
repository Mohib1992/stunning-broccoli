use crate::entities::*;
use hecs::World;

pub fn particle_system(world: &mut World, dt: f32) {
    for (_id, (_pos, vel, rot, size, damping, _tag)) in world.query_mut::<(
        &mut Position,
        &mut Velocity,
        &mut ParticleRotation,
        &mut ParticleSize,
        &ParticleDamping,
        &ParticleTag,
    )>() {
        // Friction / Damping
        vel.0 *= damping.0;

        // Rotation
        rot.0 += rot.1 * dt;

        // Scaling (Shrink over time)
        // Here we just shrink it slightly every frame for simplicity,
        // or we could use the lifetime to be more precise.
        size.0 *= 0.98;
    }
}
