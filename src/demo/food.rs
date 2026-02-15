use bevy::prelude::*;

use crate::{
    audio::sound_effect,
    demo::{
        gun::{self, Gun},
        level::CurseLevel,
        movement::MovementController,
        player::{Player, PlayerAssets},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, eat);
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Food {
    pub gives_gun: bool,
}

fn eat(
    mut commands: Commands,
    player_assets: If<Res<PlayerAssets>>,
    input: Res<ButtonInput<KeyCode>>,
    food_query: Query<(&Transform, &Food, Entity)>,
    player_query: Query<(&Transform, &mut Gun, &MovementController), With<Player>>,
    mut curse_level: ResMut<CurseLevel>,
) {
    if !input.just_pressed(KeyCode::KeyE) {
        return;
    }
    for (player_transform, mut gun, movement) in player_query {
        for (food_transform, food, entity) in food_query {
            if player_transform
                .translation
                .distance(food_transform.translation)
                < 64.0
            {
                if food.gives_gun {
                    curse_level.value = 1;
                    curse_level.needs_change = true;
                    gun.enabled = true;
                    gun.shooting_cooldown.reset();
                    gun::spawn_bullet(
                        &mut commands,
                        player_transform.translation,
                        movement.facing_right,
                        &player_assets,
                    );
                }
                commands.spawn(sound_effect(player_assets.honk.first().unwrap().clone()));
                commands.get_entity(entity).unwrap().despawn();
            }
        }
    }
}
