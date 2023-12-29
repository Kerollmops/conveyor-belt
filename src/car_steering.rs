use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::car_suspension::CarPhysics;

pub fn update_car_steering(
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut car_query: Query<(&RapierRigidBodyHandle, &CarPhysics, &mut ExternalForce, &mut Transform)>,
) {
    let Ok((handle, car_physics, mut car_force, car_transform)) = car_query.get_single_mut() else {
        return;
    };

    let CarPhysics {
        car_size, max_suspension, tire_mass, tire_grip_factor, wheel_rotation, ..
    } = *car_physics;

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

    for (i, wheel) in wheels.into_iter().enumerate() {
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
            let steering_dir = if i == 0 || i == 1 {
                if wheel_rotation <= 0.5 {
                    car_transform.forward().lerp(car_transform.right(), wheel_rotation / 0.5)
                } else {
                    car_transform.right().lerp(car_transform.back(), (wheel_rotation - 0.5) / 0.5)
                }
            } else {
                car_transform.right()
            };

            // Fetch the rigid body from the rapier world.
            let rigid_body = rapier_context.bodies.get(handle.0).unwrap();

            // World-space velocity of the suspension.
            let tire_world_vel = rigid_body.velocity_at_point(&wheel.into()).into();

            // WHat is the tire's velocity in the steering direction?
            // note that spring_dir is a unit vector, so this returns
            // the magnitude of tire_world_vec as projected on to steering_dir.
            let steering_vel = steering_dir.dot(tire_world_vel);

            // The change in velocity that we're loking for is -steering_vel * grip_factor
            // grip_factor is in range 0-1, 0 means no grip, 1 means full grip
            let desired_vel_change = -steering_vel * tire_grip_factor;

            // The change in velocity into an acceleration (acceleration = change in vel / time)
            // this will produce the acceleration necessary to change the velocity by
            // desired_vel_change in 1 physics step
            let desired_accel = desired_vel_change / time.delta_seconds();

            // Force = Mass * Acceleration, so multiply by the mass of the tire and apply as a force!
            let add_force = ExternalForce::at_point(
                steering_dir * tire_mass * desired_accel,
                wheel,
                car_transform.translation,
            );

            car_force.force += add_force.force;
            car_force.torque += add_force.torque;
        }
    }
}
