use std::collections::HashMap;
use std::path::Path;

use sdl2::video::WindowContext;
use sdl2::image::LoadTexture;
use sdl2::render::{Texture, TextureCreator};
use sdl2::ttf::Font as Sdl2Font;

use crate::ui::{FontFace, Font};

pub struct AssetRepo<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    pub textures: HashMap<String, Texture<'a>>,
    pub fonts: [Option<Font<'a>>; 3],
}

impl<'a> AssetRepo<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<sdl2::video::WindowContext>,
    ) -> AssetRepo<'a> {
        Self {
            texture_creator,
            textures: HashMap::new(),
            fonts: [None, None, None],
        }
    }

    pub fn init(
        mut self,
        image_path: &Path,
        font_path: &Path,
    ) -> Result<Self, String> {
        self.load_textures_from_path(image_path)?;

        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        self.add_font(FontFace::Normal, ttf_context.load_font(font_path, 28)?)?;
        self.add_font(FontFace::Big, ttf_context.load_font(font_path, 48)?)?;
        self.add_font(FontFace::VeryBig, ttf_context.load_font(font_path, 96)?)?;

        Ok(self)
    }

    pub fn load_textures_from_path(
        self: &mut Self,
        path: &Path,
    ) -> Result<(), String> {
        for entry in path.read_dir().map_err(|e| e.to_string())? {
            let file_path = entry.map_err(|e| e.to_string())?.path();
            if let Some(key) = file_path.clone().file_stem().and_then(|k| k.to_str()) {
                println!("[DEBUG] loading texture {:?}...", file_path);
                let val = self.texture_creator.load_texture(file_path)?;
                self.textures.insert(key.to_string(), val);
            }
        }

        Ok(())
    }

    pub fn add_font(
        self: &mut Self,
        // key: &str,
        key: FontFace,
        font: Sdl2Font
    ) -> Result<(), String> {
        // let val = Font::from_font(self.texture_creator, font)?;
        // self.fonts.insert(key.to_string(), val);
        self.fonts[key as usize] = Some(Font::from_font(self.texture_creator, font)?);
        Ok(())
    }

    // pub fn texture(self: &Self, key: &str) -> Result<&'a Texture, String> {
    //     if let Some(t) = self.textures.get(key) {
    //     // if let Some(t) = self.textures.get(&OsString::from(key)) {
    //         return Ok(t);
    //     }
    //     Err(format!("Cannot find texture for '{}'", key))
    // }

    // pub fn font(self: &Self, key: &str) -> Result<&'a Font, String> {
    //     if let Some(f) = self.fonts.get(key) {
    //         return Ok(f);
    //     }
    //     Err(format!("Cannot find font for {}", key))
    // }

    // pub fn draw_text(self: &mut Self, cvs: &mut WindowCanvas, text: ScreenText) -> Result<(), String> {
    //     if let Some(f) = self.fonts.get_mut(text.font) {
    //         return f.draw(text, cvs)
    //     }
    //     Err(format!("Cannot find font '{}'", text.font))
    // }
}
