use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;

use sdl2::image::LoadTexture;
use sdl2::render::{Texture, TextureCreator};
use sdl2::ttf::Font as Sdl2Font;

use crate::ui::Font;

pub struct AssetRepo<'a> {
    texture_creator: &'a TextureCreator<sdl2::video::WindowContext>,
    textures: HashMap<OsString, Texture<'a>>,
    fonts: HashMap<String, Font<'a>>,
}

impl<'a> AssetRepo<'a> {

    pub fn new(
        texture_creator: &'a TextureCreator<sdl2::video::WindowContext>,
    ) -> AssetRepo<'a> {
        Self {
            texture_creator,
            textures: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub fn load_textures_from_path(
        self: &mut Self,
        path: &Path,
    ) -> Result<(), String> {
        for entry in path.read_dir().map_err(|e| e.to_string())? {
            let file_path = entry.map_err(|e| e.to_string())?.path();
            if let Some(key) = file_path.clone().file_stem() {
                println!("[DEBUG] loading texture {:?}...", file_path);
                let val = self.texture_creator.load_texture(file_path)?;
                self.textures.insert(key.to_os_string(), val);
            }
        }

        Ok(())
    }

    pub fn add_font(
        self: &mut Self,
        key: &str,
        font: Sdl2Font
    ) -> Result<(), String> {
        let val = Font::from_font(self.texture_creator, font)?;
        self.fonts.insert(key.to_string(), val);
        Ok(())
    }

    pub fn texture(self: &Self, key: &str) -> Result<&'a Texture, String> {
        if let Some(t) = self.textures.get(&OsString::from(key)) {
            return Ok(t);
        }
        Err(format!("Cannot find texture for '{}'", key))
    }

    pub fn font(self: &Self, key: &str) -> Result<&'a Font, String> {
        if let Some(f) = self.fonts.get(key) {
            return Ok(f);
        }
        Err(format!("Cannot find font for {}", key))
    }
}
