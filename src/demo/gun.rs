use std::{default, f32::consts::PI};

use bevy::{ecs::event::SetEntityEventTarget, prelude::*};

use crate::{
    audio::sound_effect,
    demo::{
        aabb::AABB, boss::Boss, enemy::Enemy, events::LevelUpEvent, level::CurseLevel, movement::MovementController, platform::Platform, player::PlayerAssets
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, shoot);
    app.add_systems(FixedUpdate, (update_bullets, handle_collisions).chain());
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Gun {
    pub shooting_cooldown: Timer,
    pub can_shoot: bool,
    pub shooting: bool,
    pub enabled: bool,
}

fn shoot(
    mut commands: Commands,
    player_assets: If<Res<PlayerAssets>>,
    time: Res<Time>,
    mut gun_query: Query<(&mut Gun, &Transform, &MovementController)>,
) {
    for (mut gun, transform, movement) in &mut gun_query {
        if !gun.can_shoot {
            gun.shooting_cooldown.tick(time.delta());
        }
        if !gun.shooting {
            if gun.shooting_cooldown.just_finished() {
                gun.can_shoot = true;
                gun.shooting_cooldown.reset();
            }
            continue;
        }
        if gun.can_shoot || gun.shooting_cooldown.just_finished() {
            gun.can_shoot = false;
            spawn_bullet(
                &mut commands,
                transform.translation,
                movement.facing_right,
                &player_assets,
            );
        }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Bullet {
    pub velocity: Vec2,
    pub despawn_timer: Timer,
}

impl Default for Bullet {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            despawn_timer: Timer::from_seconds(0.7, TimerMode::Once),
        }
    }
}

fn update_bullets(time: Res<Time>, mut bullet_query: Query<(&mut Transform, &Bullet)>) {
    for (mut transform, bullet) in &mut bullet_query {
        transform.translation += (bullet.velocity * time.delta_secs()).extend(0.0);
    }
}

pub fn spawn_bullet(
    mut commands: &mut Commands,
    player_location: Vec3,
    player_facing_right: bool,
    player_assets: &If<Res<PlayerAssets>>,
) {
    let spread = 0.0;
    let velocity = Vec2 {
        x: if player_facing_right { 1000.0 } else { -1000.0 },
        y: rand::random::<f32>() * spread - spread / 2.0,
    };

    let spawn_location = Vec3 {
        x: player_location.x + if player_facing_right { 40.0 } else { -40.0 },
        y: player_location.y + 22.0,
        z: 3.0,
    };

    // TODO: adjust based on velocity
    let rotation = if player_facing_right { 0.0 } else { PI };

    commands.spawn(sound_effect(player_assets.honk.first().unwrap().clone()));
    commands.spawn(sound_effect(player_assets.gunshot.first().unwrap().clone()));
    commands.spawn((
        Bullet {
            velocity: velocity,
            ..Default::default()
        },
        Transform::from_translation(spawn_location)
            .with_scale(Vec2::splat(0.3).extend(1.0))
            .with_rotation(Quat::from_rotation_z(rotation)),
        Sprite {
            image: player_assets.bullet.clone(),
            ..Default::default()
        },
    ));
}

fn handle_collisions(
    mut commands: Commands,
    time: Res<Time>,
    platform_query: Query<&AABB, With<Platform>>,
    mut enemy_query: Query<(&Transform, &mut Enemy, Entity), (Without<Bullet>, Without<Boss>)>,
    mut boss_query: Query<(&Transform, &mut Boss, Entity), (Without<Bullet>, Without<Enemy>)>,
    mut bullet_query: Query<(&Transform, &mut Bullet, Entity), (Without<Enemy>, Without<Boss>)>,
    mut curse_level: If<ResMut<CurseLevel>>,
) {
    'bullet: for (bullet_transform, mut bullet, bullet_entity) in bullet_query {
        bullet.despawn_timer.tick(time.delta());
        if bullet.despawn_timer.just_finished() {
            commands.get_entity(bullet_entity).unwrap().despawn();
            continue 'bullet;
        }
        let bullet_size = bullet_transform.scale.xy() * 16.0;
        let mut bullet_aabb = AABB::new(bullet_transform.translation.xy(), bullet_size);
        for platform_aabb in &platform_query {
            let depth = bullet_aabb.get_intersection_depth(&platform_aabb);
            if depth != Vec2::ZERO {
                commands.get_entity(bullet_entity).unwrap().despawn();
                continue 'bullet;
            }
        }
        for (enemy_transform, mut enemy, enemy_entity) in &mut enemy_query {
            let enemy_aabb = AABB::new(
                enemy_transform.translation.xy(),
                enemy_transform.scale.xy() * 16.0,
            );
            let depth = enemy_aabb.get_intersection_depth(&bullet_aabb);
            if depth != Vec2::ZERO {
                enemy.health -= 50.0;
                if enemy.health <= 0.0 {
                    commands.get_entity(enemy_entity).unwrap().despawn();
                    curse_level.value += 1;
                    curse_level.needs_change = true;
                }
                commands.get_entity(bullet_entity).unwrap().despawn();
                continue 'bullet;
            }
        }
        for (boss_transform, mut boss, boss_entity) in &mut boss_query {
            let boss_aabb = AABB::new(
                boss_transform.translation.xy(),
                boss_transform.scale.xy() * 16.0,
            );
            let depth = boss_aabb.get_intersection_depth(&bullet_aabb);
            if depth != Vec2::ZERO {
                boss.health -= 50.0;
                if boss.health <= 0.0 {
                    commands.get_entity(boss_entity).unwrap().despawn();
                    curse_level.value += 100;
                    curse_level.needs_change = true;
                }
                commands.get_entity(bullet_entity).unwrap().despawn();
                continue 'bullet;
            }
        }
    }
}
