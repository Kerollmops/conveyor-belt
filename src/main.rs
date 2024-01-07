#![allow(clippy::type_complexity)]

use std::iter::once;

use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin};
use bevy::core_pipeline::fxaa::Fxaa;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::render::view::ColorGrading;
use bevy::window::close_on_esc;
use bevy_asset_loader::prelude::*;
use bevy_dolly::dolly_type::Rig;
use bevy_dolly::system::Dolly;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_scene_hook::{HookPlugin, HookedSceneBundle, SceneHook};
use bevy_xpbd_3d::prelude::*;
use car_acceleration::car_acceleration;
use car_steering::update_car_steering;
use car_suspension::{update_car_suspension, CarPhysics};
use car_wheel_control::{
    update_car_wheel_control, update_car_wheel_rotation_speed, update_car_wheels,
};

mod car_acceleration;
mod car_steering;
mod car_suspension;
mod car_wheel_control;

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_plugins((
            DefaultPlugins,
            TemporalAntiAliasPlugin,
            HookPlugin,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::I)),
        ))
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Next)
                .load_collection::<MyAssets>(),
        )
        .insert_resource(Time::new_with(Physics::fixed_hz(144.0)))
        .insert_resource(PhysicsDebugConfig {
            enabled: false,
            raycast_normal_color: None,
            ..default()
        })
        .insert_resource(Msaa::Off)
        .insert_resource(AmbientLight { brightness: 0.0, ..default() })
        .register_type::<CarPhysics>()
        .add_systems(OnEnter(GameState::Next), (setup_with_assets, setup_map))
        .add_systems(Update, close_on_esc)
        .add_systems(
            Update,
            (
                Dolly::<MainCamera>::update_active,
                update_camera,
                update_car_suspension,
                update_car_steering,
                car_acceleration,
                update_car_wheel_rotation_speed,
                update_car_wheel_control.after(update_car_wheel_rotation_speed),
                update_car_wheels.after(update_car_wheel_control),
                text_kmh_update_system,
            )
                .run_if(in_state(GameState::Next)),
        )
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "cars/models/porsche_911_930_turbo.glb#Scene0")]
    porsche: Handle<Scene>,
    // #[asset(path = "cars/models/chassis.glb#Mesh0/Primitive0")]
    // chassis: Handle<Mesh>,
    #[asset(path = "maps/playground.glb#Scene0")]
    playground: Handle<Mesh>,

    #[asset(path = "environments_maps/diffuse_rgb9e5_zstd.ktx2")]
    diffuse_map: Handle<Image>,
    #[asset(path = "environments_maps/specular_rgb9e5_zstd.ktx2")]
    specular_map: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    AssetLoading,
    Next,
}

/// Associated to a RayCaster to help get the wheel forward direction and other things.
#[derive(Component)]
struct RayCastWheelEntity(pub Entity);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CarWheel {
    FrontRight,
    FrontLeft,
    BackRight,
    BackLeft,
}

#[derive(Component)]
struct MainCamera;

