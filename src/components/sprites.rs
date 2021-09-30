use std::time::Instant;

use specs::prelude::*;
use specs_derive::Component;

use crate::ui::{ScreenPos, ScreenSprite, Align};
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
pub struct Sprites(Instant, Align, Vec<SpriteConfig>);

impl Sprites {
    pub fn new(sprites: Vec<SpriteConfig>) -> Self {
        Self(Instant::now(), Align::MidCenter, sprites)
    }
    
    pub fn sample(&self, pos: ScreenPos) -> SpriteIter {
        let runtime = (Instant::now() - self.0).as_millis();

        SpriteIter {
            runtime,
            pos,
            align: self.1,
            sprites: self.2.iter(),
        }
    }

    pub fn set_scale(&mut self, new_scale: f32) {
        for sprite in self.2.iter_mut() {
            sprite.scale = new_scale;
        }
    }

    pub fn set_alpha(&mut self, new_alpha: u8) {
        for sprite in self.2.iter_mut() {
            sprite.alpha = new_alpha;
        }
    }
}

pub struct SpriteIter<'a> {
    runtime: u128,
    pos: ScreenPos,
    align: Align,
    sprites: std::slice::Iter<'a, SpriteConfig>,
}

impl<'a> Iterator for SpriteIter<'a> {
    type Item = ScreenSprite;

    fn next(&mut self) -> Option<Self::Item> {
        self.sprites
            .next()
            .map(|cfg| ScreenSprite(self.pos, self.align, cfg.sample(self.runtime)))
    }
}
