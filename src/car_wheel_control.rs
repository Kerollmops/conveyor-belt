use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::dynamics::Velocity;
use lerp::Lerp;

use crate::car_suspension::CarPhysics;
use crate::CarWheel;

pub fn update_car_wheel_control(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut car_query: Query<(&mut CarPhysics, &Velocity, &mut Transform)>,
) {
    let Ok((mut car_physics, velocity, car_transform)) = car_query.get_single_mut() else {
        return;
    };

    let CarPhysics { wheel_rotation, wheel_rotation_speed, top_speed, .. } = car_physics.as_mut();

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

    // Forward speed of the car (in the direction of driving)
    let car_speed = car_transform.forward().dot(velocity.linvel);
    // Normalized car speed
    let normalized_speed = (car_speed.abs() / *top_speed).clamp(0.0, 1.0);

    // The faster you go the less wheel rotation amplitude you got
    let amplitude = 0.4 * (1.0 - normalized_speed);

    *wheel_rotation = wheel_rotation.clamp(0.5 - amplitude, 0.5 + amplitude);
}

pub fn update_car_wheels(
    mut car_query: Query<&mut CarPhysics>,
    mut wheels_transforms_query: Query<(&CarWheel, &mut Transform)>,
) {
    let Ok(car_physics) = car_query.get_single_mut() else {
        return;
    };

    let CarPhysics { wheel_rotation, .. } = *car_physics;

    for (wheel, mut transform) in &mut wheels_transforms_query {
        if matches!(wheel, CarWheel::FrontLeft | CarWheel::FrontRight) {
            let angle = if wheel_rotation <= 0.5 {
                (PI / 3.0).lerp(0.0, wheel_rotation / 0.5)
            } else {
                (2.0 * PI).lerp(5.0 * PI / 3.0, (wheel_rotation - 0.5) / 0.5)
            };
            transform.rotation = Quat::from_rotation_y(angle);
        }
    }
}
