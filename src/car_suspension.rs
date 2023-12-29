use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;
use bevy_rapier3d::prelude::*;
use bevy_vector_shapes::painter::{ShapeConfig, ShapePainter};
use bevy_vector_shapes::shapes::LinePainter;

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct CarPhysics {
    pub car_size: Vec3,
    pub car_transform_camera: Transform,
    pub max_suspension: f32,
    pub suspension_strength: f32,
    pub suspension_damping: f32,
    #[inspector(min = 0.0, max = 1.0)]
    pub tire_grip_factor: f32,
    pub tire_mass: f32,
    pub top_speed: f32,
    #[inspector(min = 0.0, max = 1.0)]
    pub wheel_rotation: f32,
    pub wheel_rotation_speed: f32,
}

pub fn update_car_suspension(
    mut painter: ShapePainter,
    rapier_context: Res<RapierContext>,
    mut car_query: Query<(
        &RapierRigidBodyHandle,
        &mut CarPhysics,
        &mut ExternalForce,
        &mut Transform,
    )>,
) {
    let Ok((handle, car_physics, mut car_force, car_transform)) = car_query.get_single_mut() else {
        return;
    };

    let CarPhysics { car_size, max_suspension, suspension_strength, suspension_damping, .. } =
        *car_physics;

    let front_right = car_transform.translation
        + (car_transform.down() * car_size.y + car_transform.forward() * car_size.z)
        + (car_transform.right() * car_size.x);

    let front_left = car_transform.translation
        + (car_transform.down() * car_size.y + car_transform.forward() * car_size.z)
        + (car_transform.left() * car_size.x);

    let back_right = car_transform.translation
        + (car_transform.down() * car_size.y + car_transform.back() * car_size.z)
        + (car_transform.right() * car_size.x);

    let back_left = car_transform.translation
        + (car_transform.down() * car_size.y + car_transform.back() * car_size.z)
        + (car_transform.left() * car_size.x);

    let wheels = [front_right, front_left, back_right, back_left];

    for wheel in wheels {
        let hit = rapier_context.cast_ray_and_get_normal(
            wheel,
            car_transform.down(),
            max_suspension,
            true,
            QueryFilter::only_fixed(),
        );

        // suspension spring force
        match hit {
            Some((_entity, ray_intersection)) => {
                painter.set_config(ShapeConfig { color: Color::GREEN, ..painter.config().clone() });
                painter.line(wheel, ray_intersection.point);

                // World-space direction of the spring force.
                let suspension_dir = car_transform.up();

                // Fetch the rigid body from the rapier world.
                let rigid_body = rapier_context.bodies.get(handle.0).unwrap();

                // World-space velocity of this tire.
                let tire_world_vel = rigid_body.velocity_at_point(&wheel.into()).into();

                // Calculate offset from the raycast.
                let offset = max_suspension - ray_intersection.toi;

                // Calculate velocity along the spring direction
                // note that spring_dir is a unit vector, so this returns
                // the magnitude of tire_world_vec as projected on to spring_dir.
                let vel = suspension_dir.dot(tire_world_vel);

                // Calculate he magnitude of the dampened spring force!
                let force = (offset * suspension_strength) - (vel * suspension_damping);

                // Apply force at the location of this tire, in the direction
                // of the suspension.
                let add_force = ExternalForce::at_point(
                    suspension_dir * force,
                    wheel,
                    car_transform.translation,
                );

                car_force.force += add_force.force;
                car_force.torque += add_force.torque;
            }
            None => {
                painter.reset();
                painter.line(wheel, wheel + car_transform.down() * max_suspension);
            }
        }
    }
}
