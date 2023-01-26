use std::time::Duration;

use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

use bevy_tweening::{lens::*, *};
use interpolation::Ease;

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
#[derive(Inspectable, Resource)]
struct Options {
    jump_duration: u64,
    fall_duration: u64,
    landing_duration: u64,
    jump_ease: String,
    fall_ease: String,
    landing_ease: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            jump_duration: 300,
            fall_duration: 500,
            landing_duration: 100,
            jump_ease: "BackOut".to_string(),
            fall_ease: "CubicIn".to_string(),
            landing_ease: "CubicOut".to_string(),
        }
    }
}

fn string_to_ease_function(string: &String) -> EaseFunction {
    match string.as_str() {
        "QuadraticIn" => EaseFunction::QuadraticIn,
        "QuadraticOut" => EaseFunction::QuadraticOut,
        "QuadraticInOut" => EaseFunction::QuadraticInOut,
        "CubicIn" => EaseFunction::CubicIn,
        "CubicOut" => EaseFunction::CubicOut,
        "CubicInOut" => EaseFunction::CubicInOut,
        "QuarticIn" => EaseFunction::QuarticIn,
        "QuarticOut" => EaseFunction::QuarticOut,
        "QuarticInOut" => EaseFunction::QuarticInOut,
        "QuinticIn" => EaseFunction::QuinticIn,
        "QuinticOut" => EaseFunction::QuinticOut,
        "QuinticInOut" => EaseFunction::QuinticInOut,
        "SineIn" => EaseFunction::SineIn,
        "SineOut" => EaseFunction::SineOut,
        "SineInOut" => EaseFunction::SineInOut,
        "CircularIn" => EaseFunction::CircularIn,
        "CircularOut" => EaseFunction::CircularOut,
        "CircularInOut" => EaseFunction::CircularInOut,
        "ExponentialIn" => EaseFunction::ExponentialIn,
        "ExponentialOut" => EaseFunction::ExponentialOut,
        "ExponentialInOut" => EaseFunction::ExponentialInOut,
        "ElasticIn" => EaseFunction::ElasticIn,
        "ElasticOut" => EaseFunction::ElasticOut,
        "ElasticInOut" => EaseFunction::ElasticInOut,
        "BackIn" => EaseFunction::BackIn,
        "BackOut" => EaseFunction::BackOut,
        "BackInOut" => EaseFunction::BackInOut,
        "BounceIn" => EaseFunction::BounceIn,
        "BounceOut" => EaseFunction::BounceOut,
        "BounceInOut" => EaseFunction::BounceInOut,
        _ => EaseFunction::CubicInOut,
    }
}

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "User Input".to_string(),
                width: 1400.,
                height: 600.,
                // scale_factor_override: Some(0.3), // only here for sneaky testing
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
    if physics.velocity.y < 0.0 && *movement_state != MovementState::Falling {
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
fn tween_jump_and_fall(
    options: Res<Options>,
    mut query: Query<
        (&mut Animator<Transform>, &MovementState, &Transform),
        Changed<MovementState>,
    >,
) {
    if query.is_empty() {
        return;
    }
    let (mut animator, movement_state, transform) = query.single_mut();

    let rest_scale = Vec3::new(1.0, 1.0, 0.0);
    let jump_scale = Vec3::new(0.9, 1.1, 0.0);
    let fall_scale = Vec3::new(0.7, 1.3, 0.0);
    let landing_scale = Vec3::new(1.2, 0.8, 0.0);

    match *movement_state {
        MovementState::Jumping => {
            let tween = Tween::new(
                string_to_ease_function(&options.jump_ease),
                Duration::from_millis(options.jump_duration),
                TransformScaleLens {
                    start: rest_scale,
                    end: jump_scale,
                },
            );
            animator.set_tweenable(tween);
        }
        MovementState::Falling => {
            let tween = Tween::new(
                string_to_ease_function(&options.fall_ease),
                Duration::from_millis(options.fall_duration),
                TransformScaleLens {
                    start: jump_scale,
                    end: fall_scale,
                },
            );
            animator.set_tweenable(tween);
        }
        MovementState::Idle => {
            let tween = Tween::new(
                string_to_ease_function(&options.landing_ease),
                Duration::from_millis(options.landing_duration),
                TransformScaleLens {
                    start: fall_scale,
                    end: landing_scale,
                },
            )
            .then(Tween::new(
                string_to_ease_function(&options.landing_ease),
                Duration::from_millis(options.landing_duration),
                TransformScaleLens {
                    start: landing_scale,
                    end: rest_scale,
                },
            ));
            animator.set_tweenable(tween);
        }
    }
}
