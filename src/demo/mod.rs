//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

mod aabb;
mod animation;
mod enemy;
mod events;
mod food;
mod gun;
pub mod level;
mod movement;
mod platform;
pub mod player;
mod boss;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        platform::plugin,
        enemy::plugin,
        level::plugin,
        movement::plugin,
        gun::plugin,
        player::plugin,
        food::plugin,
        boss::plugin,
    ));
}
