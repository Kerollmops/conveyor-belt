use std::f32::consts::PI;

use bevy::prelude::*;
use lerp::Lerp;

use crate::car_suspension::CarPhysics;
use crate::CarWheel;

pub fn update_car_wheel_control(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut car_query: Query<&mut CarPhysics>,
) {
    let Ok(mut car_physics) = car_query.get_single_mut() else {
        return;
    };

    let CarPhysics { wheel_rotation, wheel_rotation_speed, .. } = car_physics.as_mut();

    if keys.pressed(KeyCode::A) {
        *wheel_rotation -= *wheel_rotation_speed * time.delta_seconds();
    }
    if keys.pressed(KeyCode::D) {
        *wheel_rotation += *wheel_rotation_speed * time.delta_seconds();
    }

    // Move the wheels back to position
    if !keys.pressed(KeyCode::A) && !keys.pressed(KeyCode::D) {
        *wheel_rotation = if *wheel_rotation <= 0.5 {
            (*wheel_rotation + *wheel_rotation_speed * time.delta_seconds()).min(0.5)
        } else {
            (*wheel_rotation - *wheel_rotation_speed * time.delta_seconds()).max(0.5)
        };
    }

    *wheel_rotation = wheel_rotation.clamp(0.2, 0.8);
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
