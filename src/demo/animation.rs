//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use bevy::prelude::*;
use rand::prelude::*;
use std::time::Duration;

use crate::{
    AppSystems, PausableSystems,
    audio::sound_effect,
    demo::{
        movement::MovementController,
        player::{Player, PlayerAssets},
    },
};

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            (
                update_animation_movement,
                update_animation_atlas,
                trigger_step_sound_effect,
            )
                .chain()
                .in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}

/// Update the animation timer.
fn update_animation_timer(time: Res<Time>, mut query: Query<&mut MovementAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Update the sprite direction and animation state (idling/walking).
fn update_animation_movement(
    mut player_query: Query<(&MovementController, &mut Sprite, &mut MovementAnimation)>,
) {
    for (controller, mut sprite, mut animation) in &mut player_query {
        sprite.flip_x = !controller.facing_right;

        let animation_state = if controller.gliding {
            MovementAnimationState::Gliding
        } else if controller.horizontal == 0.0 || !controller.grounded {
            MovementAnimationState::Idling
        } else {
            MovementAnimationState::Walking
        };
        animation.update_state(animation_state);
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_atlas(mut query: Query<(&MovementAnimation, &mut Sprite)>) {
    for (animation, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.changed() {
            atlas.index = animation.get_atlas_index();
        }
    }
}

/// If the player is moving, play a step sound effect synchronized with the
/// animation.
fn trigger_step_sound_effect(
    mut commands: Commands,
    player_assets: If<Res<PlayerAssets>>,
    mut step_query: Query<(&MovementAnimation, Option<&Player>)>,
) {
    for (animation, player) in &mut step_query {
        if animation.state == MovementAnimationState::Gliding {
            // Wind sound?
        }
        if animation.state == MovementAnimationState::Walking
            && player.is_some()
            && animation.changed()
            && (animation.frame == 0)
        {
            let rng = &mut rand::rng();
            let random_step = player_assets.steps.choose(rng).unwrap().clone();
            commands.spawn(sound_effect(random_step));
        }
    }
}

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementAnimation {
    timer: Timer,
    frame: usize,
    state: MovementAnimationState,
}

#[derive(Reflect, PartialEq)]
pub enum MovementAnimationState {
    Idling,
    Walking,
    Gliding,
}

impl MovementAnimation {
    /// The number of idle frames.
    const IDLE_FRAMES: usize = 1;
    /// The number of walking frames.
    const WALKING_FRAMES: usize = 4;
    /// The duration of each walking frame.
    const WALKING_INTERVAL: Duration = Duration::from_millis(50);
    /// Number of gliding frames
    const GLIDING_FRAMES: usize = 1;

    fn idling() -> Self {
        Self {
            timer: Timer::from_seconds(0.0, TimerMode::Repeating),
            frame: 0,
            state: MovementAnimationState::Idling,
        }
    }

    fn walking() -> Self {
        Self {
            timer: Timer::new(Self::WALKING_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: MovementAnimationState::Walking,
        }
    }

    fn gliding() -> Self {
        Self {
            timer: Timer::from_seconds(0.0, TimerMode::Repeating),
            frame: 0,
            state: MovementAnimationState::Gliding,
        }
    }

    pub fn new() -> Self {
        Self::idling()
    }

    /// Update animation timers.
    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.is_finished() {
            return;
        }
        self.frame = (self.frame + 1)
            % match self.state {
                MovementAnimationState::Idling => Self::IDLE_FRAMES,
                MovementAnimationState::Walking => Self::WALKING_FRAMES,
                MovementAnimationState::Gliding => Self::GLIDING_FRAMES,
            };
    }

    /// Update animation state if it changes.
    pub fn update_state(&mut self, state: MovementAnimationState) {
        if self.state != state {
            match state {
                MovementAnimationState::Idling => *self = Self::idling(),
                MovementAnimationState::Walking => *self = Self::walking(),
                MovementAnimationState::Gliding => *self = Self::gliding(),
            }
        }
    }

    /// Whether animation changed this tick.
    pub fn changed(&self) -> bool {
        self.timer.is_finished()
    }

    /// Return sprite index in the atlas.
    pub fn get_atlas_index(&self) -> usize {
        match self.state {
            MovementAnimationState::Idling => 0,
            MovementAnimationState::Walking => 4 + self.frame,
            MovementAnimationState::Gliding => 8,
        }
    }
}
