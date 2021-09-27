use crate::core::{
    Action, Actor, CombatData, CombatState, DisplayStr, GameObject, InputContext, Trait,
    TraitSource, UserInput, WorldPos, Health
};
use crate::ui::types::{ClickArea, ClickAreas, Scene, ScreenPos, ScreenText};

const DLG_WIDTH: u32 = 400;
const BTN_HEIGHT: u32 = 65;

pub fn render(
    scene: &mut Scene,
    click_areas: &mut ClickAreas,
    viewport: (u32, u32),
    game: &CombatData,
) {
    if let CombatState::WaitForUserAction(_, ctxt) = &game.state {
        match ctxt {
            Some(InputContext::SelectedArea(p, objects, actions)) => {
                draw_area_details(scene, viewport.0, *p, objects);
                draw_action_buttons(scene, click_areas, game, viewport, Some(actions));
            }

            _ => {
                draw_action_buttons(scene, click_areas, game, viewport, None);
            }
        }
    };
}

fn draw_area_details(
    scene: &mut Scene,
    viewport_width: u32,
    pos: WorldPos,
    objects: &Vec<GameObject>,
) {
    let mut txt = format!("You look at ({}, {}).", pos.x(), pos.y());

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

    let x = (viewport_width - DLG_WIDTH) as i32;

    scene.texts.push(
        // scene.texts[FontFace::Normal as usize].push(
        ScreenText::new(DisplayStr::new(txt), ScreenPos(x, 0))
            .width(DLG_WIDTH)
            .padding(10)
            .background((252, 251, 250, 255))
            .border(3, (23, 22, 21, 255)),
    );
}

fn draw_action_buttons(
    scene: &mut Scene,
    click_areas: &mut ClickAreas,
    game: &CombatData,
    (viewport_width, viewport_height): (u32, u32),
    actions: Option<&Vec<(Action, u8)>>,
) {
    let mut action_buttons = create_action_buttons(game, actions);
    let x = (viewport_width - DLG_WIDTH) as i32;
    let mut y = (viewport_height - action_buttons.len() as u32 * BTN_HEIGHT) as i32;

    for (text, action) in action_buttons.drain(..) {
        scene.texts.push(
            // scene.texts[FontFace::Normal as usize].push(
            ScreenText::new(text, ScreenPos(x, y))
                .padding(10)
                .border(3, (23, 22, 21, 255))
                .background((252, 251, 250, 255))
                .width(DLG_WIDTH),
        );

        click_areas.push(ClickArea {
            clipping_area: (x, y, DLG_WIDTH, BTN_HEIGHT),
            action: Box::new(move |_| UserInput::SelectAction(action.clone())),
        });

        y += BTN_HEIGHT as i32;
    }
}

//////////////////////////////////////////////////
// PRIVATE HELPER
//

fn display_text((action, _): &(Action, u8), is_first: bool) -> DisplayStr {
    let str = match action {
        Action::Done(_) => format!("Do nothing"),
        Action::MoveTo(..) => format!("Move Here"),
        Action::Activate(_) => format!("Activate"),
        Action::MeleeAttack(_, a, _) => format!("{}", a.name),
        Action::RangeAttack(_, a, _, _) => format!("{}", a.name),
        Action::Charge(..) => "Charge!".to_string(),
        Action::Dodge(..) => "Dodge".to_string(),
        Action::UseAbility(_, _, t) => format!("Use ability: {}", t.name),
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
    let Health { pain, wounds, ..} = a.health();
    let condition = match (pain, wounds.0) {
        (0, 0) => "perfect condition",
        (_, 0) => "unharmed but in some pain",
        (_, 1) => "wounded",
        _ => "critically wounded",
    };

    let action_str = match &a.pending_action {
        Some((Action::MeleeAttack(_, attack, name), _)) => format!("{} at {}", attack.name, name),

        Some((Action::RangeAttack(_, attack, _, name), _)) => format!("{} at {}", attack.name, name),

        Some((Action::Charge(_, _, name), _)) => format!("Charging at {}", name),

        Some((Action::Done(msg), _)) => format!("{}", msg),

        None => "Waiting for instructions...".to_string(),

        _ => "".to_string(),
    };

    let traits_str: String = a
        .active_traits()
        .map(describe_trait)
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "\n{} ({})\n\n{}\n\n{}",
        a.name, condition, action_str, traits_str
    )
}

fn describe_trait(t: &Trait) -> String {
    let Trait { name, source, .. } = t;

    let source_str = match source {
        TraitSource::IntrinsicProperty => "initrinsic".to_string(),
        TraitSource::Temporary(rounds_left) => format!("temporary, {} left", rounds_left),
    };

    format!("{} ({})", name.clone().into_string(), source_str)
}

fn create_action_buttons(
    game: &CombatData,
    actions: Option<&Vec<(Action, u8)>>,
) -> Vec<(DisplayStr, (Action, u8))> {
    let mut result = vec![];
    let mut is_first = true;

    if let Some(actions) = actions {
        for a in actions.iter() {
            result.push((display_text(a, is_first), a.clone()));

            is_first = false;
        }
    }

    result.push((
        DisplayStr::new(format!("End Turn {}", game.turn)),
        Action::end_turn(game.active_team()),
    ));

    result
}
