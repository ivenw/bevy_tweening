use std::time::Duration;

use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

use bevy_tweening::{lens::*, *};

const PIXELS_PER_METER: f32 = 100.0;
const JUMP_VELOCITY: f32 = 11.0 * PIXELS_PER_METER;
const GRAVITY: f32 = 15.0 * PIXELS_PER_METER;
const PLAYER_SIZE: Vec2 = Vec2::new(1.0 * PIXELS_PER_METER, 1.0 * PIXELS_PER_METER);

#[derive(Component)]
struct Player;

#[derive(Component, PartialEq, Eq)]
enum MovementState {
    Jumping,
    Falling,
    Idle,
}

#[derive(Component)]
struct Velocity(Vec2);

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
    let window = WindowDescriptor {
        title: "User Input".to_string(),
        width: 1400.,
        height: 600.,
        present_mode: bevy::window::PresentMode::Fifo, // vsync
        resizable: false,
        ..default()
    };

    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window,
            ..default()
        }))
        .add_system(bevy::window::close_on_esc)
        .add_plugin(TweeningPlugin)
        .add_plugin(InspectorPlugin::<Options>::new())
        .add_startup_system(setup)
        .add_system(change_movement_state)
        .add_system(apply_gravity)
        .add_system(apply_velocity)
        .add_system(tween_player)
        .run();
}

fn setup(mut commands: Commands, windows: Res<Windows>, asset_server: Res<AssetServer>) {
    let window = windows.get_primary().unwrap();
    let bottom = window.height() / -2.0;

    commands.spawn(Camera2dBundle::default());

    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "Press 'space' to jump",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                font_size: 40.0,
                color: Color::WHITE,
            },
        )
        .with_alignment(TextAlignment {
            vertical: VerticalAlign::Center,
            horizontal: HorizontalAlign::Center,
        }),
        ..Default::default()
    });

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(PLAYER_SIZE),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, bottom + (PLAYER_SIZE.y / 2.0), 0.0),
                ..Default::default()
            },
            ..Default::default()
        },
        Animator::new(Dummy::<Transform>::new()),
        MovementState::Idle,
        Velocity(Vec2::new(0.0, 0.0)),
        Player,
    ));
}

fn change_movement_state(
    keys: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    mut query: Query<(&mut MovementState, &mut Velocity, &Transform), With<Player>>,
) {
    let window = windows.get_primary().unwrap();
    let (mut movement_state, mut velocity, transform) = query.single_mut();

    let bottom = window.height() / -2.0;
    let ground = bottom + (PLAYER_SIZE.y / 2.0);

    match *movement_state {
        MovementState::Idle => {
            if keys.just_pressed(KeyCode::Space) {
                velocity.0.y = JUMP_VELOCITY;
                *movement_state = MovementState::Jumping;
            }
        }
        MovementState::Jumping => {
            if velocity.0.y < 0.0 {
                *movement_state = MovementState::Falling;
            }
        }
        MovementState::Falling => {
            if transform.translation.y <= ground {
                velocity.0.y = 0.0;
                *movement_state = MovementState::Idle;
            }
        }
    }
}

fn apply_gravity(time: Res<Time>, mut query: Query<&mut Velocity>) {
    let mut velocity = query.single_mut();

    velocity.0.y -= GRAVITY * time.delta_seconds();
}

fn apply_velocity(
    time: Res<Time>,
    windows: Res<Windows>,
    mut query: Query<(&mut Transform, &Velocity)>,
) {
    let window = windows.get_primary().unwrap();
    let (mut transform, velocity) = query.single_mut();

    let bottom = window.height() / -2.0;
    let ground = bottom + (PLAYER_SIZE.y / 2.0);
    let translation_change = velocity.0 * time.delta_seconds();

    if transform.translation.y + translation_change.y < ground {
        transform.translation.y = ground;
    } else {
        transform.translation += translation_change.extend(0.0);
    }
}

fn tween_player(
    options: Res<Options>,
    mut query: Query<
        (&mut Animator<Transform>, &MovementState, &Transform),
        (Changed<MovementState>, With<Player>),
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
