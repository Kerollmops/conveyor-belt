use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;
use bevy_rapier3d::prelude::*;

use crate::car_suspension::CarPhysics;

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct CarController {
    pub rotated_last_frame: bool,
    pub rotate_speed: f32,
    pub speed: f32,
    pub center_of_mass_altered: bool,
    pub rotate_to_rotation: Quat,
    pub slerp_speed: f32,
    pub car_linear_damping: f32,
}

pub fn car_controls(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut car_query: Query<(
        Entity,
        &mut Damping,
        &mut CarController,
        &mut CarPhysics,
        &mut ExternalForce,
        &mut ExternalImpulse,
        &mut Transform,
    )>,
    mut transform_query: Query<&mut Transform, Without<CarPhysics>>,
) {
    let Ok((
        entity,
        mut damping,
        mut car_controller,
        mut car_physics,
        mut force,
        mut impulse,
        car_transform,
    )) = car_query.get_single_mut()
    else {
        return;
    };

    if keys.just_pressed(KeyCode::Space) {
        let new_impluse = ExternalImpulse::at_point(
            Vec3::new(5., 50., 0.),
            car_transform.translation,
            car_transform.translation,
        );
        impulse.impulse = new_impluse.impulse;
        impulse.torque_impulse = Vec3::new(50., 0., 0.);
    }

    if !car_controller.center_of_mass_altered {
        commands.entity(entity).insert(AdditionalMassProperties::MassProperties(MassProperties {
            mass: 1.,
            local_center_of_mass: Vec3::new(0., -0.5, 0.),
            ..default()
        }));

        car_controller.center_of_mass_altered = true;
    }

    let num_on_ground = car_physics.wheel_infos.iter().filter(|wi| wi.hit).count();

    if num_on_ground >= 2 {
        damping.linear_damping = car_controller.car_linear_damping;
        if keys.pressed(KeyCode::W) {
            force.force += car_transform.forward() * car_controller.speed * time.delta_seconds();
        }
        if keys.just_pressed(KeyCode::W) {
            force.torque += car_transform.left() * 300.;
        }
        if keys.pressed(KeyCode::S) {
            force.force -= car_transform.forward() * car_controller.speed * time.delta_seconds();
        }
        if keys.just_pressed(KeyCode::S) {
            force.torque -= car_transform.left() * 300.;
        }
        car_controller.rotate_to_rotation = car_transform.rotation;

        car_physics.car_transform_camera.rotation = Quat::slerp(
            car_physics.car_transform_camera.rotation,
            car_controller.rotate_to_rotation,
            car_controller.slerp_speed * time.delta_seconds(),
        );
        if keys.pressed(KeyCode::A) {
            force.torque += car_transform.up() * time.delta_seconds() * car_controller.rotate_speed;
        }
        if keys.pressed(KeyCode::D) {
            force.torque -= car_transform.up() * time.delta_seconds() * car_controller.rotate_speed;
        }
    } else {
        damping.linear_damping = 0.;
    }

    car_physics.car_transform_camera.translation = car_transform.translation;
}
