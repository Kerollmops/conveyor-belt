use std::f32::consts::PI;
use std::time::Duration;

use bevy::animation::RepeatAnimation;
use bevy::input::common_conditions::input_toggle_active;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::window::close_on_esc;
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::I)))
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::Next),
        )
        .add_collection_to_loading_state::<_, MyAssets>(GameState::AssetLoading)
        .init_resource::<Animations>()
        .insert_resource(AmbientLight { color: Color::WHITE, brightness: 1.0 })
        // .add_systems(Startup, setup)
        .add_systems(
            OnEnter(GameState::Next),
            (setup_with_assets, setup_cars_transforms.after(setup_with_assets)),
        )
        .add_systems(Update, close_on_esc)
        .add_systems(
            Update,
            (setup_scene_once_loaded, keyboard_animation_control, move_player)
                .run_if(in_state(GameState::Next)),
        )
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    // Cars
    #[asset(path = "cars/models/garbageTruck.glb#Scene0")]
    garbage_truck: Handle<Scene>,
    #[asset(path = "cars/models/police.glb#Scene0")]
    police: Handle<Scene>,
    #[asset(path = "cars/models/sedan.glb#Scene0")]
    sedan: Handle<Scene>,

    // Characters
    #[asset(path = "characters/animations/run.glb#Scene0")]
    character_scene: Handle<Scene>,
    #[asset(path = "characters/animations/run.glb#Animation0")]
    tpose_animation: Handle<AnimationClip>,
    #[asset(path = "characters/animations/run.glb#Animation1")]
    run_animation: Handle<AnimationClip>,
    // Characters Skins
    #[asset(path = "characters/skins/animalBaseH.png")]
    animal_base_h: Handle<Image>,
    #[asset(path = "characters/skins/animalBaseI.png")]
    animal_base_i: Handle<Image>,
    #[asset(path = "characters/skins/cyborg.png")]
    cyborg: Handle<Image>,
    #[asset(path = "characters/skins/militaryFemaleA.png")]
    military_female_a: Handle<Image>,
    #[asset(path = "characters/skins/militaryFemaleB.png")]
    military_female_b: Handle<Image>,
    #[asset(path = "characters/skins/militaryMaleA.png")]
    military_male_a: Handle<Image>,
    #[asset(path = "characters/skins/militaryMaleB.png")]
    military_male_b: Handle<Image>,
    #[asset(path = "characters/skins/robot.png")]
    robot: Handle<Image>,
    #[asset(path = "characters/skins/robot2.png")]
    robot2: Handle<Image>,
    #[asset(path = "characters/skins/robot3.png")]
    robot3: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    AssetLoading,
    AssetPreparation,
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
    // Insert a resource with the current scene information
    commands.insert_resource(Animations(vec![
        assets.tpose_animation.clone(),
        assets.run_animation.clone(),
    ]));

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

    // Character
    commands
        .spawn(PlayerBundle {
            character: CharacterBundle {
                scene: SceneBundle { scene: assets.character_scene.clone(), ..default() },
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // camera
            parent.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0., 5.5, -9.)
                    .looking_at(Vec3::new(0., 3., 10.), Vec3::ZERO),
                ..default()
            });
        });

    // Cars
    commands.spawn(CarBundle {
        scene: SceneBundle {
            scene: assets.garbage_truck.clone(),
            transform: Transform::from_xyz(10., 0., 10.)
                .with_scale(Vec3::splat(4.))
                .with_rotation(Quat::from_rotation_y(PI / 3.)),
            ..default()
        },
        ..default()
    });
    commands.spawn(CarBundle {
        scene: SceneBundle {
            scene: assets.police.clone(),
            transform: Transform::from_xyz(15., 0., -8.)
                .with_scale(Vec3::splat(3.))
                .with_rotation(Quat::from_rotation_y(PI / 6.)),
            ..default()
        },
        ..default()
    });
    commands.spawn(CarBundle {
        scene: SceneBundle {
            scene: assets.sedan.clone(),
            transform: Transform::from_xyz(15., 0., 8.)
                .with_scale(Vec3::splat(3.))
                .with_rotation(Quat::from_rotation_y(PI / 2.)),
            ..default()
        },
        ..default()
    });

    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(30.0).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight { intensity: 1500.0, shadows_enabled: true, ..default() },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

fn setup_cars_transforms(
    q_cars: Query<(Entity, &Children), With<Car>>,
    q_children: Query<&Children>,
    mut q_transforms: Query<&mut Transform>,
) {
    for (entity, children) in &q_cars {
        let mut transform = q_transforms.get_mut(entity).unwrap();
        transform.scale = Vec3::splat(2.5);
        transform.translation = Vec3::new(10., 1.1, 10.);

        for &child in children.iter() {
            let nested_child = q_children.get(child).unwrap();
            for &nested_child in nested_child.iter() {
                let mut transform = q_transforms.get_mut(nested_child).unwrap();
                transform.translation = Vec3::ZERO;
            }
        }
    }
}

#[derive(Default, Resource)]
struct Animations(Vec<Handle<AnimationClip>>);

#[derive(Default, Component)]
struct Character;

#[derive(Default, Bundle)]
struct CharacterBundle {
    marker: Character,
    scene: SceneBundle,
}

#[derive(Default, Component)]
struct Player;

#[derive(Default, Bundle)]
struct PlayerBundle {
    marker: Player,
    character: CharacterBundle,
}

#[derive(Default, Component)]
struct Car;

#[derive(Default, Bundle)]
struct CarBundle {
    marker: Car,
    scene: SceneBundle,
}

fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in &mut players {
        player.play(animations.0[0].clone_weak()).repeat();
    }
}

fn move_player(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut q_player: Query<&mut Transform, With<Player>>,
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

/// Animation controls:
///   - spacebar: play / pause
///   - arrow up / down: speed up / slow down animation playback
///   - arrow left / right: seek backward / forward
///   - digit 1 / 3 / 5: play the animation <digit> times
///   - L: loop the animation forever
///   - return: change animation
fn keyboard_animation_control(
    keyboard_input: Res<Input<KeyCode>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
) {
    for mut player in &mut animation_players {
        if keyboard_input.just_pressed(KeyCode::Space) {
            if player.is_paused() {
                player.resume();
            } else {
                player.pause();
            }
        }

        if keyboard_input.just_pressed(KeyCode::Return) {
            *current_animation = (*current_animation + 1) % animations.0.len();
            player
                .play_with_transition(
                    animations.0[*current_animation].clone_weak(),
                    Duration::from_millis(250),
                )
                .repeat();
        }

        if keyboard_input.just_pressed(KeyCode::Key1) {
            player.set_repeat(RepeatAnimation::Count(1));
            player.replay();
        }

        if keyboard_input.just_pressed(KeyCode::Key3) {
            player.set_repeat(RepeatAnimation::Count(3));
            player.replay();
        }

        if keyboard_input.just_pressed(KeyCode::Key5) {
            player.set_repeat(RepeatAnimation::Count(5));
            player.replay();
        }

        if keyboard_input.just_pressed(KeyCode::L) {
            player.set_repeat(RepeatAnimation::Forever);
        }
    }
}
