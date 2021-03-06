use std::cmp::max;
use std::collections::HashMap;
use std::string::ToString;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::ttf::Font as Sdl2Font;
use sdl2::video::WindowContext;

use crate::ui::types::ScreenText;

const ASCII: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

pub struct Font<'a> {
    texture: Texture<'a>,
    glyphs: Vec<GlyphRegion>,
    line_height: u32,
    space_advance: i32,

    texture_creator: &'a TextureCreator<WindowContext>,
    cached_texts: HashMap<String, (Texture<'a>, u32, u32)>,
}

struct GlyphRegion {
    start: i32,
    advance: i32,
    width: u32,
    height: u32,
}

impl<'a> Font<'a> {
    pub fn from_font(
        texture_creator: &'a TextureCreator<sdl2::video::WindowContext>,
        font: Sdl2Font,
    ) -> Result<Self, String> {
        let mut total_width = 0;
        let mut total_height = 0;
        let mut glyphs: Vec<GlyphRegion> = Vec::new();
        let mut space_advance = 0;

        for c in ASCII.chars() {
            if let Some(metric) = font.find_glyph_metrics(c) {
                let (w, h) = font.size_of_char(c).map_err(to_string)?;

                glyphs.push(GlyphRegion {
                    start: total_width as i32,
                    width: w,
                    height: h,
                    advance: metric.advance,
                });

                if c == ' ' {
                    space_advance = metric.advance;
                }

                total_width += w;
                total_height = h;
            } else {
                return Err(format!("Unsupported character: {}", c));
            }
        }

        let mut font_canvas = Surface::new(
            total_width,
            total_height,
            texture_creator.default_pixel_format(),
        )?
        .into_canvas()?;

        let font_texture_creator = font_canvas.texture_creator();
        let mut x = 0;

        for (i, c) in ASCII.char_indices() {
            let GlyphRegion { width, .. } = glyphs[i];

            let char_surface = font
                .render(&c.to_string())
                .blended(Color::RGBA(255, 255, 255, 255))
                .map_err(to_string)?;

            let char_tex = font_texture_creator
                .create_texture_from_surface(&char_surface)
                .map_err(to_string)?;

            let target = Rect::new(x, 0, width, total_height);
            font_canvas.copy(&char_tex, None, Some(target))?;

            x += width as i32;
        }

        let texture = texture_creator
            .create_texture_from_surface(font_canvas.into_surface())
            .map_err(to_string)?;

        Ok(Font {
            texture,
            glyphs,
            line_height: total_height,
            space_advance,
            texture_creator,
            cached_texts: HashMap::new(),
        })
    }

    pub fn draw(&mut self, screen_txt: ScreenText, cvs: &mut WindowCanvas) -> Result<(), String> {
        let cache_key = screen_txt.text.to_string();

        if let Some((tex, w, h)) = self.cached_texts.get(&cache_key) {
            let (x, y) = screen_txt.pos.to_xy();
            return cvs.copy(tex, Rect::new(0, 0, *w, *h), Rect::new(x, y, *w, *h));
        }

        let (x, y) = screen_txt.pos.to_xy();
        let prepared_text = prepare(screen_txt, self);
        let (w, h) = prepared_text.dim;
        // let pixel_format = sdl2::pixels::PixelFormatEnum::RGBA8888;
        let pixel_format = self.texture_creator.default_pixel_format();
        let mut target_tex = self
            .texture_creator
            // !!! ATTENSION: there seems to be a very wierd issue if texture is to small
            // the background/transparency of small textures is broken
            .create_texture_target(pixel_format, max(w, 65), max(h, 65))
            .map_err(to_string)?;

        target_tex.set_blend_mode(BlendMode::Blend);

        draw_text(
            prepared_text,
            cvs,
            &mut self.texture,
            &mut target_tex,
            (x, y, w, h),
        )?;

        self.cached_texts.insert(cache_key, (target_tex, w, h));

        Ok(())
    }
}

struct PreparedWord {
    chars: Vec<(i32, i32, u32, u32)>,
    width: u32,
}

impl PreparedWord {
    fn prepare(glyphs: &Vec<GlyphRegion>, txt: &str) -> Self {
        let mut x = 0;
        let mut chars = Vec::new();

        for c in txt.chars() {
            if let Some(r) = find_glyph_region(c, glyphs) {
                chars.push((r.start, r.advance, r.width, r.height));
                x = x + r.advance;
            }
        }

        PreparedWord {
            chars,
            width: x as u32,
        }
    }

    fn draw(
        self: &Self,
        texture: &Texture,
        cvs: &mut WindowCanvas,
        target: &mut Texture,
        pos: (i32, i32),
    ) -> Result<(), String> {
        let (mut x, y) = pos;

        for (start, advance, width, height) in self.chars.iter() {
            let from = Rect::new(*start, 0, *width, *height);
            let to = Rect::new(x, y, *width, *height);

            cvs.with_texture_canvas(target, |texture_canvas| {
                texture_canvas.copy(&texture, Some(from), Some(to)).unwrap();
            })
            .map_err(to_string)?;

            x = x + advance;
        }

        Ok(())
    }
}

