use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::surface::Surface;
use sdl2::image::LoadTexture;
use sdl2::render::{Texture, TextureCreator};
use sdl2::ttf::Font as Sdl2Font;
use sdl2::video::WindowContext;

use ron::de::from_reader;

use crate::ui::{Font, FontFace, ProtoSpriteConfig};
use crate::core::{SpriteConfig, SpriteSource, TextureMap};

pub struct AssetRepo<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,

    pub texture: Option<Texture<'a>>,
    pub textures: HashMap<String, Texture<'a>>,
    pub fonts: [Option<Font<'a>>; 3],
}

impl<'a> AssetRepo<'a> {
    pub fn new(texture_creator: &'a TextureCreator<sdl2::video::WindowContext>) -> AssetRepo<'a> {
        Self {
            texture_creator,
            texture: None,
            textures: HashMap::new(),
            fonts: [None, None, None],
        }
    }

    pub fn init(mut self, image_path: &Path, font_path: &Path) -> Result<Self, String> {
        self.load_textures_from_path(image_path)?;

        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        self.add_font(FontFace::Normal, ttf_context.load_font(font_path, 28)?)?;
        self.add_font(FontFace::Big, ttf_context.load_font(font_path, 48)?)?;
        self.add_font(FontFace::VeryBig, ttf_context.load_font(font_path, 96)?)?;

        Ok(self)
    }

    pub fn load_textures_from_path(self: &mut Self, path: &Path) -> Result<(), String> {
        for entry in path.read_dir().map_err(|e| e.to_string())? {
            let file_path = entry.map_err(|e| e.to_string())?.path();

            if let Some(ext) = file_path.clone().extension().and_then(|k| k.to_str()) {
                if let Some(key) = file_path.clone().file_stem().and_then(|k| k.to_str()) {
                    if ext == "png" {
                        // println!("[DEBUG] loading texture {:?}...", file_path);
                        let val = self.texture_creator.load_texture(file_path)?;
                        self.textures.insert(key.to_string(), val);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn create_texture_from_path(
        self: &mut Self,
        path: &Path,
    ) -> Result<HashMap<String, SpriteConfig>, String> {
        let proto_sprites = read_proto_sprite_config(path);
        let pixel_format = self.texture_creator.default_pixel_format();
        let (sheet_width, sheet_height, sprite_positions) =
            compile_sprite_sheet_data(path, &proto_sprites, pixel_format)?;
        let mut tmp_cvs = Surface::new(sheet_width, sheet_height, pixel_format)?.into_canvas()?;
        let tmp_texture_creator = tmp_cvs.texture_creator();

        let texture_map = compile_texture_map(&proto_sprites, &sprite_positions);

        for (file, (x, y, w, h)) in sprite_positions.iter() {
            let file_path = path.join(Path::new(file));
            let texture = tmp_texture_creator.load_texture(file_path)?;
            tmp_cvs.copy(&texture, Rect::new(0, 0, *w, *h), Rect::new(*x, *y, *w, *h))?;
        }

        tmp_cvs.present();

        let sprite_sheet_texture = self
            .texture_creator
            .create_texture_from_surface(tmp_cvs.into_surface())
            .map_err(|e| e.to_string())?;

        self.texture = Some(sprite_sheet_texture);

        Ok(texture_map)
    }

    pub fn add_font(
        self: &mut Self,
        key: FontFace,
        font: Sdl2Font,
    ) -> Result<(), String> {
        self.fonts[key as usize] = Some(Font::from_font(self.texture_creator, font)?);
        Ok(())
    }
}

fn read_proto_sprite_config(path: &Path) -> Vec<(String, ProtoSpriteConfig)> {
    let p = path.join("sprites.ron");
    let f = match File::open(p) {
        Ok(result) => result,
        Err(e) => {
            panic!("Error opening proto sprite config file: {:?}", e);
        }
    };

    match from_reader(f) {
        Ok(result) => result,
        Err(e) => {
            panic!("Error parsing proto sprite config: {:?}", e);
        }
    }
}

fn compile_texture_map(
    proto_sprites: &Vec<(String, ProtoSpriteConfig)>,
    sprite_positions: &HashMap<String, (i32, i32, u32, u32)>,
) -> TextureMap {
    let mut texture_map = HashMap::new();
    for (key, proto_cfg) in proto_sprites.iter() {
        if proto_cfg.files.len() == 1 {
            // a single image without animation
            let (x, y, w, h) = sprite_positions.get(&proto_cfg.files[0]).unwrap();
            texture_map.insert(
                key.to_string(),
                SpriteConfig {
                    source: SpriteSource::Static(*x, *y),
                    dim: (*w, *h),
                    offset: proto_cfg.offset.unwrap_or((0, 0)),
                    alpha: proto_cfg.alpha.unwrap_or(255),
                    scale: 1.0,
                },
            );
        } else {
            let (_, _, w, h) = sprite_positions.get(&proto_cfg.files[0]).unwrap();
            let mut frame_positions = vec![];

            for f in proto_cfg.files.iter() {
                let (x, y, _, _) = sprite_positions.get(f).unwrap();
                frame_positions.push((*x, *y));
            }

            texture_map.insert(
                key.to_string(),
                SpriteConfig {
                    source: SpriteSource::SimpleAnimation(
                        proto_cfg.frame_durration.unwrap_or(50),
                        frame_positions,
                    ),
                    dim: (*w, *h),
                    offset: proto_cfg.offset.unwrap_or((0, 0)),
                    alpha: proto_cfg.alpha.unwrap_or(255),
                    scale: 1.0,
                },
            );
        }
    }

    texture_map
}

fn compile_sprite_sheet_data(
    path: &Path,
    proto_sprites: &Vec<(String, ProtoSpriteConfig)>,
    pixel_format_enum: PixelFormatEnum,
) -> Result<(u32, u32, HashMap<String, (i32, i32, u32, u32)>), String> {
    let (mut x, mut y) = (0, 0);
    let mut line_height = 0;
    let max_width = 2048;
    let mut total_height = 0;
    let tmp_cvs = Surface::new(max_width, 2048, pixel_format_enum)?.into_canvas()?;
    let tmp_texture_creator = tmp_cvs.texture_creator();
    let mut sprite_position_cache: HashMap<String, (i32, i32, u32, u32)> = HashMap::new();

    for (_, proto_cfg) in proto_sprites.iter() {
        for file in proto_cfg.files.iter() {
            if sprite_position_cache.contains_key(file) {
                continue;
            }

            let file_path = path.join(Path::new(file));
            let texture = tmp_texture_creator.load_texture(file_path)?;
            let TextureQuery { width, height, .. } = texture.query();

            line_height = std::cmp::max(line_height, height);

            if x as u32 + width > max_width {
                // new line
                x = 0;
                y = y + line_height as i32;
                total_height = total_height + line_height;
                line_height = 0;
            }

            sprite_position_cache.insert(file.to_string(), (x, y, width, height));

            x += width as i32;
        }
    }

    total_height = total_height + line_height; // add last line

    // println!(
    //     "sheet width: {}, sheet height: {}, sprite positions: {:?}",
    //     max_width, total_height, sprite_position_cache
    // );
    Ok((max_width, total_height, sprite_position_cache))
}
