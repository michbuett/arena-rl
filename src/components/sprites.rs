use specs::prelude::*;
use specs_derive::Component;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Sprites(pub Vec<Sprite>);

#[derive(Debug)]
pub struct Sprite {
    pub texture: String,
    pub region: (i32, i32, u32, u32),
    pub offset: (i32, i32),
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct VisualCmp(pub std::time::Instant, pub Vec<String>);
