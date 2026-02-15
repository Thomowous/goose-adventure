//! Spawn the main level.

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    demo::{
        aabb::AABB, boss::{BossAssets, boss}, enemy::{EnemyAssets, mushroom}, food::Food, gun::Gun, platform::{Grass, Platform, PlatformAssets, platform}, player::{Player, PlayerAssets, player}
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
    app.insert_resource::<CurseLevel>(CurseLevel {
        value: 0,
        needs_change: false,
    });
    app.add_systems(Update, curse_level_change);
}

#[derive(Resource)]
pub struct CurseLevel {
    pub value: u32,
    pub needs_change: bool,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
    barn: Handle<Image>,
    hay: Handle<Image>,
    pistol: Handle<Image>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
            barn: assets.load_with_settings(
                "images/barn.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            hay: assets.load_with_settings(
                "images/hay.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            pistol: assets.load_with_settings(
                "images/pistol.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: If<Res<LevelAssets>>,
    player_assets: If<Res<PlayerAssets>>,
    platform_assets: If<Res<PlatformAssets>>,
    enemy_assets: If<Res<EnemyAssets>>,
    boss_assets: If<Res<BossAssets>>,
    mut texture_atlas_layouts: If<ResMut<Assets<TextureAtlasLayout>>>,
    mut meshes: If<ResMut<Assets<Mesh>>>,
    mut materials: If<ResMut<Assets<ColorMaterial>>>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![
            player(400.0, &player_assets, &mut texture_atlas_layouts, &mut meshes, &mut materials),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            ),
            barn(&level_assets),
            // Platforms
            platform(
                Vec2::new(120.0, -5.5),
                Vec2::new(300.0, 1.0),
                &platform_assets
            ),
            // Gun
            (
                Food { gives_gun: true },
                Transform::from_xyz(3500.0, -300.0, 3.0).with_scale(Vec3::new(1.5, 1.5, 1.0)),
                Sprite {
                    image: level_assets.pistol.clone(),
                    ..Default::default()
                }
            ),
            // Enemies
            mushroom(
                250.0,
                Vec3::new(5000.0, -200.0, 4.0),
                5.0,
                &enemy_assets,
                &mut texture_atlas_layouts,
            ),
            boss(
                Vec2::new(10_000.0, 0.0),
                &boss_assets,
                &mut meshes,
                &mut materials,
            ),
            (
                Text2d::new("Quit the game already"),
                Transform::from_xyz(12_500.0, 0.0, 10.0),
            ),
            (
                Text2d::new("You weren't supposed to see this..."),
                Transform::from_xyz(17_500.0, 0.0, 10.0),
            )
        ],
    ));

    commands.spawn((
        Text2d::new("press Left Mouse Button to honk"),
        Transform::from_xyz(0.0, -100.0, 10.0),
    ));
    for i in 0..10 {
        let x: f32 = 800.0 + (i as f32) * 200.0 + rand::random::<f32>() * 180.0;
        if i == 0 {
            commands.spawn((
                Text2d::new("press E to eat"),
                Transform::from_xyz(x, -100.0, 10.0),
            ));
        }
        commands.spawn((
            Food { gives_gun: false },
            Transform::from_xyz(x, -300.0, 3.0).with_scale(Vec3::new(1.5, 1.5, 1.0)),
            Sprite {
                image: level_assets.hay.clone(),
                ..Default::default()
            },
        ));
    }

    for i in 0..15 {
        let x: f32 = 5500.0 + (i as f32) * 100.0 + rand::random::<f32>() * 100.0;
        commands.spawn(mushroom(
            100.0,
            Vec3::new(x, -200.0, 4.0),
            rand::random::<f32>() + 3.0,
            &enemy_assets,
            &mut texture_atlas_layouts,
        ));
    }
}

fn barn(level_assets: &If<Res<LevelAssets>>) -> impl Bundle {
    (
        Name::new("Barn"),
        Transform::from_translation((Vec2::new(-11.4, 5.0) * 64.0).extend(1.0))
            .with_scale(Vec3::new(10.0, 10.0, 1.0)),
        Sprite {
            image: level_assets.barn.clone(),
            ..Default::default()
        },
        Platform,
        AABB {
            center: Vec2::new(-6.0, -2.0) * 64.0,
            half_size: Vec2::new(2.0, 8.0) * 32.0,
        },
    )
}

fn curse_level_change(
    mut commands: Commands,
    grass_query: Query<&mut Sprite, With<Grass>>,
    gun_query: Query<&mut Gun>,
    player_query: Query<&Transform, With<Player>>,
    platform_assets: If<Res<PlatformAssets>>,
    mut curse_level: If<ResMut<CurseLevel>>,
) {
    if !curse_level.needs_change {
        return;
    }
    curse_level.needs_change = false;
    let image = match curse_level.value {
        0 => &platform_assets.grass0,
        1 => &platform_assets.grass1,
        2..7 => &platform_assets.grass2,
        _ => &platform_assets.grass3,
    };
    if curse_level.value >= 2 {
        for mut gun in gun_query {
            gun.shooting_cooldown = Timer::from_seconds(0.1, TimerMode::Repeating);
        }
    }
    for mut sprite in grass_query {
        sprite.image = image.clone();
    }
    if curse_level.value > 100 {
        for player_transform in player_query {
            commands.spawn(
                (
                    Text2d::new("THE END"),
                    Transform::from_xyz(player_transform.translation.x, 0.0, 10.0),
                    TextFont {
                        font_size: 50.0,
                        ..default()
                    },
                )
            );
        }

    }
}