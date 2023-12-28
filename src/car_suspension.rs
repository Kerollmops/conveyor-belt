use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;
use bevy_rapier3d::prelude::*;

#[derive(Clone, Reflect)]
pub struct WheelInfo {
    pub hit: bool,
    pub entity: Entity,
}

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct CarPhysics {
    pub plane: Vec3,
    pub car_size: Vec3,
    pub wheel_infos: Vec<WheelInfo>,
    pub car_transform_camera: Transform,
    pub wheels_animation_speed: f32,
    pub wheels_stationary_animation_speed: f32,
}

pub fn update_car_suspension(
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut car_query: Query<(&mut CarPhysics, &mut ExternalForce, &mut Velocity, &mut Transform)>,
    mut transform_query: Query<&mut Transform, Without<CarPhysics>>,
) {
    let Ok((mut car_physics, mut force, velocity, car_transform)) = car_query.get_single_mut()
    else {
        return;
    };

    let front_right_direction = car_transform.translation
        + (car_transform.down() * car_physics.car_size.y
            + car_transform.forward() * car_physics.car_size.z)
        + (car_transform.right() * car_physics.car_size.x);

    let front_left_direction = car_transform.translation
        + (car_transform.down() * car_physics.car_size.y
            + car_transform.forward() * car_physics.car_size.z)
        + (car_transform.left() * car_physics.car_size.x);

    let back_right_direction = car_transform.translation
        + (car_transform.down() * car_physics.car_size.y
            + car_transform.back() * car_physics.car_size.z)
        + (car_transform.right() * car_physics.car_size.x);

    let back_left_direction = car_transform.translation
        + (car_transform.down() * car_physics.car_size.y
            + car_transform.back() * car_physics.car_size.z)
        + (car_transform.left() * car_physics.car_size.x);

    let wheels =
        [front_right_direction, front_left_direction, back_right_direction, back_left_direction];

    let max_suspension = 0.3;
    force.force = Vec3::ZERO;
    force.torque = Vec3::ZERO;

    let CarPhysics { wheel_infos, wheels_stationary_animation_speed, .. } = car_physics.as_mut();
    for (i, (wheel, infos)) in wheels.into_iter().zip(wheel_infos).enumerate() {
        let Ok(mut wheel_transform) = transform_query.get_mut(infos.entity) else {
            continue;
        };

        let hit = rapier_context.cast_ray_and_get_normal(
            wheel + car_transform.up() * 0.01,
            car_transform.down(),
            max_suspension,
            true,
            QueryFilter::only_fixed(),
        );

        if let Some((_entity, ray_intersection)) = hit {
            infos.hit = true;
            let compression =
                1. - (ray_intersection.toi * car_transform.down().length() / max_suspension);
            let suspension_strength = 15000.;
            let suspension_damping = 1200.;

            let add_force = ExternalForce::at_point(
                car_transform.up()
                    * ((compression * suspension_strength)
                        - (suspension_damping * (velocity.linvel.y)))
                    * time.delta_seconds(),
                wheel,
                car_transform.translation,
            );

            force.force += add_force.force;
            force.torque += add_force.torque;

            wheel_transform.translation = ray_intersection.point + car_transform.up() * 0.2;
            if i == 2 || i == 3 {
                wheel_transform.rotation = Quat::slerp(
                    wheel_transform.rotation,
                    car_transform.rotation,
                    *wheels_stationary_animation_speed * time.delta_seconds(),
                );
            }
        } else {
            infos.hit = false;

            wheel_transform.translation = wheel - car_transform.up() * (max_suspension - 0.2);
            if i == 2 || i == 3 {
                wheel_transform.rotation = Quat::slerp(
                    wheel_transform.rotation,
                    car_transform.rotation,
                    *wheels_stationary_animation_speed * time.delta_seconds(),
                );
            }
        }
    }
}
