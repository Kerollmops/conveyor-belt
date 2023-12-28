#![allow(clippy::type_complexity)]

use std::f32::consts::PI;

use bevy::input::common_conditions::input_toggle_active;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::window::close_on_esc;
use bevy_asset_loader::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_scene_hook::{HookPlugin, HookedSceneBundle, SceneHook};

// mod ray_cast_vehicle_controller;

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(InfiniteGridPlugin)
        .add_plugins(HookPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::I)))
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Next)
                .load_collection::<MyAssets>(),
        )
        .insert_resource(AmbientLight { color: Color::WHITE, brightness: 1.0 })
        .add_systems(OnEnter(GameState::Next), setup_with_assets)
        .add_systems(Update, close_on_esc)
        .add_systems(
            Update,
            (
                move_controlled,
                move_wheels,
                // get_in_nearest_car,
                // get_out_of_the_car,
            )
                .run_if(in_state(GameState::Next)),
        )
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "cars/models/porsche_911_930_turbo.glb#Scene0")]
    porsche: Handle<Scene>,
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
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 3.5, -9.).looking_at(Vec3::new(0., 0., 0.), Vec3::ZERO),
        ..default()
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
            Controlled,
            Car,
            RigidBody::Dynamic,
            Collider::cuboid(1.5, 0.5, 2.5),
            Restitution::coefficient(0.7),
            TransformBundle::from(
                Transform::from_xyz(0., 2., 0.)
                    .with_rotation(Quat::from_rotation_y(5.0 * PI / 6.0)),
            ),
        ))
        .with_children(|parent| {
            // Spawn Car and Identify car wheels and elements
            parent.spawn(HookedSceneBundle {
                scene: SceneBundle {
                    scene: assets.porsche.clone_weak(),
                    transform: Transform::from_xyz(0.0, -0.7, 0.0),
                    ..Default::default()
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

    // circular base
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Circle::new(30.0).into()),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_rotation(Quat::from_rotation_x(
                -std::f32::consts::FRAC_PI_2,
            )),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
        .insert(Collider::cuboid(20.0, 0.1, 20.0));
}

#[derive(Default, Component)]
struct Controlled;

#[derive(Default, Component)]
struct Car;

fn move_controlled(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut q_controlled: Query<&mut Transform, With<Controlled>>,
) {
    let mut transform = q_controlled.single_mut();

    if keyboard_input.pressed(KeyCode::Up) {
        let direction = transform.local_z();
        transform.translation += direction * 5.0 * time.delta_seconds();
        // transform.translation += Vec3::new(0., 0., 2. * time.delta_seconds());
    }

    if keyboard_input.pressed(KeyCode::Down) {
        let direction = transform.local_z();
        transform.translation += direction * -5.0 * time.delta_seconds();
        // transform.translation += Vec3::new(0., 0., -2. * time.delta_seconds());
    }

    if keyboard_input.pressed(KeyCode::Left) {
        transform.rotate_y(PI * time.delta_seconds());
    }

    if keyboard_input.pressed(KeyCode::Right) {
        transform.rotate_y(-PI * time.delta_seconds());
    }
}

fn move_wheels(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut q_wheels: Query<(&mut Transform, &CarWheel)>,
) {
    // Clamp the wheel rotation between π/4 and 7π/4
    fn rotation_clamp(f: f32) -> f32 {
        let f = if f < 0.0 { f + 2.0 * PI } else { f };
        if f > PI / 4.0 && f < PI {
            PI / 4.0
        } else if f > PI && f < (7.0 * PI / 4.0) {
            7.0 * PI / 4.0
        } else {
            f
        }
    }

    for (mut transform, wheel) in q_wheels.iter_mut() {
        if matches!(wheel, CarWheel::FrontLeft | CarWheel::FrontRight) {
            let rotation_y = transform.rotation.to_scaled_axis().y;
            if keyboard_input.pressed(KeyCode::Left) {
                let rotation_y = rotation_y + PI * time.delta_seconds();
                transform.rotation = Quat::from_rotation_y(rotation_clamp(rotation_y));
            }
            if keyboard_input.pressed(KeyCode::Right) {
                let rotation_y = rotation_y - PI * time.delta_seconds();
                transform.rotation = Quat::from_rotation_y(rotation_clamp(rotation_y));
            }
        }
    }
}
