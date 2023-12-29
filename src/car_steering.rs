use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;
use bevy_rapier3d::prelude::*;
use bevy_vector_shapes::painter::{ShapeConfig, ShapePainter};
use bevy_vector_shapes::shapes::LinePainter;

use crate::car_suspension::CarPhysics;

pub fn update_car_steering(
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut car_query: Query<(
        &RapierRigidBodyHandle,
        &mut CarPhysics,
        &mut ExternalForce,
        &mut ExternalImpulse,
        &mut Transform,
    )>,
) {
    let Ok((handle, mut car_physics, mut car_force, mut car_impulse, car_transform)) =
        car_query.get_single_mut()
    else {
        return;
    };

    let CarPhysics { car_size, max_suspension, tire_mass, tire_grip_factor, .. } = *car_physics;

    let front_right = car_transform.translation
        + (car_transform.down() * car_size.y + car_transform.forward() * car_size.z)
        + (car_transform.right() * car_size.x);

    let front_left = car_transform.translation
        + (car_transform.down() * car_size.y + car_transform.forward() * car_size.z)
        + (car_transform.left() * car_size.x);

    let back_right = car_transform.translation
        + (car_transform.down() * car_size.y + car_transform.back() * car_size.z)
        + (car_transform.right() * car_size.x);

    let back_left = car_transform.translation
        + (car_transform.down() * car_size.y + car_transform.back() * car_size.z)
        + (car_transform.left() * car_size.x);

    let wheels = [front_right, front_left, back_right, back_left];

    for (i, &wheel) in wheels.iter().enumerate() {
        let hit = rapier_context.cast_ray(
            wheel,
            car_transform.down(),
            max_suspension,
            true,
            QueryFilter::only_fixed(),
        );

        // steering force
        if hit.is_some() {
            // World-space direction of the spring force
            // TODO
            let steering_dir = if i == 0 || i == 1 {
                car_transform.forward().lerp(car_transform.right(), 0.5)
            } else {
                car_transform.forward()
            };

            // Fetch the rigid body from the rapier world.
            let rigid_body = rapier_context.bodies.get(handle.0).unwrap();

            // World-space velocity of the suspension.
            let tire_world_vel = rigid_body.velocity_at_point(&wheel.into()).into();

            // WHat is the tire's velocity in the steering direction?
            // note that spring_dir is a unit vector, so this returns
            // the magnitude of tire_world_vec as projected on to steering_dir.
            let steering_vel = steering_dir.dot(tire_world_vel);

            // The change in velocity into an acceleration (acceleration = change in vel / time)
            // this will produce the acceleration necessary to change the velocity by
            // desired_vel_change in 1 physics step
            let desired_vel_change = -steering_vel * tire_grip_factor;

            // Force = Mass * Acceleration, so multiply by the mass of the tire and apply as a force!
            let add_force = ExternalForce::at_point(
                steering_dir * tire_mass * desired_vel_change,
                wheel,
                car_transform.translation,
            );

            car_force.force += add_force.force;
            car_force.torque += add_force.torque;
        }
    }
}
