use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use lerp::Lerp;

use crate::car_suspension::CarPhysics;

pub fn car_controls(
    keys: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
    mut car_query: Query<(&CarPhysics, &Velocity, &mut ExternalForce, &mut Transform)>,
) {
    let Ok((car_physics, velocity, mut car_force, car_transform)) = car_query.get_single_mut()
    else {
        return;
    };

    let CarPhysics { car_size, max_suspension, top_speed, .. } = *car_physics;

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

    let mut accel_input = 0.0;
    if keys.pressed(KeyCode::W) {
        accel_input += top_speed;
    }

    for (i, wheel) in wheels.into_iter().enumerate() {
        let hit = rapier_context.cast_ray(
            wheel,
            car_transform.down(),
            max_suspension,
            true,
            QueryFilter::only_fixed(),
        );

        // acceleration / braking
        if hit.is_some() {
            // World-space direction of the acceleration/braking force.
            let accel_dir = car_transform.forward();

            if accel_input > 0.0 {
                // Forward speed of the car (in the direction of driving)
                let car_speed = car_transform.forward().dot(velocity.linvel);

                // Normalized car speed
                let normalized_speed = (car_speed.abs() / top_speed).clamp(0.0, 1.0);

                fn evaluate_power_curve(normalized_speed: f32) -> f32 {
                    let teeing_off = 0.4;
                    let near_limit = 0.75;
                    if normalized_speed <= teeing_off {
                        0.5.lerp_bounded(1.0, normalized_speed / teeing_off)
                    } else if normalized_speed <= near_limit {
                        1.0
                    } else if normalized_speed != 1.0 {
                        1.0.lerp_bounded(0.3, (normalized_speed - near_limit) / (1.0 - near_limit))
                    } else {
                        0.0
                    }
                }

                // Available torque
                let available_torque =
                    dbg!(dbg!(evaluate_power_curve(normalized_speed)) * accel_input);

                let add_force = ExternalForce::at_point(
                    accel_dir * available_torque,
                    wheel,
                    car_transform.translation,
                );

                car_force.force += add_force.force;
                car_force.torque += add_force.torque;
            }
        }
    }
}