/// set up a simple 3D scene
fn setup_with_assets(mut commands: Commands, assets: Res<MyAssets>) {
    use bevy_dolly::dolly::drivers::*;

    let car_transform = Transform::from_xyz(0.0, 1.6, 0.0);

    // camera
    commands.spawn((
        MainCamera,
        Camera3dBundle {
            transform: Transform::from_xyz(-6.0, 6.0, -6.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera { hdr: true, order: 1, ..default() },
            color_grading: ColorGrading { exposure: 1.0, ..default() },
            tonemapping: Tonemapping::AcesFitted,
            projection: Projection::Perspective(PerspectiveProjection {
                near: 1e-8,
                ..Default::default()
            }),
            ..default()
        },
        Fxaa::default(),
        BloomSettings::default(),
        EnvironmentMapLight {
            diffuse_map: assets.diffuse_map.clone(),
            specular_map: assets.specular_map.clone(),
        },
        TemporalAntiAliasBundle::default(),
        // Already declared by the Rig camera
        // ScreenSpaceAmbientOcclusionBundle::default(),
        // Look at and follow vehicule
        Rig::builder()
            .with(Position::new(car_transform.translation))
            .with(Rotation::new(car_transform.rotation))
            .with(Smooth::new_position(2.5))
            .with(Arm::new(Vec3::new(0.0, 2.5, 8.0)))
            .with(
                LookAt::new(car_transform.translation + Vec3::Y)
                    .tracking_smoothness(1.25)
                    .tracking_predictive(true),
            )
            .build(),
        // // Look at vehicule from a fixed point
        // Rig::builder()
        //     .with(Position::new(Vec3::new(6.0, 6.0, 6.0)))
        //     .with(Rotation::new(Quat::IDENTITY))
        //     .with(Smooth::new_position(1.25).predictive(true))
        //     .with(
        //         LookAt::new(car_transform.translation + Vec3::Y)
        //             .tracking_smoothness(1.25)
        //             .tracking_predictive(true),
        //     )
        //     .build(),
    ));

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 12_000.0,
            color: Color::rgb_u8(255, 255, 233),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    let chassis_size = Vec3::new(0.95, 0.4, 1.3);
    let max_suspension = 0.7;
    commands
        .spawn((
            RigidBody::Dynamic,
            TransformBundle::from(car_transform),
            Collider::cuboid(2.0, 1.0, 4.4),
            // Collider::trimesh_from_mesh(meshes.get(&assets.chassis).unwrap()).unwrap(),
            AngularDamping(3.0),
            Mass(30.0 - 8.8), // there always is 8.8 more ???
            CenterOfMass(Vec3::new(0.0, -0.3, 0.3)),
            CarPhysics {
                chassis_size,
                max_suspension,
                suspension_strength: 450.,
                suspension_damping: 100.,
                front_tire_max_grip_factor: 0.9,
                front_tire_min_grip_factor: 0.4,
                back_tire_max_grip_factor: 0.7,
                back_tire_min_grip_factor: 0.3,
                tire_grip_velocity_multiplier: 5.0,
                tire_mass: 0.7,
                top_speed: 350.0,
                wheel_rotation: 0.5,
                wheel_rotation_speed: 1.5,
            },
        ))
        .with_children(move |parent| {
            let parent_entity = parent.parent_entity();
            // Spawn Car and Identify car wheels and elements
            parent.spawn(HookedSceneBundle {
                scene: SceneBundle {
                    scene: assets.porsche.clone_weak(),
                    transform: Transform::from_xyz(0.0, -1.0, 0.3)
                        .with_scale(Vec3::new(-1.0, 1.0, -1.0)),
                    ..default()
                },
                hook: SceneHook::new(move |entity, commands| {
                    let identity_transform = Transform::IDENTITY;
                    match entity.get::<Name>().map(|t| t.as_str()) {
                        Some("Front-Left-Wheel") => {
                            commands.insert(CarWheel::FrontLeft);
                            let origin = (identity_transform.down() * chassis_size.y
                                + identity_transform.forward() * chassis_size.z)
                                + (identity_transform.left() * chassis_size.x);
                            commands
                                .commands()
                                .spawn((
                                    RayCastWheelEntity(entity.id()),
                                    RayCaster::new(origin, identity_transform.down())
                                        .with_max_time_of_impact(max_suspension)
                                        .with_solidness(true)
                                        .with_max_hits(1)
                                        .with_query_filter(
                                            SpatialQueryFilter::new()
                                                .without_entities(once(parent_entity)),
                                        ),
                                ))
                                .set_parent(parent_entity);
                        }
                        Some("Front-Right-Wheel") => {
                            commands.insert(CarWheel::FrontRight);
                            let origin = (identity_transform.down() * chassis_size.y
                                + identity_transform.forward() * chassis_size.z)
                                + (identity_transform.right() * chassis_size.x);
                            commands
                                .commands()
                                .spawn((
                                    RayCastWheelEntity(entity.id()),
                                    RayCaster::new(origin, identity_transform.down())
                                        .with_max_time_of_impact(max_suspension)
                                        .with_solidness(true)
                                        .with_max_hits(1)
                                        .with_query_filter(
                                            SpatialQueryFilter::new()
                                                .without_entities(once(parent_entity)),
                                        ),
                                ))
                                .set_parent(parent_entity);
                        }
                        Some("Back-Left-Wheel") => {
                            commands.insert(CarWheel::BackLeft);
                            let origin = (identity_transform.down() * chassis_size.y
                                + identity_transform.back() * chassis_size.z)
                                + (identity_transform.left() * chassis_size.x);
                            commands
                                .commands()
                                .spawn((
                                    RayCastWheelEntity(entity.id()),
                                    RayCaster::new(origin, identity_transform.down())
                                        .with_max_time_of_impact(max_suspension)
                                        .with_solidness(true)
                                        .with_max_hits(1)
                                        .with_query_filter(
                                            SpatialQueryFilter::new()
                                                .without_entities(once(parent_entity)),
                                        ),
                                ))
                                .set_parent(parent_entity);
                        }
                        Some("Back-Right-Wheel") => {
                            commands.insert(CarWheel::BackRight);
                            let origin = (identity_transform.down() * chassis_size.y
                                + identity_transform.back() * chassis_size.z)
                                + (identity_transform.right() * chassis_size.x);
                            commands
                                .commands()
                                .spawn((
                                    RayCastWheelEntity(entity.id()),
                                    RayCaster::new(origin, identity_transform.down())
                                        .with_max_time_of_impact(max_suspension)
                                        .with_solidness(true)
                                        .with_max_hits(1)
                                        .with_query_filter(
                                            SpatialQueryFilter::new()
                                                .without_entities(once(parent_entity)),
                                        ),
                                ))
                                .set_parent(parent_entity);
                        }
                        _ => (),
                    }
                }),
            });
        });

    // Current peed of the car in km/h
    commands.spawn((
        TextBundle::from_sections([
            TextSection::from_style(TextStyle {
                font_size: 30.0,
                color: Color::ORANGE_RED,
                ..default()
            }),
            TextSection::new(" km/h", TextStyle { font_size: 30.0, ..default() }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        })
        .with_background_color(Color::DARK_GRAY.with_a(0.8)),
        KmhText,
    ));
}

#[derive(Component)]
struct KmhText;

fn text_kmh_update_system(
    car_q: Query<(&LinearVelocity, &Transform), With<CarPhysics>>,
    mut query: Query<&mut Text, With<KmhText>>,
) {
    let Ok((&LinearVelocity(lin_vel), car_transform)) = car_q.get_single() else { return };
    let car_speed_ms = car_transform.forward().dot(lin_vel).abs();
    let car_speed_kmh = car_speed_ms * 3.6;

    for mut text in &mut query {
        text.sections[0].value = format!("{car_speed_kmh:>3.0}");
    }
}

fn update_camera(mut rig_q: Query<&mut Rig>, car_q: Query<&Transform, With<CarPhysics>>) {
    use bevy_dolly::dolly::drivers::{LookAt, Position, Rotation};

    let mut rig = rig_q.single_mut();
    let transform = car_q.single();
    rig.driver_mut::<Position>().position = transform.translation;
    rig.driver_mut::<Rotation>().rotation = transform.rotation;
    rig.driver_mut::<LookAt>().target = transform.translation + Vec3::Y;
}

fn setup_map(
    mut commands: Commands,
    assets: Res<MyAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let m = meshes.get(&assets.playground);
    let mut map_mesh = m.unwrap().clone();
    Mesh::generate_tangents(&mut map_mesh).unwrap();
    let collider = Collider::trimesh_from_mesh(m.unwrap()).unwrap();

    commands
        .spawn((
            RigidBody::Static,
            PbrBundle {
                transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::new(5., 5., 5.)),
                mesh: meshes.add(map_mesh),
                material: materials.add(Color::ANTIQUE_WHITE.into()),
                ..default()
            },
        ))
        .insert(collider);
}
