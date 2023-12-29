#![allow(clippy::type_complexity)]

use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::close_on_esc;
use bevy_asset_loader::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_scene_hook::{HookPlugin, HookedSceneBundle, SceneHook};
use bevy_vector_shapes::ShapePlugin;
use car_acceleration::car_acceleration;
use car_camera::{camera_follow, CameraFollow};
use car_steering::update_car_steering;
use car_suspension::{update_car_suspension, CarPhysics};
use car_wheel_control::{update_car_wheel_control, update_car_wheels};

mod car_acceleration;
mod car_camera;
mod car_steering;
mod car_suspension;
mod car_wheel_control;

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(InfiniteGridPlugin)
        .add_plugins(HookPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(ShapePlugin::default())
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::I)))
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Next)
                .load_collection::<MyAssets>(),
        )
        .insert_resource(AmbientLight { color: Color::WHITE, brightness: 1.0 })
        .register_type::<CarPhysics>()
        .register_type::<CameraFollow>()
        .add_systems(OnEnter(GameState::Next), (setup_with_assets, setup_map))
        .add_systems(PreUpdate, reset_car_external_forces.run_if(in_state(GameState::Next)))
        .add_systems(Update, close_on_esc)
        .add_systems(
            Update,
            (
                update_car_suspension,
                update_car_steering,
                car_acceleration,
                update_car_wheel_control,
                update_car_wheels.after(update_car_wheel_control),
            )
                .run_if(in_state(GameState::Next)),
        )
        .add_systems(PostUpdate, camera_follow.run_if(in_state(GameState::Next)))
        .run();
}

fn reset_car_external_forces(mut car_query: Query<&mut ExternalForce, With<CarPhysics>>) {
    if let Ok(mut car_force) = car_query.get_single_mut() {
        car_force.force = Vec3::ZERO;
        car_force.torque = Vec3::ZERO;
    }
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "cars/models/porsche_911_930_turbo.glb#Scene0")]
    porsche: Handle<Scene>,
    #[asset(path = "maps/playground.glb#Scene0")]
    playground: Handle<Mesh>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    AssetLoading,
    Next,
}

#[derive(Component)]
enum CarWheel {
    FrontLeft,
    FrontRight,
    BackLeft,
    BackRight,
}

/// set up a simple 3D scene
fn setup_with_assets(mut commands: Commands, assets: Res<MyAssets>) {
    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0., 2.0, -10.)
                .looking_at(Vec3::new(0., 0., 0.), Vec3::ZERO),
            ..default()
        })
        .insert(car_camera::CameraFollow {
            camera_translation_speed: 2.0,
            distance_behind: 5.0,
            fake_transform: Transform::default(),
        });

    commands.spawn(InfiniteGridBundle::default());

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight { intensity: 1500.0, shadows_enabled: true, ..default() },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands
        .spawn((
            Car,
            RigidBody::Dynamic,
            TransformBundle::from(Transform::from_xyz(0.0, 1.6, 0.0)),
            Collider::cuboid(1.0, 0.5, 2.2),
        ))
        .insert(CarPhysics {
            chassis_size: Vec3::new(1.0, 0.4, 1.3),
            max_suspension: 0.6,
            suspension_strength: 350.,
            suspension_damping: 25.,
            front_tire_max_grip_factor: 0.9,
            front_tire_min_grip_factor: 0.2,
            back_tire_max_grip_factor: 0.4,
            back_tire_min_grip_factor: 0.1,
            tire_grip_velocity_multiplier: 5.0,
            tire_mass: 0.5,
            top_speed: 150.0,
            wheel_rotation: 0.5,
            wheel_rotation_speed: 3.0,
        })
        .insert(Velocity { ..default() })
        .insert(ExternalImpulse::default())
        .insert(ExternalForce::default())
        .insert(ColliderMassProperties::Density(2.0))
        .insert(GravityScale(1.))
        .insert(Damping { linear_damping: 0., angular_damping: 3. })
        // .insert(Ccd::enabled())
        // Makes rapier to panic:
        // thread 'Compute Task Pool (0)' panicked at parry3d-0.13.5/src/query/nonlinear_time_of_impact/nonlinear_time_of_impact_support_map_support_map.rs:201:40:
        // internal error: entered unreachable code
        .with_children(|parent| {
            // Spawn Car and Identify car wheels and elements
            parent.spawn(HookedSceneBundle {
                scene: SceneBundle {
                    scene: assets.porsche.clone_weak(),
                    transform: Transform::from_xyz(0.0, -1.0, 0.3)
                        .with_scale(Vec3::new(-1.0, 1.0, -1.0)),
                    ..default()
                },
                hook: SceneHook::new(|entity, commands| {
                    match entity.get::<Name>().map(|t| t.as_str()) {
                        Some("Front-Left-Wheel") => commands.insert(CarWheel::FrontLeft),
                        Some("Front-Right-Wheel") => commands.insert(CarWheel::FrontRight),
                        Some("Back-Left-Wheel") => commands.insert(CarWheel::BackLeft),
                        Some("Back-Right-Wheel") => commands.insert(CarWheel::BackRight),
                        _ => commands,
                    };
                }),
            });
        });
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

    let x_shape = Collider::from_bevy_mesh(m.unwrap(), &ComputedColliderShape::TriMesh).unwrap();
    if Collider::from_bevy_mesh(m.unwrap(), &ComputedColliderShape::TriMesh).is_none() {
        println!("the mesh failed to load");
    }

    commands
        .spawn((
            RigidBody::Fixed,
            PbrBundle {
                transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::new(5., 5., 5.)),
                mesh: meshes.add(map_mesh),
                material: materials.add(Color::ANTIQUE_WHITE.into()),
                ..default()
            },
        ))
        .insert(x_shape);
}

#[derive(Default, Component)]
struct Car;

fn looking_at_car(
    mut camera_q: Query<&mut Transform, With<Camera>>,
    car_q: Query<&mut GlobalTransform, With<Car>>,
) {
    let car_transform = car_q.get_single().unwrap();
    for mut transform in camera_q.iter_mut() {
        transform.look_at(car_transform.translation(), Vec3::ZERO);
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}
