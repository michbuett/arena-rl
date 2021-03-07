use sdl2::rect::Rect;
use sdl2::render::{TextureQuery, WindowCanvas};

use crate::ui::{ClickArea, ClickAreas, AssetRepo};
use crate::core::{UserInput, DisplayStr};

pub fn render(
    cvs: &mut WindowCanvas,
    viewport: &Rect,
    assets: &AssetRepo,
) -> Result<ClickAreas, String> {
    let logo = assets.texture("logo")?;
    let TextureQuery { width, height, .. } = logo.query();
    let xpos = ((viewport.width() - width) / 2) as i32;
    let ypos = ((viewport.height() - height) / 2) as i32;
    
    cvs.copy(&logo, None, Rect::new(xpos, ypos, width, height))?;

    let text = assets.font("normal")?
        .text(DisplayStr::new("Click somewhere to continue ..."))
        .prepare();

    text.draw(cvs, (
        ((viewport.width() - text.dimension().0) / 2) as i32,
        ypos + height as i32 + 500
    ))?;
   
    Ok(vec!(ClickArea {
        clipping_area: viewport.clone(),
        action: Box::new(|_| UserInput::NewGame ),
    }))
}
