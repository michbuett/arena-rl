mod components;
mod core;
mod ui;

extern crate sdl2;

use std::path::Path;
use std::time::Duration;

use specs::prelude::*;
use sdl2::image::InitFlag;
use sdl2::rect::{Rect};
use sdl2::render::TextureCreator;
use sdl2::ttf::Sdl2TtfContext;

use crate::ui::*;
use crate::core::*;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG);
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let dim = Rect::new(0, 0, 800, 450);
    let window = video_subsystem
        .window("ArenaRL", dim.width(), dim.height())
        .allow_highdpi()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let pixel_ratio = (window.drawable_size().0 / window.size().0) as u8;
    let mut canvas = window.into_canvas()
        .index(find_render_driver("opengl").unwrap())
        .accelerated()
        .present_vsync() // caps fps at 60 (monitor refresh rate)
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let assets = init_assets(&texture_creator, &ttf_context)?;
    // let (dispatcher, world) = init_ecs();
    let mut click_areas = vec!();
    let mut sdl_events = sdl_context.event_pump()?;
    let mut game = Game::Start;
    // let mut game = Game::Combat(CombatData {
    //     turn: 0,
    //     world,
    //     dispatcher,
    //     state: CombatState::Init (vec!(
    //         create_player(WorldPos(6.0, 6.0)),
    //         create_player(WorldPos(6.0, 5.0)),
    //         create_player(WorldPos(5.0, 6.0)),
    //         create_enemy(WorldPos(4.0, 3.0)),
    //         create_enemy(WorldPos(6.0, 0.0)),
    //     )),
    // });

    let mut ui = init_ui(&game, canvas.viewport(), pixel_ratio);

    'main: loop {
        let user_input = poll(&mut sdl_events, &click_areas, &ui);

        if let Some(UserInput::Exit()) = user_input {
            break 'main;
        }

        game = step(game, &user_input);
        ui = step_ui(ui, &user_input);
        click_areas = render(&mut canvas, &ui, &game, &assets)?;

        std::thread::sleep(Duration::from_nanos(0)); // TODO: fps limit without vsync
    }

    Ok(())
}


//////////////////////////////////////////////////
// PRIVATE HELPER FUNCTIONS

fn init_assets<'a>(
    texture_creator: &'a TextureCreator<sdl2::video::WindowContext>,
    ttf_context: &Sdl2TtfContext,
) -> Result<AssetRepo<'a>, String> {
    let p = Path::new("./assets/fonts/font.ttf");
    let mut assets = AssetRepo::new( &texture_creator );

    assets.load_textures_from_path(Path::new("./assets/images"))?;
    assets.add_font("normal", ttf_context.load_font(p, 28)?)?;
    assets.add_font("big", ttf_context.load_font(p, 48)?)?;
    assets.add_font("very big", ttf_context.load_font(p, 96)?)?;

    Ok(assets)
}

fn init_ecs<'a, 'b>() -> (Dispatcher<'a, 'b>, World) {
    let dispatcher = DispatcherBuilder::new()
        .with(components::Animation, "Animaton", &[])
        .with(components::EndOfLiveSystem, "EOL", &[])
        .build();

    let mut world = World::new();
    let map = dummy();

    components::register(&mut world);

    world.add_resource(map);

    (dispatcher, world)
}

/// opengl, opengles2, metal, software, ...
fn find_render_driver(name: &str) -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() { 
        if item.name == name {
            return Some(index as u32);
        }
    }
    None
}

// fn create_player(pos: WorldPos) -> GameObject {
//     let actor = ActorBuilder::new(pos, Attributes::new(4, 4, 4), Team("Player", 1))
//         .armor(Armor { look: vec!("player"), protection: 2 })
//         .build();

//     GameObject::Actor(actor)
// }

// fn create_enemy(pos: WorldPos) -> GameObject {
//     let actor = ActorBuilder::new(pos, Attributes::new(4, 4, 4), Team("CPU", 2))
//         .behaviour(AiBehaviour::Default)
//         .armor(Armor { look: vec!("enemy"), protection: 0 })
//         .build();
    
//     GameObject::Actor(actor)
// }
