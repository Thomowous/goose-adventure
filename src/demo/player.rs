//! Player-specific behavior.

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    audio::sound_effect,
    demo::{
        animation::MovementAnimation,
        food::Food,
        gun::{self, Gun},
        movement::{FollowCamera, MovementController},
    },
};


pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (record_player_directional_input, record_shooting_input, update_health_bar)
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
}

/// The player character.
pub fn player(
    max_speed: f32,
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 3, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = MovementAnimation::new();

    (
        Name::new("Player"),
        Player {
            has_gun: false,
            health: 100.0,
            max_health: 100.0,
        },
        Sprite::from_atlas_image(
            player_assets.goose.clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: player_animation.get_atlas_index(),
            },
        ),
        Transform::from_xyz(-250.0, -200.0, 10.0).with_scale(Vec2::splat(2.0).extend(1.0)),
        MovementController {
            speed: max_speed,
            ..default()
        },
        FollowCamera,
        player_animation,
        Gun {
            shooting: false,
            can_shoot: true,
            shooting_cooldown: Timer::from_seconds(0.8, TimerMode::Repeating),
            enabled: false,
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
                HealthBar,
            ), 
            
        ]

    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
pub struct Player {
    pub has_gun: bool,
    pub health: f32,
    pub max_health: f32,
}

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = 0.0;
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent += 1.0;
    }

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        // Jump
        if input.pressed(KeyCode::Space) {
            if controller.grounded {
                controller.velocity.y = controller.jump_force * 3.0;
                controller.grounded = false;
            } else if controller.velocity.y < 0.0 && controller.jump_timer < controller.jump_time {
                controller.gliding = true;
            } else {
                controller.gliding = false;
            }
        } else {
            controller.gliding = false;
        }
        // Movement
        controller.horizontal = intent;
        if intent < 0.0 {
            controller.facing_right = false;
        }
        if intent > 0.0 {
            controller.facing_right = true;
        }
    }
}

fn record_shooting_input(
    mut commands: Commands,
    player_assets: If<Res<PlayerAssets>>,
    input: Res<ButtonInput<MouseButton>>,
    mut gun_query: Query<&mut Gun>,
) {
    for mut gun in &mut gun_query {
        if gun.enabled {
            if gun.shooting_cooldown.duration().as_secs_f32() <= 0.3 {
                gun.shooting = input.pressed(MouseButton::Left);
            } else {
                gun.shooting = input.just_pressed(MouseButton::Left);
            }
        }
        if input.just_pressed(MouseButton::Left) {
            commands.spawn(sound_effect(player_assets.honk.first().unwrap().clone()));
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    goose: Handle<Image>,
    #[dependency]
    pub bullet: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
    #[dependency]
    pub honk: Vec<Handle<AudioSource>>,
    #[dependency]
    pub gunshot: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            goose: assets.load_with_settings(
                "images/goose_sprite_sheet.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            bullet: assets.load_with_settings(
                "images/bullet.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
            honk: vec![assets.load("audio/sound_effects/honk.ogg")],
            gunshot: vec![assets.load("audio/sound_effects/gunshot.ogg")],
        }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct HealthBar;

fn update_health_bar(
    player_query: Query<&Player>,
    mut health_bar_query: Query<&mut Transform, With<HealthBar>>,
) {
    for player in player_query {
        for mut health_bar_transform in health_bar_query.iter_mut() {
            health_bar_transform.scale.x = (player.health / player.max_health).max(0.0);
            health_bar_transform.translation.x = (player.health / player.max_health) * 40.0 - 40.0;

        }
    } 
}