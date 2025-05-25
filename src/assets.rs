use crate::{
    animations::{FadeAnimation, SpriteAnimation},
    GameState, MarkedForDeath,
};
use bevy::{
    asset::{io::Reader, ron, AssetLoader, AsyncReadExt, LoadContext, LoadedFolder},
    ecs::system::EntityCommands,
    prelude::*,
    reflect::TypePath,
    utils::HashMap,
};
use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    prelude::*,
};
use serde::Deserialize;
use std::{marker::PhantomData, time::Duration};
use thiserror::Error;

/// An generic asset loader for data stored in RON files
pub struct DataAssetLoader<T> {
    _t: PhantomData<fn() -> T>,
}

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DataAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl<T> AssetLoader for DataAssetLoader<T>
where
    for<'de> T: Deserialize<'de> + Asset + Send + Sync,
{
    type Asset = T;
    type Settings = ();
    type Error = DataAssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let custom_asset = ron::de::from_bytes::<T>(&bytes)?;
        Ok(custom_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct RawSpriteConfig(Vec<(String, ProtoSpriteConfig)>);

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
    pub texture: Handle<Image>,
    pub map: HashMap<String, SpriteConfig>,
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

#[derive(Resource, Default)]
struct CombatSpritesFolder(Handle<LoadedFolder>);

pub fn assets_plugin(app: &mut App) {
    app.init_asset::<RawSpriteConfig>()
        .register_asset_loader(DataAssetLoader::<RawSpriteConfig> { _t: PhantomData })
        .add_systems(OnEnter(GameState::Start), load_sprites)
        .add_systems(
            Update,
            (
                check_textures.run_if(in_state(GameState::Start)),
                update_sprites_from_visuals.run_if(resource_exists::<SpriteConfigMap>),
            ),
        );
}

fn load_sprites(mut commands: Commands, asset_server: Res<AssetServer>) {
    // load multiple, individual sprites from a folder
    commands.insert_resource(CombatSpritesFolder(
        asset_server.load_folder("images/combat"),
    ));
}

fn check_textures(
    mut events: EventReader<AssetEvent<LoadedFolder>>,
    rpg_sprite_folder: Res<CombatSpritesFolder>,
    raw_sprite_config: Res<Assets<RawSpriteConfig>>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut textures: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&rpg_sprite_folder.0) {
            let sprite_cfg_map = create_texture_atlas(
                &raw_sprite_config,
                &asset_server,
                &mut atlas_layouts,
                &mut textures,
            );

            commands.insert_resource(sprite_cfg_map);

            next_state.set(GameState::MainMenu);
        }
    }
}

fn create_texture_atlas(
    raw_sprite_config: &Res<Assets<RawSpriteConfig>>,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    textures: &mut ResMut<Assets<Image>>,
) -> SpriteConfigMap {
    // ) -> (TextureAtlasLayout, Image, SpriteConfigMap) {
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
                    asset_server.get_handle::<Image>(format!("images/combat/{}", file_name))
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

    let (texture_atlas_layout, texture) = texture_atlas_builder.build().unwrap();
    let atlas_layout_handle = atlas_layouts.add(texture_atlas_layout);
    let atlas_texture_handle = textures.add(texture);

    SpriteConfigMap {
        layout: atlas_layout_handle,
        texture: atlas_texture_handle,
        map: sprite_configs,
    }
}

#[derive(Component, Clone, Debug)]
pub enum Visual {
    Single(String),
    Multi(Vec<String>),
}

#[derive(Component)]
pub struct SpriteContainer;

fn update_sprites_from_visuals(
    mut commands: Commands,
    sprite_cfg_map: Res<SpriteConfigMap>,
    layouts: Res<Assets<TextureAtlasLayout>>,
    visual_q: Query<(Entity, &Visual, Option<&FadeAnimation>), Changed<Visual>>,
    sprite_container_q: Query<(&Parent, Entity, &SpriteContainer)>,
) {
    if visual_q.is_empty() {
        return;
    }

    let Some(layout) = layouts.get(sprite_cfg_map.layout.id()) else {
        panic!("Could not find layout in assets for sprite map")
    };

    for (entity, visual, fade_animation) in visual_q.iter() {
        // clear "old" sprites
        for (parent, child_entity, _) in sprite_container_q.iter() {
            if parent.get() == entity {
                commands.entity(child_entity).insert(MarkedForDeath);
            }
        }

        // the container entity exists to help removeing all changed visuals
        let mut container_entity_cmd = commands.spawn((
            Name::new("SpriteContainer"),
            SpatialBundle {
                ..Default::default()
            },
            SpriteContainer,
        ));

        container_entity_cmd.set_parent(entity);

        match visual {
            Visual::Single(v) => {
                insert_sprite(
                    container_entity_cmd,
                    &sprite_cfg_map,
                    layout,
                    v,
                    0.0,
                    fade_animation,
                );
            }

            Visual::Multi(visuals) => {
                for (idx, v) in visuals.iter().enumerate() {
                    container_entity_cmd.with_children(|parent| {
                        let c = parent.spawn_empty();
                        insert_sprite(c, &sprite_cfg_map, layout, v, idx as f32, fade_animation);
                    });
                }
            }
        }
    }
}

fn insert_sprite(
    mut commands: EntityCommands,
    sprite_cfg_map: &Res<SpriteConfigMap>,
    layout: &TextureAtlasLayout,
    visual: &str,
    zlayer: f32,
    fade_animation: Option<&FadeAnimation>,
) {
    match sprite_cfg_map.map.get(visual) {
        Some(SpriteConfig::Single {
            image_id,
            offset: (dx, dy),
        }) => {
            let index = layout.get_texture_index(*image_id).unwrap_or(0);

            commands.insert((
                SpriteBundle {
                    texture: sprite_cfg_map.texture.clone(),
                    transform: Transform::from_translation(Vec3::new(*dx, *dy, zlayer)),
                    ..Default::default()
                },
                TextureAtlas {
                    layout: sprite_cfg_map.layout.clone(),
                    index,
                },
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
                .map(|image_id| layout.get_texture_index(*image_id).unwrap_or(0))
                .collect::<Vec<_>>();

            let current_idx: usize = rand_between(0..indices.len());
            let mut timer = Timer::new(
                Duration::from_millis(*frame_duration as u64),
                TimerMode::Repeating,
            );

            timer.tick(Duration::from_millis(
                rand_between(0..*frame_duration) as u64
            ));

            commands.insert((
                SpriteBundle {
                    texture: sprite_cfg_map.texture.clone(),
                    transform: Transform::from_translation(Vec3::new(*dx, *dy, zlayer)),
                    ..Default::default()
                },
                TextureAtlas {
                    layout: sprite_cfg_map.layout.clone(),
                    index: *indices.first().unwrap(),
                },
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

fn rand_between<R, T>(r: R) -> T
where
    T: SampleUniform,
    R: SampleRange<T>,
{
    let mut rng = thread_rng();
    rng.gen_range(r)
}
