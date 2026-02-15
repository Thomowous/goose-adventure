//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    demo::{
        aabb::AABB,
        enemy::{Explosion, Garlic},
        platform::Platform,
        player::Player,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (apply_movement, handle_collisions, apply_follow_camera)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    pub speed: f32,
    pub jump_force: f32,
    pub velocity: Vec2,
    pub gravity: f32,
    pub grounded: bool,

    pub jump_time: f32,
    pub jump_timer: f32,
    pub horizontal: f32,
    pub gliding: bool,

    pub facing_right: bool,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            speed: 70.0,
            jump_force: 666.0,
            velocity: Vec2::ZERO,
            gravity: 100.0,
            grounded: false,
            jump_time: 1.0,
            jump_timer: 0.0,
            horizontal: 0.0,
            gliding: false,
            facing_right: true,
        }
    }
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&mut MovementController, &mut Transform)>,
) {
    let terminal_velocity = -1500.0;
    for (mut controller, mut transform) in &mut movement_query {
        controller.velocity.x = controller.speed * controller.horizontal;
        if !controller.grounded {
            if controller.gliding {
                controller.velocity.y = -controller.gravity * 0.3;
                controller.jump_timer += time.delta_secs();
            } else {
                controller.velocity.y -= controller.gravity;
            }
        }
        controller.velocity.y = controller.velocity.y.max(terminal_velocity);
        transform.translation += controller.velocity.extend(0.0) * time.delta_secs();
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FollowCamera;

fn apply_follow_camera(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    player_query: Query<&Transform, (With<FollowCamera>, Without<Camera2d>)>,
) {
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        if let Ok(player_transform) = player_query.single() {
            // camera_transform.translation.x = camera_transform.translation.x.max(player_transform.translation.x);
            camera_transform.translation.x = player_transform.translation.x;
        }
    }
}

fn handle_collisions(
    mut commands: Commands,
    platform_query: Query<&AABB, With<Platform>>,
    mut movement_query: Query<(
        &mut Transform,
        &mut MovementController,
        Entity,
        Option<&Garlic>,
    )>,
) {
    for (mut movement_transform, mut movement, entity, garlic) in &mut movement_query {
        let movement_size = movement_transform.scale.xy() * 16.0;
        let mut movement_aabb = AABB::new(movement_transform.translation.xy(), movement_size);
        let mut collided = false;
        for platform_aabb in &platform_query {
            if movement_aabb.bottom() > platform_aabb.top() {
                continue;
            }
            if movement_aabb.left() >= platform_aabb.right() {
                continue;
            }
            if movement_aabb.right() <= platform_aabb.left() {
                continue;
            }
            if movement_aabb.top() < platform_aabb.bottom() {
                continue;
            }

            collided = true;
            let mut depth = movement_aabb.get_intersection_depth(&platform_aabb);

            if depth.x.abs() <= 8.0 {
                movement_transform.translation.x += depth.x;
                movement.velocity.x = 0.0;
                movement_aabb.center = movement_transform.translation.xy();
                depth = movement_aabb.get_intersection_depth(&platform_aabb);
            }

            if depth.y.abs() <= 24.0 {
                movement_transform.translation.y += depth.y;
                movement.velocity.y = 0.0;

                if depth.y > 0.0 && depth.y <= 24.0 {
                    movement.grounded = true;
                    movement.jump_timer = 0.0;
                }
            }
        }
        if collided {
            if garlic.is_some() {
                commands.spawn((
                    Explosion { radius: 60.0 },
                    Transform::from_translation(movement_transform.translation),
                ));
                commands.get_entity(entity).unwrap().despawn();
            }
        } else {
            movement.grounded = false;
        }
    }
}
