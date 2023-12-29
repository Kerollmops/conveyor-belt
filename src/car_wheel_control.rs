use bevy::prelude::*;

use crate::car_suspension::CarPhysics;

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

    *wheel_rotation = wheel_rotation.clamp(0.2, 0.8);
}
