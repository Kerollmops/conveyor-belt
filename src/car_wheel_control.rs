use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::dynamics::Velocity;
use bevy_rapier3d::pipeline::QueryFilter;
use bevy_rapier3d::plugin::RapierContext;
use interpolation::Lerp;

use crate::car_suspension::CarPhysics;
use crate::CarWheel;

pub fn update_car_wheel_rotation_speed(
    mut car_query: Query<(&mut CarPhysics, &Velocity, &Transform)>,
) {
    let Ok((mut car_physics, velocity, car_transform)) = car_query.get_single_mut() else {
        return;
    };

    let CarPhysics { wheel_rotation_speed, top_speed, .. } = car_physics.as_mut();

    // Forward speed of the car (in the direction of driving)
    let car_speed = car_transform.forward().dot(velocity.linvel);
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
    mut car_query: Query<(&mut CarPhysics, &Transform), Without<CarWheel>>,
    rapier_context: Res<RapierContext>,
    mut wheels_transforms_query: Query<(&CarWheel, &mut Transform), Without<CarPhysics>>,
) {
    let Ok((car_physics, car_transform)) = car_query.get_single_mut() else {
        return;
    };

    let CarPhysics { wheel_rotation, chassis_size, max_suspension, .. } = *car_physics;

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

    let wheels_pos = [front_right, front_left, back_right, back_left];
    let mut wheels_transforms: Vec<_> = wheels_transforms_query.iter_mut().collect();
    wheels_transforms.sort_by_key(|(w, _)| *w);

    let wheel_half_height = 0.3;

    for ((wheel, mut transform), wheel_pos) in wheels_transforms.into_iter().zip(wheels_pos) {
        let hit = rapier_context.cast_ray_and_get_normal(
            wheel_pos,
            car_transform.down(),
            max_suspension,
            true,
            QueryFilter::only_fixed(),
        );

        let angle = if wheel_rotation <= 0.5 {
            (PI / 3.0).lerp(&0.0, &(wheel_rotation / 0.5))
        } else {
            (2.0 * PI).lerp(&(5.0 * PI / 3.0), &((wheel_rotation - 0.5) / 0.5))
        };

        // suspension spring force
        *transform = match hit {
            Some((_entity, ray_intersection)) => {
                let mut new_transform = match wheel {
                    CarWheel::FrontLeft | CarWheel::FrontRight => {
                        transform.with_rotation(Quat::from_rotation_y(angle))
                    }
                    CarWheel::BackLeft | CarWheel::BackRight => *transform,
                };
                new_transform.translation.y =
                    (1.0 - ray_intersection.toi) * max_suspension + wheel_half_height;
                new_transform
            }
            None => match wheel {
                CarWheel::FrontLeft | CarWheel::FrontRight => {
                    transform.with_rotation(Quat::from_rotation_y(angle))
                }
                CarWheel::BackLeft | CarWheel::BackRight => *transform,
            },
        }
    }
}
