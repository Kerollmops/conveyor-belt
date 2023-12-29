use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use lerp::Lerp;

use crate::car_suspension::CarPhysics;

pub fn update_car_steering(
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut car_query: Query<(
        &RapierRigidBodyHandle,
        &CarPhysics,
        &Velocity,
        &mut ExternalForce,
        &mut Transform,
    )>,
) {
    let Ok((handle, car_physics, velocity, mut car_force, car_transform)) =
        car_query.get_single_mut()
    else {
        return;
    };

    let CarPhysics {
        chassis_size,
        max_suspension,
        tire_mass,
        front_tire_max_grip_factor,
        back_tire_max_grip_factor,
        front_tire_min_grip_factor,
        back_tire_min_grip_factor,
        tire_grip_velocity_multiplier,
        wheel_rotation,
        top_speed,
        ..
    } = *car_physics;

    let front_right = car_transform.translation
        + (car_transform.down() * chassis_size.y + car_transform.forward() * chassis_size.z)
        + (car_transform.right() * chassis_size.x);

    let front_left = car_transform.translation
        + (car_transform.down() * chassis_size.y + car_transform.forward() * chassis_size.z)
        + (car_transform.left() * chassis_size.x);

    let back_right = car_transform.translation
        + (car_transform.down() * chassis_size.y + car_transform.back() * chassis_size.z)
        + (car_transform.right() * chassis_size.x);

    let back_left = car_transform.translation
        + (car_transform.down() * chassis_size.y + car_transform.back() * chassis_size.z)
        + (car_transform.left() * chassis_size.x);

    let wheels = [front_right, front_left, back_right, back_left];
    let is_front_wheel = |id: usize| id == 0 || id == 1;

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
            let steering_dir = if is_front_wheel(i) {
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

            // What is the tire's velocity in the steering direction?
            // note that spring_dir is a unit vector, so this returns
            // the magnitude of tire_world_vec as projected on to steering_dir.
            let steering_vel = steering_dir.dot(tire_world_vel);

            // Forward speed of the car (in the direction of driving)
            // Normalized car speed
            let car_speed = car_transform.forward().dot(velocity.linvel);
            let normalized_speed = (car_speed.abs() / top_speed).clamp(0.0, 1.0);

            // The tire grip factor is lower the faster the steering velocity is.
            let tire_grip_factor = if is_front_wheel(i) {
                front_tire_max_grip_factor.lerp_bounded(
                    front_tire_min_grip_factor,
                    normalized_speed * tire_grip_velocity_multiplier,
                )
            } else {
                back_tire_max_grip_factor.lerp_bounded(
                    back_tire_min_grip_factor,
                    normalized_speed * tire_grip_velocity_multiplier,
                )
            };

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
