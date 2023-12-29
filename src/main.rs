#![allow(clippy::type_complexity)]

use std::iter;

use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::close_on_esc;
use bevy_asset_loader::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_scene_hook::HookPlugin;
use bevy_vector_shapes::ShapePlugin;
use car_camera::CameraFollow;
use car_controls::{car_controls, CarController};
use car_suspension::{update_car_suspension, CarPhysics, WheelInfo};

mod car_camera;
mod car_controls;
mod car_suspension;

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
        .register_type::<CarController>()
        .add_systems(OnEnter(GameState::Next), setup_with_assets)
        .add_systems(Update, close_on_esc)
        .add_systems(
            Update,
            (update_car_suspension, looking_at_car, car_controls.after(update_car_suspension))
                .run_if(in_state(GameState::Next)),
        )
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "cars/models/porsche_911_930_turbo.glb#Scene0")]
    porsche: Handle<Scene>,
    // #[asset(path = "maps/playground.glb#Scene0")]
    // playground: Handle<Scene>,
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
fn setup_with_assets(
    mut commands: Commands,
    assets: Res<MyAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0., 10.0, -10.)
                .looking_at(Vec3::new(0., 0., 0.), Vec3::ZERO),
            ..default()
        })
        .insert(CameraFollow {
            camera_translation_speed: 1000.,
            fake_transform: Transform::from_xyz(0., 0., 0.),
            distance_behind: 10.,
        });

    commands.spawn(InfiniteGridBundle::default());

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight { intensity: 1500.0, shadows_enabled: true, ..default() },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    let car_size = Vec3::new(1.0, 0.5, 2.2);
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });
    let cylinder = meshes.add(shape::Cylinder { radius: 1.0, height: 1.0, ..default() }.into());
    let wheel_infos = iter::repeat_with(|| {
        let entity = commands
            .spawn(PbrBundle {
                mesh: cylinder.clone_weak(),
                material: debug_material.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            })
            .id();
        WheelInfo { entity, hit: false }
    })
    .take(4)
    .collect();

    commands
        .spawn((
            Car,
            RigidBody::Dynamic,
            TransformBundle::from(Transform::from_xyz(0.0, 1.6, 0.0)),
            Collider::cuboid(car_size.x, car_size.y, car_size.z),
        ))
        .insert(CarPhysics {
            wheel_infos,
            car_size,
            car_transform_camera: Transform::from_xyz(0., 0., 0.),
            max_suspension: 0.6,
            suspension_strength: 250.,
            suspension_damping: 120.,
        })
        .insert(CarController {
            car_linear_damping: 0.5,
            rotate_to_rotation: Quat::IDENTITY,
            slerp_speed: 5.,
            rotated_last_frame: false,
            center_of_mass_altered: false,
            speed: 5000.,
            rotate_speed: 5200.,
        })
        .insert(Velocity { ..default() })
        .insert(ExternalImpulse::default())
        .insert(ExternalForce::default())
        .insert(GravityScale(1.))
        .insert(Damping { linear_damping: 0., angular_damping: 3. })
        .insert(Ccd::enabled());
    // .with_children(|parent| {
    //     // Spawn Car and Identify car wheels and elements
    //     parent.spawn(HookedSceneBundle {
    //         scene: SceneBundle {
    //             scene: assets.porsche.clone_weak(),
    //             transform: Transform::from_xyz(0.0, -0.9, -0.3),
    //             ..default()
    //         },
    //         hook: SceneHook::new(|entity, commands| {
    //             match entity.get::<Name>().map(|t| t.as_str()) {
    //                 Some("Front-Left-Wheel") => commands.insert(CarWheel::FrontLeft),
    //                 Some("Front-Right-Wheel") => commands.insert(CarWheel::FrontRight),
    //                 Some("Back-Left-Wheel") => commands.insert(CarWheel::BackLeft),
    //                 Some("Back-Right-Wheel") => commands.insert(CarWheel::BackRight),
    //                 _ => commands,
    //             };
    //         }),
    //     });
    // });

    // square base
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(300.0, 0.1, 300.0),
        PbrBundle {
            mesh: meshes.add(shape::Quad { size: Vec2::splat(300.0), flip: false }.into()),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
    ));
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
