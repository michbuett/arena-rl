use crate::{
    MarkedForDeath,
    animations::{FadeAnimation, SpriteAnimation},
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    prelude::*,
};
use serde::Deserialize;
use std::{collections::HashMap, fmt::Display, time::Duration};

#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct RawSpriteConfig(pub Vec<(String, ProtoSpriteConfig)>);

#[derive(Debug, Clone, Deserialize)]
pub struct ProtoSpriteConfig {
    pub files: Vec<String>,
    pub offset: Option<(i32, i32)>,
    // pub alpha: Option<u8>,
    pub frame_durration: Option<u32>,
}

#[derive(Debug, Resource)]
pub struct SpriteConfigMap {
    pub layout: Handle<TextureAtlasLayout>,
    pub sources: TextureAtlasSources,
    pub texture: Handle<Image>,
    pub map: HashMap<String, SpriteConfig>,
}

impl SpriteConfigMap {
    pub fn new(
        raw_sprite_config: &Res<Assets<RawSpriteConfig>>,
        asset_server: &Res<AssetServer>,
        atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        textures: &mut ResMut<Assets<Image>>,
    ) -> SpriteConfigMap {
        let mut sprite_configs: HashMap<String, SpriteConfig> = HashMap::new();
        let mut texture_atlas_builder = TextureAtlasBuilder::default();

        for (_, rsc) in raw_sprite_config.iter() {
            for (key, proto_sprite_cfg) in rsc.0.iter() {
                let offset = proto_sprite_cfg
                    .offset
                    .map(|(dx, dy)| (dx as f32, dy as f32))
                    .unwrap_or((0.0, 0.0));

                if proto_sprite_cfg.files.len() == 1 {
                    let file_name = proto_sprite_cfg.files.first().unwrap();
                    let Some(handle) =
                        asset_server.get_handle::<Image>(combat_sprite_path(file_name))
                    else {
                        warn!(
                            "Cannot find handle for path \"{}\". (sprite_key=\"{}\")",
                            file_name, key,
                        );
                        continue;
                    };

                    let Some(texture) = textures.get(handle.id()) else {
                        warn!(
                            "{:?} did not resolve to an `Image` asset. (sprite_key=\"{}\")",
                            handle.path().unwrap(),
                            key,
                        );
                        continue;
                    };

                    texture_atlas_builder.add_texture(Some(handle.id()), texture);

                    sprite_configs.insert(
                        key.clone(),
                        SpriteConfig::Single {
                            image_id: handle.id(),
                            offset,
                        },
                    );
                } else {
                    let image_ids = proto_sprite_cfg
                        .files
                        .iter()
                        .filter_map(|file_name| {
                            asset_server.get_handle::<Image>(format!("images/combat/{}", file_name))
                        })
                        .map(|handle| handle.id())
                        .collect::<Vec<_>>();

                    for id in image_ids.iter() {
                        let Some(texture) = textures.get(*id) else {
                            warn!("{:?} did not resolve to an `Image` asset.", id);
                            continue;
                        };
                        texture_atlas_builder.add_texture(Some(*id), texture);
                    }

                    sprite_configs.insert(
                        key.clone(),
                        SpriteConfig::Animated {
                            image_ids,
                            offset,
                            frame_duration: proto_sprite_cfg.frame_durration.unwrap_or(50),
                        },
                    );
                }
            }
        }

        let (atlas_layout, atlas_sources, texture) = texture_atlas_builder.build().unwrap();
        let atlas_layout_handle = atlas_layouts.add(atlas_layout);
        let atlas_texture_handle = textures.add(texture);

        SpriteConfigMap {
            layout: atlas_layout_handle,
            sources: atlas_sources,
            texture: atlas_texture_handle,
            map: sprite_configs,
        }
    }
}

#[derive(Debug)]
pub enum SpriteConfig {
    Single {
        image_id: AssetId<Image>,
        offset: (f32, f32),
    },

    Animated {
        image_ids: Vec<AssetId<Image>>,
        offset: (f32, f32),
        frame_duration: u32,
    },
}

#[derive(Component, Clone, Debug)]
#[require(Visibility, Transform)]
pub enum Visual {
    Single(String),
    Multi(Vec<String>),
}

