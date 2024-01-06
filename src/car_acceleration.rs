use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use interpolation::Lerp;

use crate::car_suspension::CarPhysics;
use crate::{CarWheel, RayCastWheelEntity};

pub fn car_acceleration(
    keys: Res<Input<KeyCode>>,
    mut car_query: Query<
        (&CarPhysics, &LinearVelocity, &mut ExternalForce, &Transform, &CenterOfMass),
        Without<CarWheel>,
    >,
    wheels_transforms_query: Query<(&CarWheel, &Transform), Without<CarPhysics>>,
    raycast_query: Query<(&RayCastWheelEntity, &RayCaster, &RayHits)>,
) {
    let Ok((
        car_physics,
        &LinearVelocity(lin_vel),
        mut external_force,
        &car_transform,
        &CenterOfMass(car_center_of_mass),
    )) = car_query.get_single_mut()
    else {
        return;
    };

    let CarPhysics { top_speed, .. } = *car_physics;

    let accel_input = if keys.pressed(KeyCode::Up) || keys.pressed(KeyCode::Down) {
        top_speed
    } else {
        top_speed / 10.0
    };

    for (&RayCastWheelEntity(entity), ray, hits) in &raycast_query {
        let (car_wheel, &wheel_transform) = wheels_transforms_query.get(entity).unwrap();

        assert!(hits.len() <= 1);
        let hit = hits.as_slice().get(0);

        // acceleration / braking
        if hit.is_some() && matches!(car_wheel, CarWheel::BackRight | CarWheel::BackLeft) {
            // Forward speed of the car (in the direction of driving)
            let car_speed = car_transform.forward().dot(lin_vel);

            // World-space direction of the acceleration/braking force.
            #[allow(clippy::collapsible_else_if)]
            let accel_dir = if keys.pressed(KeyCode::Up) {
                (car_transform * wheel_transform).forward()
            } else if keys.pressed(KeyCode::Down) {
                (car_transform * wheel_transform).back()
            } else {
                if car_speed > 0.0 {
                    (car_transform * wheel_transform).back()
                } else if car_speed < 0.0 {
                    (car_transform * wheel_transform).forward()
                } else {
                    Vec3::ZERO
                }
            };

            if accel_input > 0.0 {
                // Normalized car speed
                let normalized_speed = (car_speed.abs() / top_speed).clamp(0.0, 1.0);

                // Available torque
                let available_torque = evaluate_power_curve(normalized_speed) * accel_input;

                external_force.persistent = false;
                external_force.apply_force_at_point(
                    accel_dir * available_torque,
                    car_transform.rotation * ray.origin,
                    car_center_of_mass,
                );
            }
        }
    }
}

fn evaluate_power_curve(normalized_speed: f32) -> f32 {
    let teeing_off = 0.4;
    let near_limit = 0.75;
    if normalized_speed <= teeing_off {
        0.5.lerp(&1.0, &(normalized_speed / teeing_off))
    } else if normalized_speed <= near_limit {
        1.0
    } else if normalized_speed != 1.0 {
        1.0.lerp(&0.3, &((normalized_speed - near_limit) / (1.0 - near_limit)))
    } else {
        0.0
    }
}
