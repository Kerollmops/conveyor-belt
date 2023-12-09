use std::f32::consts::PI;
use std::time::Duration;

use bevy::animation::RepeatAnimation;
use bevy::input::common_conditions::input_toggle_active;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::window::close_on_esc;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::I)))
        .insert_resource(AmbientLight { color: Color::WHITE, brightness: 1.0 })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                close_on_esc,
                reset_cars_transform,
                setup_scene_once_loaded,
                keyboard_animation_control,
                track_character,
            ),
        )
        .run();
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip>>);

#[derive(Component)]
struct Character;

#[derive(Bundle)]
struct CharacterBundle {
    marker: Character,
    scene: SceneBundle,
}

#[derive(Component)]
struct Car;

#[derive(Bundle)]
struct CarBundle {
    marker: Car,
    scene: SceneBundle,
}

/// Once the scene is loaded, start the animation
/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Insert a resource with the current scene information
    commands.insert_resource(Animations(vec![
        asset_server.load("Animated Characters bundle/Animations/run.glb#Animation0"),
        asset_server.load("Animated Characters bundle/Animations/run.glb#Animation1"),
    ]));

    // // Camera
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(100.0, 100.0, 150.0)
    //         .looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
    //     ..default()
    // });

    // // Plane
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Plane::from_size(500000.0).into()),
    //     material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
    //     ..default()
    // });

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
    commands.spawn(CharacterBundle {
        marker: Character,
        scene: SceneBundle {
            scene: asset_server.load("Animated Characters bundle/Animations/run.glb#Scene0"),
            // transform: Transform::from_xyz(-4.0994653, -0.21640539, -0.54153347),
            ..default()
        },
    });

    // Cars
    commands.spawn(CarBundle {
        marker: Car,
        scene: SceneBundle {
            scene: asset_server.load("Mini Car Kit/Models/glTF format/carAmbulance.gltf#Scene0"),
            // transform: Transform::from_xyz(-4.0994653, -0.21640539, -0.54153347),
            ..default()
        },
    });
    commands.spawn(CarBundle {
        marker: Car,
        scene: SceneBundle {
            scene: asset_server.load("Mini Car Kit/Models/glTF format/carDelivery.gltf#Scene0"),
            // transform: Transform::from_xyz(-4.0994653, -0.21640539 + 1.2, -0.54153347),
            ..default()
        },
    });
    commands.spawn(CarBundle {
        marker: Car,
        scene: SceneBundle {
            scene: asset_server.load("Mini Car Kit/Models/glTF format/carFormula.gltf#Scene0"),
            // transform: Transform::from_xyz(-4.0994653, -0.21640539 + 1.2, -0.54153347),
            ..default()
        },
    });
    commands.spawn(CarBundle {
        marker: Car,
        scene: SceneBundle {
            scene: asset_server.load("Mini Car Kit/Models/glTF format/carGarbage.gltf#Scene0"),
            // transform: Transform::from_xyz(-4.0994653, -0.21640539 + 1.2, -0.54153347),
            ..default()
        },
    });
    commands.spawn(CarBundle {
        marker: Car,
        scene: SceneBundle {
            scene: asset_server.load("Mini Car Kit/Models/glTF format/carJeep.gltf#Scene0"),
            // transform: Transform::from_xyz(-4.0994653, -0.21640539 + 1.2, -0.54153347),
            ..default()
        },
    });

    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(4.0).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight { intensity: 1500.0, shadows_enabled: true, ..default() },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn reset_cars_transform(
    q_cars: Query<&Children, With<Car>>,
    q_children: Query<&Children>,
    mut q_transforms: Query<&mut Transform>,
) {
    for children in &q_cars {
        for &child in children.iter() {
            let nested_child = q_children.get(child).unwrap();
            for &nested_child in nested_child.iter() {
                let mut transform = q_transforms.get_mut(nested_child).unwrap();
                transform.translation = Vec3::ZERO;
            }
        }
    }
}

fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in &mut players {
        player.play(animations.0[0].clone_weak()).repeat();
    }
}

fn track_character(
    mut q_camera: Query<&mut Transform, With<Camera>>,
    q_character: Query<&GlobalTransform, With<Character>>,
) {
    let mut camera_transform = q_camera.single_mut();
    let q_character = q_character.single();

    camera_transform.look_at(q_character.translation(), Vec3::ZERO);
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

        if keyboard_input.just_pressed(KeyCode::Up) {
            let speed = player.speed();
            player.set_speed(speed * 1.2);
        }

        if keyboard_input.just_pressed(KeyCode::Down) {
            let speed = player.speed();
            player.set_speed(speed * 0.8);
        }

        if keyboard_input.just_pressed(KeyCode::Left) {
            let elapsed = player.seek_time();
            player.seek_to(elapsed - 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::Right) {
            let elapsed = player.seek_time();
            player.seek_to(elapsed + 0.1);
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
