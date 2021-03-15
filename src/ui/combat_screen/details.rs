use std::cmp::max;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

use super::map::{TILE_HEIGHT, TILE_WIDTH};
use crate::core::*;
use crate::ui::*;

const DLG_WIDTH: u32 = 400;
const SPACING: u32 = 5;

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
            Some(InputContext::SelectedArea(p, objects, actions)) => {
                draw_area_details(cvs, assets, focus_pos, viewport, *p, objects, actions)?
            }

            // Some(InputContext::Opportunity(o, actions)) =>
            //     draw_reaction_details(cvs, assets, focus_pos, viewport, o, actions)?,
            _ => vec![],
        }
    } else {
        vec![]
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
                    txt += &describe_actor(a);
                }

                GameObject::Item(_, i) => {
                    txt += &format!("\nan item ({}) on the ground\n", i.name);
                }
            }
        }
    }

    draw_dialog(cvs, assets, (txt, actions), focus_pos, viewport)
}

//////////////////////////////////////////////////
// PRIVATE HELPER
//

fn draw_dialog(
    cvs: &mut WindowCanvas,
    assets: &AssetRepo,
    content: (String, &Vec<(Action, u8)>),
    ScreenPos(focus_x, focus_y): ScreenPos,
    viewport: &Rect,
) -> Result<ClickAreas, String> {
    let (text, actions) = content;
    let font = assets.font("normal")?;
    let txt_box = font
        .text(DisplayStr::new(text))
        .width(DLG_WIDTH)
        .padding(10)
        .background(Color::RGB(252, 251, 250))
        .border(3, Color::RGB(23, 22, 21))
        .prepare();

    let mut dlg_boxes = create_action_buttons(assets, actions)?;

    dlg_boxes.insert(0, (txt_box, None));

    let focus_pos = ScreenPos(focus_x, focus_y);
    let click_areas = draw_dlg_next_to_pos(cvs, focus_pos, viewport, dlg_boxes)?;

    Ok(click_areas)
}

fn display_text((action, delay): &(Action, u8), is_first: bool) -> DisplayStr {
    let str = match action {
        Action::Wait() => format!("Wait"),
        Action::MoveTo(..) => format!("Move Here"),
        Action::Activate(_) => format!("Activate"),
        Action::MeleeAttack(_, a) => format!("{} ({})", a.name, delay),
        Action::Charge(_, _) => "Charge!".to_string(),
        Action::UseAbility(_, name, _) => format!("Use ability: {}", name),
        _ => format!("Unnamed action: {:?}", action),
    };

    let str = if is_first {
        format!("> {} <", str)
    } else {
        str
    };

    DisplayStr::new(str)
}

fn describe_actor(a: &Actor) -> String {
    let (pain, wounds) = a.health();
    let condition = match (pain, wounds) {
        (0, 0) => "perfect condition",
        (_, 0) => "unharmed but in some pain",
        (_, 1) => "wounded",
        _ => "critically wounded",
    };

    let traits_str: String = a
        .active_traits()
        .map(describe_trait)
        .collect::<Vec<_>>()
        .join("\n");

    format!("\n{} ({})\n{}", a.name, condition, traits_str)
}

fn describe_trait(t: &Trait) -> String {
    let Trait { name, source, .. } = t;

    let source_str = match source {
        TraitSource::IntrinsicProperty => "initrinsic".to_string(),
        TraitSource::Temporary(rounds_left) => format!("temporary, {} left", rounds_left),
    };

    format!("{} ({})", name.clone().into_string(), source_str)
}

fn create_action_buttons<'a>(
    assets: &'a AssetRepo,
    actions: &'a Vec<(Action, u8)>,
) -> Result<Vec<(PreparedText<'a>, Option<&'a (Action, u8)>)>, String> {
    let f = &assets.font("normal")?;
    let mut buttons = vec![];
    let mut is_first = true;

    for a in actions.iter() {
        let txt_box = f
            .text(display_text(a, is_first))
            .padding(10)
            .border(3, Color::RGB(23, 22, 21))
            .background(Color::RGB(252, 251, 250))
            .width(DLG_WIDTH)
            .prepare();

        buttons.push((txt_box, Some(a)));

        is_first = false;
    }

    Ok(buttons)
}

fn draw_dlg_next_to_pos(
    cvs: &mut WindowCanvas,
    ScreenPos(focus_x, focus_y): ScreenPos,
    viewport: &Rect,
    dlg_boxes: Vec<(PreparedText, Option<&(Action, u8)>)>,
) -> Result<ClickAreas, String> {
    let total_height: u32 = dlg_boxes
        .iter()
        .map(|(tbox, _)| tbox.dimension().1)
        .sum::<u32>()
        + (dlg_boxes.len() - 1) as u32 * SPACING;
    let mut dlg_boxes = dlg_boxes;
    let mut click_areas = Vec::new();
    let x_pos_space = 2 * SPACING as i32;
    let x_pos_right = focus_x + TILE_WIDTH as i32 + x_pos_space;
    let screen_width = viewport.width() as i32;

    let x = if x_pos_right + DLG_WIDTH as i32 >= screen_width {
        // place the dialog to the left of the focus pos
        focus_x - DLG_WIDTH as i32 - x_pos_space
    } else {
        // place the dialog to the right of the focus pos
        x_pos_right
    };

    let mut y = max(0, focus_y - (total_height as i32 - TILE_HEIGHT as i32) / 2);

    for (button, action) in dlg_boxes.drain(..) {
        let (w, h) = button.dimension();

        if let Some(action) = action {
            let action = action.clone(); // first clone so closure can take owership of "action"
            click_areas.push(ClickArea {
                clipping_area: Rect::new(x, y, w, h),
                // second clone so closure can return the action more than once (-> Fn)
                action: Box::new(move |_| UserInput::SelectAction(action.clone())),
            });
        }

        button.draw(cvs, (x, y))?;

        y = y + h as i32 + SPACING as i32;
    }

    Ok(click_areas)
}
