use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

use crate::core::*;
use crate::ui::*;

const DLG_WIDTH: u32 = 400;

use crate::ui::types::*;
pub fn render(
    cvs: &mut WindowCanvas,
    viewport: &Rect,
    focus_pos: ScreenPos,
    game: &CombatData,
    assets: &AssetRepo,
) -> Result<ClickAreas, String> {
    let click_areas = if let CombatState::WaitForUserAction(_, ctxt) = &game.state {
        match ctxt {
            Some(InputContext::SelectedArea(p, objects, actions)) => 
                draw_area_details(cvs, assets, focus_pos, viewport, *p, objects, actions)?,

            Some(InputContext::Opportunity(o, actions)) => 
                draw_reaction_details(cvs, assets, focus_pos, viewport, o, actions)?,

            _ => vec!(),
        }
    } else {
        vec!()
    };

    Ok(click_areas)
}

fn draw_area_details(
    cvs: &mut WindowCanvas,
    assets: &AssetRepo,
    focus_pos: ScreenPos,
    viewport: &Rect,
    pos: WorldPos,
    objects: &Vec<GameObject>,
    actions: &Vec<(Action, u8)>,
) -> Result<ClickAreas, String> {
    
    let mut txt = format!("You look at ({}, {}).", pos.0, pos.1);

    if objects.is_empty() {
        txt += &"\nNothing special here.".to_string();
    } else {
        txt += &"\nYou see ...".to_string();
        for o in objects {
            match o {
                GameObject::Actor(a) => {
                    let condition = match a.num_wounds() {
                        0 => "perfect condition",
                        1 => "scratched",
                        2..=4 => "wounded",
                        _ => "critically wounded",
                    };

                    txt += &format!("\n{} ({})\n", a.name, condition);
                }

                GameObject::Item(_, i) => {
                    txt += &format!("\nan item ({}) on the ground\n", i.name);
                }
            }
        }
    }

    draw_dialog(cvs, assets, (txt, actions), focus_pos, viewport)
}

fn draw_reaction_details(
    cvs: &mut WindowCanvas,
    assets: &AssetRepo,
    focus_pos: ScreenPos,
    viewport: &Rect,
    _o: &Opportunity,
    actions: &Vec<(Action, u8)>,
) -> Result<ClickAreas, String> {
    let txt = format!("This is a dummy description for a reaction opportunity");

    draw_dialog(cvs, assets, (txt, actions), focus_pos, viewport)
}

fn draw_dialog(
    cvs: &mut WindowCanvas,
    assets: &AssetRepo,
    content: (String, &Vec<(Action, u8)>),
    ScreenPos(focus_x, focus_y): ScreenPos,
    viewport: &Rect,
) -> Result<ClickAreas, String> {
    let (text, actions) = content;
    let txt_box = &assets
        .font("normal")?
        .text(text)
        .max_width(DLG_WIDTH)
        .padding(10)
        .background(Color::RGB(252, 251, 250))
        .border(3, Color::RGB(23, 22, 21))
        .prepare();

    let (text_width, text_height) = txt_box.dimension();
    let screen_width = viewport.width() as i32;

    // TODO remove magic numbers
    let x = if (text_width as i32 + 25 < focus_x && focus_x < screen_width / 2) || focus_x + text_width as i32 + 150 > screen_width {
        // place the dialog to the left of the focus pos
        focus_x - text_width as i32 - 25
    } else {
        // place the dialog to the right of the focus pos
        focus_x + 150
    };
    let y = focus_y;

    txt_box.draw(cvs, (x, y))?;

    draw_actions(cvs, assets, (x, y + text_height as i32), DLG_WIDTH, actions)
}

fn draw_actions(
    cvs: &mut WindowCanvas,
    assets: &AssetRepo,
    (x, y): (i32, i32),
    max_width: u32,
    actions: &Vec<(Action, u8)>,
) -> Result<ClickAreas, String> {
    let (mut button_x, mut button_y) = (x, y + 10);
    let mut line_height = 0;
    let f = &assets.font("normal")?;
    let mut click_areas = Vec::new();

    for a in actions.iter() {
        let txt_box = f
            .text(display_text(a))
            .padding(10)
            .border(3, Color::RGB(23, 22, 21))
            .background(Color::RGB(252, 251, 250))
            .prepare();

        let (w, h) = txt_box.dimension();

        line_height = std::cmp::max(line_height, h);

        if button_x > x {
            let line_width = w + (button_x - x) as u32;
            if line_width > max_width {
                button_x = x;
                button_y = button_y + line_height as i32 + 10;
            }
        }
       
        let action = a.clone(); // first clone so closure can take owership of "action"
        click_areas.push(ClickArea {
            clipping_area: Rect::new(button_x, button_y, w, h),
            // second clone so closure can return the action more than once (-> Fn)
            action: Box::new(move |_| UserInput::SelectAction(action.clone()))
        });

        txt_box.draw(cvs, (button_x, button_y))?;

        button_x = button_x + w as i32 + 10;
    }

    Ok(click_areas)
}

//////////////////////////////////////////////////
// PRIVATE HELPER
//

fn display_text((action, delay): &(Action, u8)) -> String {
    match action {
        Action::StartTurn() => "".to_string(),
        Action::Wait(_) => format!("Wait ({})", delay),
        Action::MoveTo(..) => format!("Move Here ({})", delay),
        Action::Activate() => format!("Activate"),
        Action::Attack(_, a) => format!("{} ({})", a.name.0, delay),
        // Action::Defence(_, _, _, d) => format!("{} ({})", d.name.0, delay),
        // Action::EndTurn(..) => format!("End Turn"),
    }
}
