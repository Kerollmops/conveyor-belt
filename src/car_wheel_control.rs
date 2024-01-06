use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use interpolation::Lerp;

use crate::car_suspension::CarPhysics;
use crate::{CarWheel, RayCastWheelEntity};

pub fn update_car_wheel_rotation_speed(
    mut car_query: Query<(&mut CarPhysics, &LinearVelocity, &Transform)>,
) {
    let Ok((mut car_physics, &LinearVelocity(lin_vel), car_transform)) = car_query.get_single_mut()
    else {
        return;
    };

    let CarPhysics { wheel_rotation_speed, top_speed, .. } = car_physics.as_mut();

    // Forward speed of the car (in the direction of driving)
    let car_speed = car_transform.forward().dot(lin_vel);
    // Normalized car speed
    let normalized_speed = (car_speed.abs() / *top_speed).clamp(0.0, 1.0);

    // The faster you go the slower the wheel rotation speed
    let increased_normalized_speed = (normalized_speed * 10.0).clamp(0.0, 1.0);

    *wheel_rotation_speed = 0.1.lerp(&1.5, &(1.0 - increased_normalized_speed));
}

pub fn update_car_wheel_control(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut car_query: Query<&mut CarPhysics>,
) {
    let Ok(mut car_physics) = car_query.get_single_mut() else {
        return;
    };

    let CarPhysics { wheel_rotation, wheel_rotation_speed, .. } = car_physics.as_mut();

    if keys.pressed(KeyCode::Left) {
        *wheel_rotation -= *wheel_rotation_speed * time.delta_seconds();
    }
    if keys.pressed(KeyCode::Right) {
        *wheel_rotation += *wheel_rotation_speed * time.delta_seconds();
    }

    // Move the wheels back to position
    if !keys.pressed(KeyCode::Left) && !keys.pressed(KeyCode::Right) {
        *wheel_rotation = if *wheel_rotation <= 0.5 {
            (*wheel_rotation + *wheel_rotation_speed * time.delta_seconds()).min(0.5)
        } else {
            (*wheel_rotation - *wheel_rotation_speed * time.delta_seconds()).max(0.5)
        };
    }

    *wheel_rotation = wheel_rotation.clamp(0.2, 0.8);
}

pub fn update_car_wheels(
    mut car_query: Query<&CarPhysics, Without<CarWheel>>,
    mut wheels_transforms_query: Query<(&CarWheel, &mut Transform), Without<CarPhysics>>,
    raycast_query: Query<(&RayCastWheelEntity, &RayCaster, &RayHits)>,
) {
    let Ok(car_physics) = car_query.get_single_mut() else {
        return;
    };

    let CarPhysics { wheel_rotation, max_suspension, .. } = *car_physics;

    let wheel_half_height = 0.3;

    for (&RayCastWheelEntity(entity), _, hits) in &raycast_query {
        let (car_wheel, mut wheel_transform) = wheels_transforms_query.get_mut(entity).unwrap();

        assert!(hits.len() <= 1);
        let hit = hits.as_slice().get(0);

        let angle = if wheel_rotation <= 0.5 {
            (PI / 3.0).lerp(&0.0, &(wheel_rotation / 0.5))
        } else {
            (2.0 * PI).lerp(&(5.0 * PI / 3.0), &((wheel_rotation - 0.5) / 0.5))
        };

        *wheel_transform = match hit {
            Some(RayHitData { time_of_impact, .. }) => {
                let mut new_transform = match car_wheel {
                    CarWheel::FrontLeft | CarWheel::FrontRight => {
                        wheel_transform.with_rotation(Quat::from_rotation_y(angle))
                    }
                    CarWheel::BackLeft | CarWheel::BackRight => *wheel_transform,
                };
                new_transform.translation.y =
                    (1.0 - (time_of_impact / max_suspension)) * max_suspension + wheel_half_height;
                new_transform
            }
            None => match car_wheel {
                CarWheel::FrontLeft | CarWheel::FrontRight => {
                    wheel_transform.with_rotation(Quat::from_rotation_y(angle))
                }
                CarWheel::BackLeft | CarWheel::BackRight => *wheel_transform,
            },
        }
    }
}
