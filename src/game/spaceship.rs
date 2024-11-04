use std::time::Duration;

use bevy::prelude::*;

use crate::asset_loader::ImageHandles;
use crate::control::{ControlMode, ControlOption};
use crate::game::ShootBulletEvent;
use crate::states::{AppState, GameState};
use crate::ui::{
    FULL_WINDOW_SIZE, get_bottom_edge, get_left_edge, get_right_edge, get_spaceship_size, get_top_edge,
    ZIndexMap,
};
use crate::util::Velocity;

#[derive(Component)]
pub struct Spaceship {
    bullet_cd: Option<Timer>,
}

#[derive(Event)]
pub struct SpaceShipMovementEvent(pub SpaceShipMovement);

pub struct SpaceshipPlugin;

impl Plugin for SpaceshipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), setup_spaceship)
            .add_systems(
                Update,
                check_spaceship_position.run_if(in_state(GameState::Ready)),
            )
            .add_systems(
                FixedUpdate,
                (
                    handle_spaceship_keyboard_interaction,
                    (handle_bullet_cooldown, handle_shoot_bullet).chain(),
                )
                    .run_if(in_state(GameState::InPlay)),
            )
            .observe(handle_spaceship_movement);
    }
}

fn setup_spaceship(
    mut commands: Commands,
    image_handles: Res<ImageHandles>,
    windows: Query<&Window>,
) {
    let window = windows.get_single().unwrap();
    commands.spawn((
        Spaceship { bullet_cd: None },
        Velocity { x: 0., y: 5. },
        SpriteBundle {
            texture: image_handles.spaceship.clone(),
            sprite: Sprite {
                custom_size: Some(get_spaceship_size(window.width())),
                ..default()
            },
            transform: Transform::from_xyz(
                0.,
                -window.height() / 1.5,
                ZIndexMap::SpaceShip.value(),
            ),
            ..default()
        },
    ));
}

fn check_spaceship_position(
    mut next_state: ResMut<NextState<GameState>>,
    mut spaceship_query: Query<(&Transform, &mut Velocity), With<Spaceship>>,
    windows: Query<&Window>,
) {
    let window = windows.get_single().unwrap();
    let (transform, mut velocity) = spaceship_query.get_single_mut().unwrap();
    if transform.translation.y >= -window.height() / 2.5 {
        velocity.y = 0.;
        next_state.set(GameState::InPlay);
    }
}

fn handle_spaceship_keyboard_interaction(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    control_option: Res<ControlOption>,
) {
    if control_option.mode != ControlMode::Keyboard {
        return;
    }
    let movement = match (
        keys.pressed(KeyCode::ArrowUp),
        keys.pressed(KeyCode::ArrowDown),
        keys.pressed(KeyCode::ArrowLeft),
        keys.pressed(KeyCode::ArrowRight),
    ) {
        (true, false, true, false) => SpaceShipMovement::UpLeft,
        (true, false, false, true) => SpaceShipMovement::UpRight,
        (false, true, true, false) => SpaceShipMovement::DownLeft,
        (false, true, false, true) => SpaceShipMovement::DownRight,
        (true, false, _, _) => SpaceShipMovement::Up,
        (false, true, _, _) => SpaceShipMovement::Down,
        (_, _, true, false) => SpaceShipMovement::Left,
        (_, _, false, true) => SpaceShipMovement::Right,
        _ => SpaceShipMovement::Rest,
    };
    commands.trigger(SpaceShipMovementEvent(movement))
}

#[derive(Eq, PartialEq)]
pub enum SpaceShipMovement {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
    Rest,
}

pub fn handle_spaceship_movement(
    trigger: Trigger<SpaceShipMovementEvent>,
    mut spaceship_query: Query<(&mut Velocity, &Transform), With<Spaceship>>,
    window_query: Query<&Window>,
) {
    let window = window_query.get_single().unwrap();
    let Ok((mut velocity, transform)) = spaceship_query.get_single_mut() else {
        return;
    };

    let Vec3 { x, y, z: _ } = transform.translation;

    if trigger.event().0 == SpaceShipMovement::Rest {
        velocity.x = 0.;
        velocity.y = 0.;
        return;
    }

    let full_velocity = if window.width() >= FULL_WINDOW_SIZE.x {
        10.
    } else {
        7.
    };
    let half_velocity = if window.width() >= FULL_WINDOW_SIZE.x {
        7.
    } else {
        5.
    };

    velocity.x = match trigger.event().0 {
        SpaceShipMovement::Left if !meet_left_edge(x, window) => -full_velocity,
        SpaceShipMovement::UpLeft | SpaceShipMovement::DownLeft if !meet_left_edge(x, window) => {
            -half_velocity
        }
        SpaceShipMovement::Right if !meet_right_edge(x, window) => full_velocity,
        SpaceShipMovement::UpRight | SpaceShipMovement::DownRight
            if !meet_right_edge(x, window) =>
        {
            half_velocity
        }
        _ => 0.,
    };

    velocity.y = match trigger.event().0 {
        SpaceShipMovement::Up if !meet_top_edge(y, window) => full_velocity,
        SpaceShipMovement::UpLeft | SpaceShipMovement::UpRight if !meet_top_edge(y, window) => {
            half_velocity
        }
        SpaceShipMovement::Down if !meet_bottom_edge(y, window) => -full_velocity,
        SpaceShipMovement::DownLeft | SpaceShipMovement::DownRight
            if !meet_bottom_edge(y, window) =>
        {
            -half_velocity
        }
        _ => 0.,
    };
}

fn meet_top_edge(position: f32, window: &Window) -> bool {
    position >= get_top_edge(window.height(), get_spaceship_size(window.width()).y)
}

fn meet_bottom_edge(position: f32, window: &Window) -> bool {
    position <= get_bottom_edge(window.height(), get_spaceship_size(window.width()).y)
}

fn meet_left_edge(position: f32, window: &Window) -> bool {
    position <= get_left_edge(window.width(), get_spaceship_size(window.width()).x)
}

fn meet_right_edge(position: f32, window: &Window) -> bool {
    position >= get_right_edge(window.width(), get_spaceship_size(window.width()).x)
}

fn handle_shoot_bullet(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut spaceship_query: Query<(&Transform, &mut Spaceship)>,
    control_option: Res<ControlOption>,
) {
    if keys.pressed(KeyCode::Space) || control_option.mode == ControlMode::Hover {
        let (transform, mut spaceship) = spaceship_query.get_single_mut().unwrap();
        let Vec3 { x, y, .. } = transform.translation;
        if spaceship.bullet_cd.is_none() {
            commands.trigger(ShootBulletEvent { x, y });
            spaceship.bullet_cd = Some(Timer::new(Duration::from_millis(100), TimerMode::Once));
        }
    }
}

fn handle_bullet_cooldown(mut spaceship_query: Query<&mut Spaceship>, time: Res<Time>) {
    let mut spaceship = spaceship_query.get_single_mut().unwrap();
    let Some(ref mut timer) = &mut spaceship.bullet_cd else {
        return;
    };
    timer.tick(time.delta());
    if timer.finished() {
        spaceship.bullet_cd = None;
    }
}
