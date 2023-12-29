use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;
use bevy_rapier3d::prelude::*;

use crate::car_suspension::CarPhysics;

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct CameraFollow {
    pub camera_translation_speed: f32,
    pub fake_transform: Transform,
    pub distance_behind: f32,
}

pub fn camera_follow(
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut car_query: Query<&mut Transform, (With<CarPhysics>, Without<CameraFollow>)>,
    mut camera_query: Query<(&mut CameraFollow, &mut Transform), Without<CarPhysics>>,
) {
    let Ok((mut camera_follow, mut camera_transform)) = camera_query.get_single_mut() else {
        return;
    };

    let Ok(car_transform) = car_query.get_single_mut() else { return };

    camera_follow.fake_transform.translation = car_transform.translation
        + (Vec3::new(Transform::default().back().x, 0., Transform::default().back().z)).normalize()
            * camera_follow.distance_behind;

    camera_follow.fake_transform.look_at(car_transform.translation, Vec3::Y);
    camera_follow.fake_transform.translation.y += 3.;

    camera_transform.translation = Vec3::lerp(
        camera_transform.translation,
        camera_follow.fake_transform.translation,
        camera_follow.camera_translation_speed * time.delta_seconds(),
    );
    camera_transform.look_at(car_transform.translation, Vec3::ZERO);

    let hit = rapier_context.cast_ray_and_get_normal(
        camera_transform.translation,
        camera_transform.forward(),
        Vec3::distance(car_transform.translation, camera_transform.translation),
        true,
        QueryFilter::only_fixed(),
    );

    if let Some((_entity, ray_intersection)) = hit {
        camera_transform.translation = ray_intersection.point;
    }
}
