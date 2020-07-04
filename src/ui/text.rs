use std::string::ToString;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::ttf::Font as Sdl2Font;

const ASCII: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

pub struct Font<'a> {
    texture: Texture<'a>,
    glyphs: Vec<GlyphRegion>,
    line_height: u32,
    space_advance: i32,
}

struct GlyphRegion {
    start: i32,
    advance: i32,
    width: u32,
    height: u32,
}

impl<'a> Font<'a> {
    // pub fn load(
    //     ttf_context: &'a Sdl2TtfContext,
    //     texture_creator: &'a TextureCreator<sdl2::video::WindowContext>,
    //     path: &Path,
    //     size: u16,
    // ) -> Result<Self, String> {
    //     let font = ttf_context.load_font(path, size)?;
    //     Self::from_font(texture_creator, font)
    // }

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
                .blended(Color::RGBA(0, 0, 0, 255))
                // .blended(Color::RGBA(255, 255, 255, 255))
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

        // texture.set_color_mod(0, 0, 250);
        // texture.set_color_mod(255, 0, 0);

        Ok(Font {
            texture,
            glyphs,
            line_height: total_height,
            space_advance,
        })
    }

    pub fn text(&self, txt: String) -> TextBuilder {
        TextBuilder::new(&self, txt)
    }
}

pub struct TextBuilder<'a> {
    font: &'a Font<'a>,
    text: String,
    background: Option<Color>,
    padding: u32,
    border: Option<(u32, Color)>,
    max_width: u32,
}

impl<'a> TextBuilder<'a> {
    fn new(font: &'a Font<'a>, text: String) -> Self {
        Self {
            font,
            text,
            background: None,
            padding: 0,
            border: None,
            max_width: u32::max_value(),
        }
    }

    pub fn background(self: Self, color: Color) -> Self {
        Self {
            background: Some(color),
            ..self
        }
    }

    pub fn padding(self: Self, padding: u32) -> Self {
        Self {
            padding: padding,
            ..self
        }
    }

    pub fn border(self: Self, padding: u32, color: Color) -> Self {
        Self {
            border: Some((padding, color)),
            ..self
        }
    }

    pub fn max_width(self: Self, max_width: u32) -> Self {
        Self {
            max_width: max_width,
            ..self
        }
    }

    pub fn prepare(self: Self) -> PreparedText<'a> {
        let (mut x, mut y) = (0, 0);
        let mut words = Vec::new();
        let mut width_so_far: u32 = 0;
        let border_width = if let Some((w, _)) = self.border { w } else { 0 };
        let spacing = 2 * self.padding + 2 * border_width;
        let max_width = self.max_width - spacing;

        for line in self.text.lines() {
            for t in line.split_whitespace() {
                let word = PreparedWord::prepare(self.font, t);
                let text_width = word.width;
                let advance = self.font.space_advance + text_width as i32;

                if x > 0 && (x + advance) as u32 > max_width {
                    // text does not fit in current line
                    // => wrap text (no wrap if first word in line)
                    x = 0;
                    y += self.font.line_height as i32;
                    width_so_far = max_width;
                }

                words.push(((x, y), word));

                x += advance;

                if x as u32 > width_so_far {
                    width_so_far = x as u32;
                }
            }

            x = 0;
            y += self.font.line_height as i32;
        }

        PreparedText {
            texture: &self.font.texture,
            words,
            dim: (width_so_far + spacing, y as u32 + spacing),
            background: self.background,
            padding: self.padding,
            border: self.border,
        }
    }
}

struct PreparedWord<'a> {
    chars: Vec<&'a GlyphRegion>,
    width: u32,
}

impl<'a> PreparedWord<'a> {
    fn prepare(font: &'a Font, txt: &str) -> Self {
        let mut x = 0;
        let mut chars = Vec::new();

        for c in txt.chars() {
            if let Some(r) = find_glyph_region(c, &font.glyphs) {
                chars.push(r);
                x = x + r.advance;
            }
        }

        PreparedWord {
            chars,
            width: x as u32,
        }
    }

    pub fn draw(
        self: &Self,
        texture: &Texture,
        cvs: &mut WindowCanvas,
        pos: (i32, i32),
    ) -> Result<(), String> {
        let (mut x, y) = pos;

        for r in self.chars.iter() {
            let from = Rect::new(r.start, 0, r.width, r.height);
            let to = Rect::new(x, y, r.width, r.height);

            cvs.copy(&texture, Some(from), Some(to))?;

            x = x + r.advance;
        }

        Ok(())
    }
}

pub struct PreparedText<'a> {
    texture: &'a Texture<'a>,
    words: Vec<((i32, i32), PreparedWord<'a>)>,
    dim: (u32, u32),
    background: Option<Color>,
    padding: u32,
    border: Option<(u32, Color)>,
}

impl<'a> PreparedText<'a> {
    pub fn draw(self: &Self, cvs: &mut WindowCanvas, pos: (i32, i32)) -> Result<(), String> {
        let (w, h) = self.dimension();

        if let Some(color) = self.background {
            // enable BlendMode::Blend to allow (half) transparent backgrounds
            // cvs.set_blend_mode(BlendMode::Blend); // TODO test performance impact
            cvs.set_draw_color(color);
            cvs.fill_rect(Rect::new(pos.0, pos.1, w, h))?;
        }

        if let Some((bw, border_color)) = self.border {
            let xl = pos.0;
            let xr = pos.0 + w as i32 - bw as i32;
            let yt = pos.1;
            let yb = pos.1 + h as i32 - bw as i32;

            cvs.set_draw_color(border_color);
            cvs.fill_rect(Rect::new(xl, yt, w, bw))?; // top
            cvs.fill_rect(Rect::new(xl, yt, bw, h))?; // left
            cvs.fill_rect(Rect::new(xr, yt, bw, h))?; // right
            cvs.fill_rect(Rect::new(xl, yb, w, bw))?; // bottom
        }

        let bw = if let Some((val, _)) = self.border {
            val as i32
        } else {
            0
        };
        let p = self.padding as i32;
        let (x, y) = (pos.0 + p + bw, pos.1 + p + bw);

        for ((offset_x, offset_y), word) in self.words.iter() {
            word.draw(self.texture, cvs, (x + offset_x, y + offset_y))?;
        }

        Ok(())
    }

    pub fn dimension(self: &Self) -> (u32, u32) {
        self.dim
    }
}

//////////////////////////////////////////////////
// PRIVATE HELPER

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
