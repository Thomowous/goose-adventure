use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{
    asset_tracking::LoadResource,
    demo::{
        animation::MovementAnimation,
        movement::MovementController,
        player::Player,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<EnemyAssets>();
    app.add_systems(Update, (update_enemies, explode));
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Enemy {
    pub health: f32,
    pub max_health: f32,
    pub garlic_cooldown: Timer,
    pub can_attack: bool,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct EnemyAssets {
    #[dependency]
    pub mushroom: Handle<Image>,
    #[dependency]
    pub garlic: Handle<Image>,
}

impl FromWorld for EnemyAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            mushroom: assets.load_with_settings(
                "images/mushroom.png",
                |settings: &mut ImageLoaderSettings| settings.sampler = ImageSampler::nearest(),
            ),
            garlic: assets
                .load_with_settings("images/garlic.png", |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest()
                }),
        }
    }
}

pub fn mushroom(
    health: f32,
    location: Vec3,
    size_modifier: f32,
    enemy_assets: &EnemyAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 3, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let enemy_animation = MovementAnimation::new();

    (
        Enemy {
            health: health,
            max_health: health,
            garlic_cooldown: Timer::from_seconds(1.2, TimerMode::Repeating),
            can_attack: true,
        },
        Transform::from_translation(location).with_scale(Vec2::splat(size_modifier).extend(1.0)),
        MovementController {
            speed: 300.0,
            ..default()
        },
        AI,
        Sprite::from_atlas_image(
            enemy_assets.mushroom.clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: enemy_animation.get_atlas_index(),
            },
        ),
        enemy_animation,
    )
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct AI;

fn update_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut ai_query: Query<
        (&Transform, &mut MovementController, &mut Enemy),
        (With<AI>, Without<Player>),
    >,
    player_query: Query<&Transform, (With<Player>, Without<AI>)>,
    enemy_assets: If<Res<EnemyAssets>>,
) {
    for player_transform in player_query {
        for (ai_transform, mut ai_movement, mut enemy) in &mut ai_query {
            let range_min = 300.0;
            let range_max = 500.0;
            let aggro_range = 1500.0;
            let diff_x = player_transform.translation.x - ai_transform.translation.x;
            if diff_x.abs() > aggro_range {
                continue;
            }
            ai_movement.facing_right = diff_x > 0.0;
            ai_movement.horizontal = 0.0;
            // Player on the left
            let sign = diff_x.signum();
            // player too far = move closer
            if diff_x.abs() > range_max {
                ai_movement.horizontal = sign * 1.0;
            } else if diff_x.abs() < range_min {
                ai_movement.horizontal = sign * -1.0;
            } else {
                enemy.garlic_cooldown.tick(time.delta());
                if enemy.garlic_cooldown.just_finished() {
                    commands.spawn((
                        Garlic,
                        Transform::from_translation(ai_transform.translation)
                            .with_scale(Vec3::new(1.5, 1.5, 1.0)),
                        Sprite {
                            image: enemy_assets.garlic.clone(),
                            ..Default::default()
                        },
                        MovementController {
                            speed: diff_x.abs() / range_max * 1111.0,
                            horizontal: sign,
                            velocity: Vec2::new(0.0, 1500.0),
                            gravity: 100.0,
                            grounded: false,
                            facing_right: diff_x > 0.0,
                            ..Default::default()
                        },
                    ));
                }
            }
        }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Garlic;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Explosion {
    pub radius: f32,
}

fn explode(
    mut commands: Commands,
    player_query: Query<(&Transform, &mut Player)>,
    explosion_query: Query<(&Transform, &Explosion, Entity)>,
    mut app_exit: MessageWriter<AppExit>
) {
    for (player_transform, mut player) in player_query {
        for (explosion_transform, explosion, explosion_entity) in explosion_query {
            let distance = player_transform
                .translation
                .distance(explosion_transform.translation);
            if distance < explosion.radius {
                player.health -= 40.0;
                if player.health <= 0.0 { 
                    app_exit.write(AppExit::Success);
                }
            }
            commands.get_entity(explosion_entity).unwrap().despawn();
        }
    }
}
