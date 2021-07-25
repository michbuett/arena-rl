use std::time::Instant;

use specs::prelude::*;
use specs_derive::Component;

use crate::ui::{ScreenPos, ScreenSprite};
use crate::core::SpriteConfig;

pub enum ZLayer {
    Floor,
    // GameObject,
    // Fx
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct ZLayerFloor;

#[derive(Component)]
#[storage(VecStorage)]
pub struct ZLayerGameObject;

#[derive(Component)]
#[storage(VecStorage)]
pub struct ZLayerFX;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Sprites(Instant, Vec<SpriteConfig>);

impl Sprites {
    pub fn new(sprites: Vec<SpriteConfig>) -> Self {
        Self(Instant::now(), sprites)
    }
    
    pub fn sample(&self, pos: ScreenPos) -> SpriteIter {
        let runtime = (Instant::now() - self.0).as_millis();

        SpriteIter {
            runtime,
            pos,
            sprites: self.1.iter(),
        }
    }

    pub fn set_scale(&mut self, new_scale: f32) {
        for sprite in self.1.iter_mut() {
            sprite.scale = new_scale;
        }
    }
}

pub struct SpriteIter<'a> {
    runtime: u128,
    pos: ScreenPos,
    sprites: std::slice::Iter<'a, SpriteConfig>,
}

impl<'a> Iterator for SpriteIter<'a> {
    type Item = ScreenSprite;

    fn next(&mut self) -> Option<Self::Item> {
        self.sprites
            .next()
            .map(|cfg| ScreenSprite(self.pos, cfg.sample(self.runtime)))
    }
}