struct PreparedText {
    words: Vec<((i32, i32), PreparedWord)>,
    dim: (u32, u32),
    color: (u8, u8, u8, u8),
    background: Color,
    // background: Option<Color>,
    padding: u32,
    border: Option<(u32, Color)>,
}

fn prepare<'a>(text: ScreenText, font: &'a Font) -> PreparedText {
    let (mut x, mut y) = (0, 0);
    let mut words = Vec::new();
    let mut width_so_far: u32 = 0;
    let border_width = text.border.map(|(w, _)| w).unwrap_or(0);
    let spacing = 2 * text.padding + 2 * border_width;
    let max_width = text.max_width - spacing;

    for line in text.text.into_string().lines() {
        for t in line.split_whitespace() {
            let word = PreparedWord::prepare(&font.glyphs, t);
            let text_width = word.width;
            let advance = font.space_advance + text_width as i32;

            if x > 0 && (x + advance) as u32 > max_width {
                // text does not fit in current line
                // => wrap text (no wrap if first word in line)
                x = 0;
                y += font.line_height as i32;
                width_so_far = max_width;
            }

            words.push(((x, y), word));

            x += advance;

            if x as u32 > width_so_far {
                width_so_far = x as u32;
            }
        }

        x = 0;
        y += font.line_height as i32;
    }

    let width = max(text.min_width, width_so_far + spacing);
    let height = y as u32 + spacing;

    PreparedText {
        words,
        dim: (width, height),
        color: text.color,
        background: text.background.map(|(r, g, b, a)| Color::RGBA(r, g, b, a)).unwrap_or(Color::RGBA(0, 0, 0, 0)),
        padding: text.padding,
        border: text
            .border
            .map(|(w, (r, g, b, a))| (w, Color::RGBA(r, g, b, a))),
    }
}

fn find_glyph_region(c: char, metrics: &Vec<GlyphRegion>) -> Option<&GlyphRegion> {
    let ascii_index = c as usize;
    if ascii_index >= 32 && ascii_index <= 126 {
        metrics.get(ascii_index - 32)
    } else {
        None
    }
}

fn to_string(s: impl ToString) -> String {
    s.to_string()
}

fn draw_background(
    cvs: &mut WindowCanvas,
    target_texture: &mut Texture,
    color: Color,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
) -> Result<(), String> {
    cvs.with_texture_canvas(target_texture, |texture_canvas| {
        texture_canvas.set_blend_mode(BlendMode::Blend); // TODO test performance impact
        texture_canvas.set_draw_color(color);
        texture_canvas.fill_rect(Rect::new(x, y, w, h)).unwrap();
    })
    .map_err(to_string)
}

fn draw_border(
    cvs: &mut WindowCanvas,
    target_texture: &mut Texture,
    color: Color,
    bw: u32,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
) -> Result<(), String> {
    let xl = x;
    let xr = x + w as i32 - bw as i32;
    let yt = y;
    let yb = y + h as i32 - bw as i32;

    cvs.with_texture_canvas(target_texture, |texture_canvas| {
        texture_canvas.set_draw_color(color);
        texture_canvas.fill_rect(Rect::new(xl, yt, w, bw)).unwrap(); // top
        texture_canvas.fill_rect(Rect::new(xl, yt, bw, h)).unwrap(); // left
        texture_canvas.fill_rect(Rect::new(xr, yt, bw, h)).unwrap(); // right
        texture_canvas.fill_rect(Rect::new(xl, yb, w, bw)).unwrap(); // bottom
    })
    .map_err(to_string)
}

fn draw_text(
    text: PreparedText,
    cvs: &mut WindowCanvas,
    texture: &mut Texture,
    target: &mut Texture,
    (x, y, w, h): (i32, i32, u32, u32),
) -> Result<(), String> {
    draw_background(cvs, target, text.background, 0, 0, w, h)?;

    if let Some((bw, border_color)) = text.border {
        draw_border(cvs, target, border_color, bw, 0, 0, w, h)?;
    }

    let shift = text.border.map(|(val, _)| val).unwrap_or(0) as i32 + text.padding as i32;

    texture.set_alpha_mod(text.color.3);
    texture.set_color_mod(text.color.0, text.color.1, text.color.2);

    for ((offset_x, offset_y), word) in text.words.iter() {
        word.draw(texture, cvs, target, (shift + offset_x, shift + offset_y))?;
    }

    texture.set_alpha_mod(255);
    texture.set_color_mod(0, 0, 0);

    cvs.copy(&target, Rect::new(0, 0, w, h), Rect::new(x, y, w, h))?;

    Ok(())
}
