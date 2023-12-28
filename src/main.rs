#![allow(clippy::type_complexity)]

use std::f32::consts::PI;

use bevy::input::common_conditions::input_toggle_active;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::window::close_on_esc;
use bevy_asset_loader::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(InfiniteGridPlugin)
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
                // get_in_nearest_car,
                // get_out_of_the_car,
            )
                .run_if(in_state(GameState::Next)),
        )
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "cars/models/porsche_911_930_turbo_small.glb#Scene0")]
    porsche: Handle<Scene>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    AssetLoading,
    Next,
}

/// Once the scene is loaded, start the animation
/// set up a simple 3D scene
fn setup_with_assets(
    mut commands: Commands,
    assets: Res<MyAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(InfiniteGridBundle::default());

    // Light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        directional_light: DirectionalLight { shadows_enabled: true, ..default() },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .into(),
        ..default()
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight { intensity: 1500.0, shadows_enabled: true, ..default() },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Cars
    commands
        .spawn((
            Controlled,
            CarBundle {
                scene: SceneBundle {
                    scene: assets.porsche.clone_weak(),
                    transform: Transform::from_xyz(10., 0., 10.)
                        .with_rotation(Quat::from_rotation_y(PI / 3.)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            // camera
            parent.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0., 5.5, -9.)
                    .looking_at(Vec3::new(0., 3., 10.), Vec3::ZERO),
                ..default()
            });
        });

    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(30.0).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
}

#[derive(Default, Component)]
struct Controlled;

#[derive(Default, Component)]
struct Car;

#[derive(Default, Bundle)]
struct CarBundle {
    marker: Car,
    scene: SceneBundle,
}

fn move_controlled(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut q_player: Query<&mut Transform, With<Controlled>>,
) {
    let mut transform = q_player.single_mut();

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

// fn get_in_nearest_car(
//     mut commands: Commands,
//     keyboard_input: Res<Input<KeyCode>>,
//     q_controlled: Query<(Entity, &Transform), (With<Controlled>, With<Character>)>,
//     q_camera: Query<Entity, With<Camera>>,
//     q_cars: Query<(Entity, &Transform), With<Car>>,
// ) {
//     if keyboard_input.just_pressed(KeyCode::E) {
//         if let Ok((controlled_entity, current_trans)) = q_controlled.get_single() {
//             if let Some((car_entity, _)) = q_cars.iter().min_by_key(|(_, car_trans)| {
//                 OrderedFloat(car_trans.translation.distance_squared(current_trans.translation))
//             }) {
//                 let cam_entity = q_camera.single();
//                 commands.entity(cam_entity).remove_parent();
//                 commands.entity(car_entity).add_child(cam_entity);
//                 commands.entity(controlled_entity).despawn_recursive();
//                 commands.entity(car_entity).insert(Controlled);
//             }
//         }
//     }
// }

// fn get_out_of_the_car(
//     mut commands: Commands,
//     assets: Res<MyAssets>,
//     keyboard_input: Res<Input<KeyCode>>,
//     q_controlled: Query<(Entity, &Transform), (With<Controlled>, With<Car>)>,
//     q_camera: Query<Entity, With<Camera>>,
// ) {
//     if keyboard_input.just_pressed(KeyCode::E) {
//         if let Ok((controlled_entity, &controlled_trans)) = q_controlled.get_single() {
//             let cam_entity = q_camera.single();
//             commands.entity(cam_entity).remove_parent();
//             commands.entity(controlled_entity).remove::<Controlled>();
//             commands
//                 .spawn(CharacterBundle {
//                     scene: SceneBundle {
//                         scene: assets.character_scene.clone_weak(),
//                         transform: Transform {
//                             translation: controlled_trans.translation
//                                 + (controlled_trans.right() * 4.0),
//                             ..controlled_trans
//                         },
//                         ..default()
//                     },
//                     ..default()
//                 })
//                 .insert(Controlled)
//                 .add_child(cam_entity);
//         }
//     }
// }