impl From<Vec<String>> for Visual {
    fn from(mut value: Vec<String>) -> Self {
        if value.len() == 1 {
            Visual::Single(value.pop().unwrap())
        } else {
            Visual::Multi(value)
        }
    }
}
impl From<&Vec<String>> for Visual {
    fn from(value: &Vec<String>) -> Self {
        Self::from(value.clone())
    }
}

#[derive(Component)]
pub struct SpriteContainer;

pub fn update_sprites_from_visuals(
    mut commands: Commands,
    sprite_cfg_map: Res<SpriteConfigMap>,
    // layouts: Res<Assets<TextureAtlasLayout>>,
    visual_q: Query<(Entity, &Visual, Option<&FadeAnimation>), Changed<Visual>>,
    sprite_container_q: Query<(&ChildOf, Entity, &SpriteContainer)>,
) {
    if visual_q.is_empty() {
        return;
    }

    for (entity, visual, fade_animation) in visual_q.iter() {
        // clear "old" sprites
        for (child_of, child_entity, _) in sprite_container_q.iter() {
            if child_of.parent() == entity {
                commands.entity(child_entity).insert(MarkedForDeath);
            }
        }

        // the container entity exists to help removeing all changed visuals
        let mut container_entity_cmd = commands.spawn((
            Name::new("SpriteContainer"),
            Transform::default(),
            Visibility::Inherited,
            SpriteContainer,
        ));

        container_entity_cmd.insert(ChildOf(entity));

        match visual {
            Visual::Single(v) => {
                insert_sprite(
                    container_entity_cmd,
                    &sprite_cfg_map,
                    v,
                    0.0,
                    fade_animation,
                );
            }

            Visual::Multi(visuals) => {
                for (idx, v) in visuals.iter().enumerate() {
                    container_entity_cmd.with_children(|parent| {
                        let c = parent.spawn_empty();
                        insert_sprite(c, &sprite_cfg_map, v, idx as f32, fade_animation);
                    });
                }
            }
        }
    }
}

fn insert_sprite(
    mut commands: EntityCommands,
    sprite_cfg_map: &Res<SpriteConfigMap>,
    visual: &str,
    zlayer: f32,
    fade_animation: Option<&FadeAnimation>,
) {
    match sprite_cfg_map.map.get(visual) {
        Some(SpriteConfig::Single {
            image_id,
            offset: (dx, dy),
        }) => {
            let index = sprite_cfg_map.sources.texture_index(*image_id).unwrap_or(0);
            let image = sprite_cfg_map.texture.clone();
            let atlas = TextureAtlas {
                layout: sprite_cfg_map.layout.clone(),
                index,
            };

            commands.insert((
                Transform::from_translation(Vec3::new(*dx, *dy, zlayer)),
                Sprite::from_atlas_image(image, atlas),
            ));

            if let Some(fa) = fade_animation {
                commands.insert(fa.clone());
            }
        }

        Some(SpriteConfig::Animated {
            image_ids,
            offset: (dx, dy),
            frame_duration,
        }) => {
            let indices = image_ids
                .iter()
                .map(|image_id| sprite_cfg_map.sources.texture_index(*image_id).unwrap_or(0))
                .collect::<Vec<_>>();

            let current_idx: usize = rand_between(0..indices.len());
            let mut timer = Timer::new(
                Duration::from_millis(*frame_duration as u64),
                TimerMode::Repeating,
            );
            let image = sprite_cfg_map.texture.clone();
            let atlas = TextureAtlas {
                layout: sprite_cfg_map.layout.clone(),
                index: *indices.first().unwrap(),
            };

            timer.tick(Duration::from_millis(
                rand_between(0..*frame_duration) as u64
            ));

            commands.insert((
                Transform::from_translation(Vec3::new(*dx, *dy, zlayer)),
                Sprite::from_atlas_image(image, atlas),
                SpriteAnimation {
                    indices,
                    current_idx,
                    timer,
                },
            ));
        }

        None => {
            warn!("Unknown visual '{}'", visual);
        }
    }
}

fn combat_sprite_path(n: impl Display) -> String {
    format!("images/combat/{}", n)
}

fn rand_between<R, T>(r: R) -> T
where
    T: SampleUniform,
    R: SampleRange<T>,
{
    let mut rng = thread_rng();
    rng.gen_range(r)
}
