mod sprites;

use crate::{
    GameState,
    combat::ManeuverTemplates,
    core::{ActorTemplates, Feats},
};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadState, io::Reader},
    prelude::*,
};
use core::panic;
use serde::Deserialize;
use sprites::{RawSpriteConfig, SpriteConfigMap, update_sprites_from_visuals};

pub use sprites::Visual;

use std::{fmt::Display, marker::PhantomData};
use thiserror::Error;

/// An generic asset loader for data stored in RON files
#[derive(TypePath)]
pub struct DataAssetLoader<T> {
    extensions: Vec<&'static str>,
    _t: PhantomData<T>,
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

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let custom_asset = ron::de::from_bytes::<T>(&bytes)?;
        Ok(custom_asset)
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}

impl<T> DataAssetLoader<T> {
    fn new(extension: &'static str) -> DataAssetLoader<T> {
        Self {
            extensions: vec![extension],
            _t: PhantomData,
        }
    }
}

#[derive(Component)]
struct AssetHandle(UntypedHandle);

#[derive(Component)]
struct Loading;

#[derive(Component)]
struct SpriteData;

#[derive(Component)]
struct OnLoaded(AssetLoadedEvent);

#[derive(Event, Clone, Copy)]
enum AssetLoadedEvent {
    SpriteConfigLoaded,
}

#[derive(EntityEvent, Clone, Copy)]
struct SpriteConfigLoadedEvent {
    entity: Entity,
}

#[derive(Event, Clone, Copy)]
struct AllAssetsLoadedEvent;

pub fn assets_plugin(app: &mut App) {
    app.init_asset::<ActorTemplates>()
        .init_asset::<RawSpriteConfig>()
        .init_asset::<ManeuverTemplates>()
        .init_asset::<Feats>()
        .register_asset_loader(DataAssetLoader::<ManeuverTemplates>::new("maneuvers.ron"))
        .register_asset_loader(DataAssetLoader::<ActorTemplates>::new("actors.ron"))
        .register_asset_loader(DataAssetLoader::<RawSpriteConfig>::new("sprites.ron"))
        .register_asset_loader(DataAssetLoader::<Feats>::new("feats.ron"))
        .add_observer(on_sprite_config_loaded_event)
        .add_observer(handle_all_assets_loaded_event)
        .add_systems(OnEnter(GameState::Start), load_data_files)
        .add_systems(
            Update,
            (
                check_asset_loading_state.run_if(in_state(GameState::Start)),
                update_sprites_from_visuals.run_if(resource_exists::<SpriteConfigMap>),
            ),
        );
}

fn load_data_files(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Loading,
        AssetHandle(
            asset_server
                .load::<ActorTemplates>("data/main.actors.ron")
                .untyped(),
        ),
    ));

    commands.spawn((
        Loading,
        AssetHandle(
            asset_server
                .load::<ManeuverTemplates>("data/main.maneuvers.ron")
                .untyped(),
        ),
    ));

    commands.spawn((
        Loading,
        AssetHandle(asset_server.load::<Feats>("data/main.feats.ron").untyped()),
    ));

    commands.spawn((
        Loading,
        SpriteData,
        OnLoaded(AssetLoadedEvent::SpriteConfigLoaded),
        AssetHandle(
            asset_server
                .load::<RawSpriteConfig>("images/combat/main.sprites.ron")
                .untyped(),
        ),
    ));
}

fn check_asset_loading_state(
    asset_server: Res<AssetServer>,
    loading_assets_q: Query<(Entity, &AssetHandle, Option<&OnLoaded>), With<Loading>>,
    mut commands: Commands,
) {
    let mut all_loaded = true;

    for (entity, AssetHandle(handle), on_loaded) in loading_assets_q.iter() {
        let Some(loading_state) = asset_server.get_load_state(handle) else {
            panic!("Could not get load state of asset '{:?}'", handle.path());
        };

        match loading_state {
            LoadState::Loaded => {
                commands.entity(entity).remove::<Loading>();

                if let Some(OnLoaded(ev)) = on_loaded {
                    match ev {
                        AssetLoadedEvent::SpriteConfigLoaded => {
                            commands.trigger(SpriteConfigLoadedEvent { entity });
                            all_loaded = false;
                        }
                    }
                }
            }

            LoadState::Loading | LoadState::NotLoaded => {
                all_loaded = false;
            }

            LoadState::Failed(err) => {
                warn!("Error loading asset '{:?}': {:?}", handle.path(), err);
            }
        }
    }

    if all_loaded {
        commands.trigger(AllAssetsLoadedEvent);
    }
}

fn on_sprite_config_loaded_event(
    trigger: On<SpriteConfigLoadedEvent>,
    asset_server: Res<AssetServer>,
    raw_sprite_config: Res<Assets<RawSpriteConfig>>,
    loading_assets_q: Query<&AssetHandle>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let SpriteConfigLoadedEvent { entity } = trigger.event();
    let AssetHandle(handle) = loading_assets_q.get(*entity)?;
    let typed_handle = handle.clone().typed::<RawSpriteConfig>();
    let Some(rsc) = raw_sprite_config.get(typed_handle.id()) else {
        panic!("Could not get RawSpriteConfig asset");
    };

    for (_, psc) in rsc.0.iter() {
        for f in psc.files.iter() {
            commands.spawn((
                Loading,
                AssetHandle(asset_server.load::<Image>(combat_sprite_path(f)).untyped()),
            ));
        }
    }

    Ok(())
}

fn handle_all_assets_loaded_event(
    _trigger: On<AllAssetsLoadedEvent>,
    asset_server: Res<AssetServer>,
    raw_sprite_config: Res<Assets<RawSpriteConfig>>,

    mut next_state: ResMut<NextState<GameState>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut textures: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    commands.insert_resource(SpriteConfigMap::new(
        &raw_sprite_config,
        &asset_server,
        &mut atlas_layouts,
        &mut textures,
    ));

    next_state.set(GameState::MainMenu);
}

fn combat_sprite_path(n: impl Display) -> String {
    format!("images/combat/{n}")
}
