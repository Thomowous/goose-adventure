use bevy::{
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    prelude::*,
};

use crate::{asset_tracking::LoadResource, demo::aabb::AABB};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlatformAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlatformAssets {
    #[dependency]
    pub grass0: Handle<Image>,
    #[dependency]
    pub grass1: Handle<Image>,
    #[dependency]
    pub grass2: Handle<Image>,
    #[dependency]
    pub grass3: Handle<Image>,
}
impl FromWorld for PlatformAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            grass0: assets.load_with_settings("images/grass0.png", |s: &mut _| {
                *s = ImageLoaderSettings {
                    sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                        // rewriting mode to repeat image,
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        mipmap_filter: ImageFilterMode::Nearest,
                        ..default()
                    }),
                    ..default()
                }
            }),
            grass1: assets.load_with_settings("images/grass1.png", |s: &mut _| {
                *s = ImageLoaderSettings {
                    sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                        // rewriting mode to repeat image,
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        mipmap_filter: ImageFilterMode::Nearest,
                        ..default()
                    }),
                    ..default()
                }
            }),
            grass2: assets.load_with_settings("images/grass2.png", |s: &mut _| {
                *s = ImageLoaderSettings {
                    sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                        // rewriting mode to repeat image,
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        mipmap_filter: ImageFilterMode::Nearest,
                        ..default()
                    }),
                    ..default()
                }
            }),
            grass3: assets.load_with_settings("images/grass3.png", |s: &mut _| {
                *s = ImageLoaderSettings {
                    sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                        // rewriting mode to repeat image,
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        mipmap_filter: ImageFilterMode::Nearest,
                        ..default()
                    }),
                    ..default()
                }
            }),
        }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Platform;

pub fn platform(location: Vec2, size: Vec2, platform_assets: &If<Res<PlatformAssets>>) -> impl Bundle {
    let real_location = location * 64.0;
    let real_size = size * 32.0;
    (
        Platform,
        Grass,
        AABB {
            center: real_location,
            half_size: real_size,
        },
        Transform::from_translation((real_location).extend(1.0))
            .with_scale(Vec3::new(2.0, 2.0, 1.0)),
        Sprite {
            image: platform_assets.grass0.clone(),
            custom_size: Some(real_size),
            rect: Some(Rect {
                min: Vec2::ZERO,
                max: real_size,
            }),
            ..Default::default()
        },
    )
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Grass;
