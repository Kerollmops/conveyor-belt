use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;
use bevy_xpbd_3d::prelude::*;

use crate::RayCastWheelEntity;

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct CarPhysics {
    pub chassis_size: Vec3,
    pub max_suspension: f32,
    pub suspension_strength: f32,
    pub suspension_damping: f32,

    #[inspector(min = 0.0, max = 1.0)]
    pub front_tire_max_grip_factor: f32,
    #[inspector(min = 0.0, max = 1.0)]
    pub front_tire_min_grip_factor: f32,

    #[inspector(min = 0.0, max = 1.0)]
    pub back_tire_max_grip_factor: f32,
    #[inspector(min = 0.0, max = 1.0)]
    pub back_tire_min_grip_factor: f32,

    pub tire_grip_velocity_multiplier: f32,

    pub tire_mass: f32,
    pub top_speed: f32,
    #[inspector(min = 0.0, max = 1.0)]
    pub wheel_rotation: f32,
    pub wheel_rotation_speed: f32,
}

pub fn update_car_suspension(
    mut car_query: Query<(
        &LinearVelocity,
        &AngularVelocity,
        &mut CarPhysics,
        &mut ExternalForce,
        &Transform,
        &CenterOfMass,
    )>,
    raycast_query: Query<(&RayCastWheelEntity, &RayCaster, &RayHits)>,
) {
    let Ok((
        &LinearVelocity(lin_vel),
        &AngularVelocity(ang_vel),
        car_physics,
        mut external_force,
        &car_transform,
        &CenterOfMass(car_center_of_mass),
    )) = car_query.get_single_mut()
    else {
        return;
    };

    let CarPhysics { max_suspension, suspension_strength, suspension_damping, .. } = *car_physics;

    for (_, ray, hits) in &raycast_query {
        assert!(hits.len() <= 1);
        let hit = hits.as_slice().get(0);

        // suspension spring force
        if let Some(RayHitData { time_of_impact, .. }) = hit {
            // World-space direction of the spring force.
            let suspension_dir = car_transform.up();

            // World-space velocity of this tire.
            let tire_world_vel = lin_vel + ang_vel.cross(car_transform.rotation * ray.origin);

            // Calculate offset from the raycast.
            let offset = max_suspension - time_of_impact;

            // Calculate velocity along the spring direction
            // note that spring_dir is a unit vector, so this returns
            // the magnitude of tire_world_vec as projected on to spring_dir.
            let vel = suspension_dir.dot(tire_world_vel);

            // Calculate he magnitude of the dampened spring force!
            let force = (offset * suspension_strength) - (vel * suspension_damping);

            // Apply force at the location of this tire, in the direction
            // of the suspension.
            external_force.persistent = false;
            external_force.apply_force_at_point(
                suspension_dir * force,
                car_transform.rotation * ray.origin,
                car_center_of_mass,
            );
        }
    }
}
