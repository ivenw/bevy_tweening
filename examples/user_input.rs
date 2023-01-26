use std::time::Duration;

use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

use bevy_tweening::{lens::*, *};

#[derive(Component)]
struct Player;

#[derive(Component, PartialEq, Eq)]
enum MovementState {
    Jumping,
    Falling,
    Idle,
}

#[derive(Component)]
struct Physics {
    velocity: Vec2,
}

// TODO adopt this for setting the tween parameters of the jump and fall
#[derive(Copy, Clone, PartialEq, Inspectable, Resource)]
struct Options {
    duration: u64,
}

impl Default for Options {
    fn default() -> Self {
        Self { duration: 100 }
    }
}

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "User Input".to_string(),
                width: 1400.,
                height: 600.,
                scale_factor_override: Some(0.3), // only here for sneaky testing
                present_mode: bevy::window::PresentMode::Fifo, // vsync
                ..default()
            },
            ..default()
        }))
        .add_system(bevy::window::close_on_esc)
        .add_plugin(TweeningPlugin)
        .add_plugin(InspectorPlugin::<Options>::new())
        .add_startup_system(setup)
        .add_system(take_input)
        .add_system(apply_gravity)
        .add_system(move_player)
        .add_system(tween_jump_and_fall)
        .run();
}

fn setup(mut commands: Commands, windows: Res<Windows>) {
    let window = windows.get_primary().unwrap();
    let bottom = window.height() / -2.0;

    let player_size = Vec2::new(100.0, 100.0);

    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(player_size),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, bottom + (player_size.y / 2.0), 0.0),
                ..Default::default()
            },
            ..Default::default()
        },
        Animator::new(Dummy::<Transform>::new()),
        MovementState::Idle,
        Physics {
            velocity: Vec2::new(0.0, 0.0),
        },
        Player,
    ));
}

// This is just a simple character controller for demonstration purposes.
// works but should protably be refactored a bit
fn take_input(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut MovementState, &mut Physics)>,
) {
    let (mut movement_state, mut physics) = query.single_mut();

    match *movement_state {
        MovementState::Idle => {
            if keys.just_pressed(KeyCode::Space) {
                *movement_state = MovementState::Jumping;
                physics.velocity.y = 1_000.0;
            }
        }
        _ => {}
    }
}

fn apply_gravity(time: Res<Time>, mut query: Query<(&mut Physics, &mut MovementState)>) {
    let (mut physics, mut movement_state) = query.single_mut();

    if *movement_state == MovementState::Jumping || *movement_state == MovementState::Falling {
        physics.velocity.y -= 1_500.0 * time.delta_seconds();
    }
    if physics.velocity.y < 0.0 {
        *movement_state = MovementState::Falling;
    }
}

fn move_player(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Physics, &mut MovementState, &Sprite)>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();
    let (mut transform, mut physics, mut movement_state, sprite) = query.single_mut();

    let bottom = window.height() / -2.0;
    let player_height = sprite.custom_size.unwrap().y;
    let ground_relative_to_player = bottom + (player_height / 2.0);
    let translation_change = physics.velocity * time.delta_seconds();

    if transform.translation.y + translation_change.y < ground_relative_to_player {
        transform.translation.y = ground_relative_to_player;
        physics.velocity.y = 0.0;
        *movement_state = MovementState::Idle;
    } else {
        transform.translation += translation_change.extend(0.0);
    }
}

// This is the actual demonstation of the tweening plugin
// Doesn't quite work yet how it should
fn tween_jump_and_fall(
    options: Res<Options>,
    mut query: Query<(&mut Animator<Transform>, &MovementState), Changed<MovementState>>,
) {
    if query.is_empty() {
        return;
    }
    let (mut animator, movement_state) = query.single_mut();

    // if !animator.is_completed() {
    //     return;
    // }

    if *movement_state == MovementState::Jumping {
        let tween = Tween::new(
            EaseFunction::CubicInOut,
            Duration::from_millis(options.duration),
            TransformScaleLens {
                start: Vec3::new(1.0, 1.0, 0.0),
                end: Vec3::new(0.8, 2.0, 0.0),
            },
        );
        animator.set_tweenable(tween);

    // } else if *movement_state == MovementState::Falling {
    //     let tween = Tween::new(
    //         EaseFunction::CubicInOut,
    //         Duration::from_millis(100),
    //         TransformScaleLens {
    //             start: Vec3::new(1.0, 2.0, 0.0),
    //             end: Vec3::new(1.0, 1.0, 0.0),
    //         },
    //     );
    //     animator.set_tweenable(tween);
    } else if *movement_state == MovementState::Idle {
        let tween = Tween::new(
            EaseFunction::BackOut,
            Duration::from_millis(options.duration),
            TransformScaleLens {
                start: Vec3::new(1.0, 1.0, 0.0),
                end: Vec3::new(1.5, 0.8, 0.0),
            },
        )
        .then(Tween::new(
            EaseFunction::BackOut,
            Duration::from_millis(options.duration),
            TransformScaleLens {
                start: Vec3::new(1.5, 0.8, 0.0),
                end: Vec3::new(1.0, 1.0, 0.0),
            },
        ));
        animator.set_tweenable(tween);
    }
}
