use bevy::{image::{ImageLoaderSettings, ImageSampler}, prelude::*};

use crate::{asset_tracking::LoadResource, demo::{enemy::{EnemyAssets, Garlic}, level::CurseLevel, movement::MovementController, player::Player}};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<BossAssets>();
    app.add_systems(FixedUpdate, move_boss);
    app.add_systems(Update, update_health_bar);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct BossAssets {
    #[dependency]
    pub boss: Handle<Image>,
    // #[dependency]
    // pub music: Handle<AudioSource>,
}

impl FromWorld for BossAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            boss: assets.load_with_settings(
                "images/boss.png",
                |settings: &mut ImageLoaderSettings| settings.sampler = ImageSampler::nearest(),
            ),
        }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Boss {
    pub health: f32,
    max_health: f32,
    speed: f32,
    target_x: f32,
    move_cooldown: Timer,
    attacked: bool,
}

pub fn boss(
    location: Vec2,
    boss_assests: &BossAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> impl Bundle {
    (
        Boss {
            health: 1200.0,
            max_health: 1200.0,
            speed: 500.0,
            target_x: location.x,
            move_cooldown: Timer::from_seconds(1.0, TimerMode::Repeating),
            attacked: false,
        },
        Transform::from_translation(location.extend(3.0)).with_scale(Vec3::new(5.0, 5.0, 1.0)),
        Sprite {
            image: boss_assests.boss.clone(),
            ..Default::default()
        },
        children![
            (
                Mesh2d(meshes.add(Rectangle::new(80.0, 6.0))),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
                Transform::from_xyz(0.0, 50.0, 4.0),
            ), 
            (
                Mesh2d(meshes.add(Rectangle::new(80.0, 6.0))),
                MeshMaterial2d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
                Transform::from_xyz(0.0, 50.0, 5.0),
                HealthBarBoss,
            ), 
        ]
    )
}

fn move_boss(
    mut commands: Commands,
    time: Res<Time>,
    mut boss_query: Query<(&mut Transform, &mut Boss), Without<Player>>,
    player_query: Query<&Transform, (With<Player>, Without<Boss>)>,
    enemy_assets: Res<EnemyAssets>,
    curse_level: Res<CurseLevel>,
) {
    if curse_level.value < 8 {
        return;
    }
    for player_transform in player_query {
        for (mut boss_transform, mut boss) in &mut boss_query {
            boss.move_cooldown.tick(time.delta());
            if boss.move_cooldown.just_finished() {
                boss.attacked = false;
                boss.target_x = player_transform.translation.x;
            }
            let diff_x = boss.target_x - boss_transform.translation.x;
            let sign = diff_x.signum();
            if diff_x.abs() >= 30.0 {
                boss_transform.translation.x += sign * boss.speed * time.delta_secs();
            } else if !boss.attacked {
                boss.attacked = true;
                commands.spawn(
               (
                        Garlic,
                        Transform::from_translation(boss_transform.translation)
                            .with_scale(Vec3::new(1.5, 1.5, 1.0)),
                        Sprite {
                            image: enemy_assets.garlic.clone(),
                            ..Default::default()
                        },
                        MovementController {
                            speed: 0.0,
                            horizontal: sign,
                            velocity: Vec2::new(0.0, -250.0),
                            gravity: 100.0,
                            grounded: false,
                            facing_right: diff_x > 0.0,
                            ..Default::default()
                        },
                    )
                );
            } 
        }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct HealthBarBoss;


fn update_health_bar(
    boss_query: Query<&Boss>,
    mut health_bar_query: Query<&mut Transform, With<HealthBarBoss>>,
) {
    for boss in boss_query {
        for mut health_bar_transform in health_bar_query.iter_mut() {
            health_bar_transform.scale.x = (boss.health / boss.max_health).max(0.0);
            health_bar_transform.translation.x = (boss.health / boss.max_health) * 40.0 - 40.0;

        }
    } 
}