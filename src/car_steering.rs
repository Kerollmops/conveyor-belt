use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use interpolation::Lerp;

use crate::car_suspension::CarPhysics;
use crate::{CarWheel, RayCastWheelEntity};

pub fn update_car_steering(
    time: Res<Time>,
    mut car_query: Query<(
        &LinearVelocity,
        &AngularVelocity,
        &CarPhysics,
        &mut ExternalForce,
        &Transform,
        &CenterOfMass,
    )>,
    wheels_transforms_query: Query<&CarWheel, Without<CarPhysics>>,
    raycast_query: Query<(&RayCastWheelEntity, &RayCaster, &RayHits)>,
) {
    let Ok((
        &LinearVelocity(lin_vel),
        &AngularVelocity(ang_vel),
        car_physics,
        mut external_force,
        &car_transform,
        &CenterOfMass(car_center_of_mass),
    )) = car_query.get_single_mut()
    else {
        return;
    };

    let CarPhysics {
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

    for (&RayCastWheelEntity(entity), ray, hits) in &raycast_query {
        let car_wheel = wheels_transforms_query.get(entity).unwrap();

        assert!(hits.len() <= 1);
        let hit = hits.as_slice().get(0);

        // steering force
        if hit.is_some() {
            // World-space direction of the spring force
            let steering_dir = if matches!(car_wheel, CarWheel::FrontLeft | CarWheel::FrontRight) {
                if wheel_rotation <= 0.5 {
                    car_transform.forward().lerp(car_transform.right(), wheel_rotation / 0.5)
                } else {
                    car_transform.right().lerp(car_transform.back(), (wheel_rotation - 0.5) / 0.5)
                }
            } else {
                car_transform.right()
            };

            // World-space velocity of the suspension.
            let tire_world_vel = lin_vel + ang_vel.cross(car_transform.rotation * ray.origin);

            // What is the tire's velocity in the steering direction?
            // note that spring_dir is a unit vector, so this returns
            // the magnitude of tire_world_vec as projected on to steering_dir.
            let steering_vel = steering_dir.dot(tire_world_vel);

            // Forward speed of the car (in the direction of driving)
            // Normalized car speed
            let car_speed = car_transform.forward().dot(lin_vel);
            let normalized_speed = (car_speed.abs() / top_speed).clamp(0.0, 1.0);

            // The tire grip factor is lower the faster the steering velocity is.
            let tire_grip_factor =
                if matches!(car_wheel, CarWheel::FrontLeft | CarWheel::FrontRight) {
                    front_tire_max_grip_factor.lerp(
                        &front_tire_min_grip_factor,
                        &(normalized_speed * tire_grip_velocity_multiplier),
                    )
                } else {
                    back_tire_max_grip_factor.lerp(
                        &back_tire_min_grip_factor,
                        &(normalized_speed * tire_grip_velocity_multiplier),
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
            external_force.persistent = false;
            external_force.apply_force_at_point(
                steering_dir * tire_mass * desired_accel,
                car_transform.rotation * ray.origin,
                car_center_of_mass,
            );
        }
    }
}
