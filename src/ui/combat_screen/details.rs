use crate::core::{
    Action, Actor, Card, CombatData, CombatState, DisplayStr, GameObject, Health, InputContext,
    MapPos, SelectedPos, SuiteSubstantiality, TeamId, Trait, TraitSource, UserInput, ID,
};
use crate::ui::types::{ClickArea, ClickAreas, Scene, ScreenPos, ScreenText};

const DLG_WIDTH: u32 = 400;
const BTN_HEIGHT: u32 = 65;
const CARD_WIDTH: u32 = 120;
const CARD_HEIGHT: u32 = 150;

pub fn render(
    scene: &mut Scene,
    click_areas: &mut ClickAreas,
    viewport: (u32, u32),
    game: &CombatData,
) {
    if let CombatState::WaitForUserInput(ctxt, selected_pos) = &game.state {
        if let Some(SelectedPos { pos, objects }) = selected_pos {
            draw_area_details(scene, viewport.0, *pos, objects);

            if let InputContext::SelectAction { options, .. } = ctxt {
                let actions = options.get(pos);
                draw_action_buttons(scene, click_areas, game, viewport, actions);
            }
        }

        if let InputContext::ActivateActor {
            hand,
            selected_card_idx,
            team,
            ..
        } = ctxt
        {
            draw_cards(
                scene,
                click_areas,
                viewport,
                hand,
                selected_card_idx,
                selected_actor_at(&selected_pos),
                *team,
            );

            draw_ready_button(scene, click_areas, viewport, *team);
        }
    };
}

fn selected_actor_at(selected_pos: &Option<SelectedPos>) -> Option<ID> {
    if let Some(SelectedPos { objects, .. }) = selected_pos {
        for go in objects.iter() {
            if let GameObject::Actor(a) = go {
                return Some(a.id);
            }
        }
    }
    None
}

fn draw_area_details(
    scene: &mut Scene,
    viewport_width: u32,
    MapPos(x, y): MapPos,
    objects: &Vec<GameObject>,
) {
    let mut txt = format!("You look at ({}, {}).", x, y);

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
    actions: Option<&Vec<Action>>,
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
            action: Box::new(move |_| UserInput::SelectPlayerAction(action.clone())),
        });

        y += BTN_HEIGHT as i32;
    }
}

fn draw_cards(
    scene: &mut Scene,
    click_areas: &mut ClickAreas,
    (_viewport_width, viewport_height): (u32, u32),
    hand: &Vec<Card>,
    selected_card_idx: &Option<usize>,
    selected_actor: Option<ID>,
    team: TeamId,
) {
    for (idx, card) in hand.iter().enumerate() {
        let is_selected = selected_card_idx
            .map(|sel_idx| sel_idx == idx)
            .unwrap_or(false);

        let offset_y = if is_selected { -30 } else { -10 };
        let x = 50 + idx as i32 * (CARD_WIDTH as i32 + 15);
        let y = viewport_height as i32 - CARD_HEIGHT as i32 + offset_y;

        draw_card(scene, card, ScreenPos(x, y));

        if is_selected {
            if let Some(selected_actor_id) = selected_actor {
                let card = *card;

                click_areas.push(ClickArea {
                    clipping_area: (x, y, CARD_WIDTH, CARD_HEIGHT),
                    action: Box::new(move |_| {
                        UserInput::AssigneActivation(selected_actor_id, team, card)
                    }),
                });
            }
        } else {
            click_areas.push(ClickArea {
                clipping_area: (x, y, CARD_WIDTH, CARD_HEIGHT),
                action: Box::new(move |_| UserInput::SelectActivationCard(idx)),
            });
        }
    }
}

fn draw_ready_button(
    scene: &mut Scene,
    click_areas: &mut ClickAreas,
    (viewport_width, viewport_height): (u32, u32),
    team: TeamId,
) {
    let x = (viewport_width - DLG_WIDTH) as i32;
    let y = (viewport_height - BTN_HEIGHT) as i32;

    scene.texts.push(
        // scene.texts[FontFace::Normal as usize].push(
        ScreenText::new(DisplayStr::new("Ready"), ScreenPos(x, y))
            .padding(10)
            .border(3, (23, 22, 21, 255))
            .background((252, 251, 250, 255))
            .width(DLG_WIDTH),
    );

    click_areas.push(ClickArea {
        clipping_area: (x, y, DLG_WIDTH, BTN_HEIGHT),
        action: Box::new(move |_| UserInput::AssigneActivationDone(team)),
    });
}

//////////////////////////////////////////////////
// PRIVATE HELPER
//

fn draw_card(scene: &mut Scene, card: &Card, pos: ScreenPos) {
    let color = if let SuiteSubstantiality::Physical = card.suite.substantiality() {
        (23, 22, 21, 255)
    } else {
        (253, 22, 21, 255)
    };

    scene.texts.push(
        ScreenText::new(DisplayStr::new(format_card(card)), pos)
            .width(CARD_WIDTH)
            .height(CARD_HEIGHT)
            .padding(5)
            .background((253, 252, 251, 255))
            .color(color)
            .border(5, color),
    );
}

fn button_text_for_player_actions(action: &Action, is_first: bool) -> DisplayStr {
    let str = match action {
        Action::DoNothing(..) => format!("Do nothing"),
        Action::MoveTo { .. } => format!("Move Here"),
        Action::Attack { attack, .. } => format!("{}", attack.name),
        Action::AddTrait { msg, .. } => msg.clone(),
        Action::ActivateActor(..) => format!("Activate"),
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
    let Health {
        pain,
        recieved_wounds,
        max_wounds,
        ..
    } = a.health;
    let condition = if a.is_alive() {
        if recieved_wounds == 0 {
            if pain > 0 {
                "in pain"
            } else {
                "unharmed"
            }
        } else {
            if recieved_wounds as f32 / max_wounds as f32 > 0.5 {
                "seriously wounded"
            } else {
                "wounded"
            }
        }
    } else {
        "dead"
    };

    let active_activation_str = a
        .active_activation
        .as_ref()
        .map(format_card)
        .unwrap_or(" - ".to_string());
    let activation_str = a
        .activations
        .iter()
        .map(format_card)
        .collect::<Vec<_>>()
        .join(" ");

    let traits_str: String = a
        .active_traits()
        .map(describe_trait)
        .collect::<Vec<_>>()
        .join("\n  - ");

    format!(
        "\n{} (condition: {})\n\nActivations: \n {}\n <{}>\n\nTraits:\n - {}",
        a.name, condition, active_activation_str, activation_str, traits_str
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

fn format_card(card: &Card) -> String {
    let value = match card.value {
        1 => "A".to_string(),
        11 => "J".to_string(),
        12 => "Q".to_string(),
        13 => "K".to_string(),
        _ => format!("{}", card.value),
    };

    format!("[{} of {:?}]", value, card.suite)
}

fn create_action_buttons(
    _game: &CombatData,
    actions: Option<&Vec<Action>>,
) -> Vec<(DisplayStr, Action)> {
    let mut result = vec![];
    let mut is_first = true;

    if let Some(actions) = actions {
        for a in actions.iter() {
            result.push((button_text_for_player_actions(&a, is_first), a.clone()));
            is_first = false;
        }
    }

    // TODO clarify what should happen when clicking "End Turn"
    // result.push((
    //     DisplayStr::new(format!("End Turn {}", game.turn.turn_number)),
    //     Action::EndTurn(game.turn.get_active_team().team.clone()),
    // ));

    result
}
