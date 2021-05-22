mod components;
mod core;
mod ui;

extern crate sdl2;

use std::path::Path;
use std::time::Duration;

use sdl2::image::InitFlag;
use sdl2::rect::Rect;

use crate::core::*;
use crate::ui::*;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG);
    let dim = Rect::new(0, 0, 1200, 600);
    let window = video_subsystem
        .window("ArenaRL", dim.width(), dim.height())
        // .fullscreen_desktop()
        .allow_highdpi()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let pixel_ratio = (window.drawable_size().0 / window.size().0) as u8;
    let mut canvas = window
        .into_canvas()
        .index(find_render_driver("opengl").unwrap())
        .accelerated()
        .present_vsync() // caps fps at 60 (monitor refresh rate)
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut assets = AssetRepo::new(&texture_creator).init(
        Path::new("./assets/images"),
        Path::new("./assets/fonts/font.ttf"),
    )?;

    let texture_map = assets.create_texture_from_path(Path::new("./assets/images/combat"))?;

    let mut click_areas = vec![];
    let mut sdl_events = sdl_context.event_pump()?;
    let mut game = Game::Start;
    let mut ui = init_ui(canvas.viewport(), pixel_ratio, texture_map);

    'main: loop {
        let user_input = poll(&mut sdl_events, &click_areas, &ui);

        if let Some(UserInput::Exit()) = user_input {
            break 'main;
        }

        game = step(game, &user_input);
        ui = step_ui(ui, &game, &user_input);
        click_areas = render(&mut canvas, &ui, &game, &mut assets)?;

        std::thread::sleep(Duration::from_nanos(0)); // TODO: fps limit without vsync
    }

    Ok(())
}

//////////////////////////////////////////////////
// PRIVATE HELPER FUNCTIONS

/// opengl, opengles2, metal, software, ...
fn find_render_driver(name: &str) -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == name {
            return Some(index as u32);
        }
    }
    None
}
