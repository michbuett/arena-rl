use bevy::prelude::*;
use rand::prelude::*;

use crate::combat::commands::ManeuverSpeed;
use crate::combat::flow::Turn;
use crate::core::{
    ActionCheck, ActionEffect, ActionEffectTrigger, ActionFx, ActionKeyword, ActiveDefence,
    ActiveEffects, Attributes, Card, CheckResult, Effect, FeatType, Hand, Health, ItemState, Items,
    KeywordSet, PassiveDefence, RawActionTemplate, Resistance, ResistanceSource, SimpleAction,
    Suite,
};

use super::GameDeck;
use super::commands::{StartInputWorkfowCommand, UserInputWorkflow};
use super::fx::{FxEffect, FxSequence};
use super::ui::Z_LAYER_ACTOR;
use super::{
    map::{MapPos, Obstacle},
    ui::Description,
};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub struct Team(pub Entity);

#[derive(Component)]
pub struct TeamReady(pub bool);

#[derive(Bundle)]
pub struct TeamBundle {
    name: Name,
    hand: Hand,
    player_controlled: Controller,
    ready: TeamReady,
}

impl TeamBundle {
    pub fn new(name: impl Into<Name>, is_pc: bool) -> Self {
        Self {
            name: name.into(),
            hand: Hand::new(),
            player_controlled: Controller(is_pc),
            ready: TeamReady(!is_pc), // CPU player are always ready
        }
    }
}

#[derive(Component)]
pub struct Controller(pub bool);

impl Controller {
    pub fn is_pc(&self) -> bool {
        self.0
    }
}

#[derive(Component)]
pub struct Actor {}

impl Actor {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Component)]
pub struct Active(pub Card);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Initiative(u8);

impl From<Card> for Initiative {
    fn from(card: Card) -> Self {
        let face_val = (card.value() - 1) * 4;
        let suite_val = match card.suite() {
            Suite::Spades => 0,
            Suite::Diamonds => 1,
            Suite::Hearts => 2,
            Suite::Clubs => 3,
        };
        Self(face_val + suite_val)
    }
}

impl Initiative {
    pub fn start() -> Self {
        Initiative(0)
    }

    pub fn face_value(self) -> u8 {
        self.0 / 4 + 1
    }

    pub fn suite(&self) -> Suite {
        match self.0 % 4 {
            0 => Suite::Spades,
            1 => Suite::Diamonds,
            2 => Suite::Hearts,
            3 => Suite::Clubs,
            _ => panic!("Unreachable"),
        }
    }
}

#[derive(Component, Debug)]
pub struct Activations {
    next: Option<(u64, Card)>,
}

impl Activations {
    pub fn new(initial_activation: Card) -> Self {
        Self {
            next: Some((0, initial_activation)),
        }
    }

    pub fn next_activation_initiative(&self, current_turn: u64) -> Option<Initiative> {
        self.next.and_then(|(turn, card)| {
            if turn <= current_turn {
                Some(card.into())
            } else {
                None
            }
        })
    }

    pub fn activate_next(&mut self) -> Card {
        assert!(self.next.is_some());
        let next = self.next.unwrap().1;
        self.next = None;
        next
    }

    pub fn set_next(&mut self, current_turn: &Turn, new_activation: Card) {
        let turn_number = if current_turn.1 >= new_activation.into() {
            current_turn.0 + 1
        } else {
            current_turn.0
        };
        self.next = Some((turn_number, new_activation));
    }

    pub fn next(&self) -> Option<(u64, Card)> {
        self.next
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct RiskComplications;

#[derive(Bundle)]
pub struct ActorBundle {
    pub actor: Actor,
    pub name: Name,
    pub description: Description,
    pub team: Team,
    pub play_controlled: Controller,
    pub map_pos: MapPos,
    pub obstacle: Obstacle,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
}

impl ActorBundle {
    pub fn new(name: Name, team: Team, is_pc: bool, map_pos: MapPos) -> Self {
        Self {
            actor: Actor::new(),
            name,
            description: Description::new("Some description for an actor"),
            team,
            play_controlled: Controller(is_pc),
            map_pos,
            obstacle: Obstacle(f32::MAX),
            transform: Transform::from_translation(map_pos.into_vec3().with_z(Z_LAYER_ACTOR)),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::default(),
        }
    }
}

#[derive(Debug, Event)]
pub struct ActionTriggeredEvent(pub Order);

#[derive(Debug, EntityEvent)]
pub struct BeginActivationCommand(pub Entity);

#[derive(Debug, EntityEvent)]
pub struct EndActivationCommand {
    #[event_target]
    pub actor: Entity,
    pub maneuver_type: ManeuverSpeed,
}

#[derive(Debug, Event)]
pub struct ActivationEndedEvent(pub Entity);

#[derive(Debug, Event)]
pub struct ActorActivatedEvent(pub Entity);

#[derive(Debug, Event)]
pub struct MoveToCommand(Entity, Vec<MapPos>);

#[derive(Debug, Event)]
pub struct ActionCommand(ActionCommandData);

#[derive(Debug, Event)]
pub struct ActionFinishedEvent {
    pub actor: Entity,
    pub targets: EffectTargets,
    pub result: CheckResult,
    pub consequences: Vec<(Entity, ActionConsequence)>,
    pub action_name: String,
    pub fx: ActionFx,
}

#[derive(Debug, Event)]
pub struct AssignActivationCommand {
    actor: Entity,
    card_index: usize,
}

#[derive(Debug, Clone)]
pub struct ActionCommandData {
    pub actor: Entity,
    pub name: String,
    pub targets: EffectTargets,
    pub fx: ActionFx,
    pub check: ActionCheck,
}

impl ActionCommandData {
    pub fn from_template(
        actor: Entity,
        targets: Vec<Entity>,
        template: &RawActionTemplate,
    ) -> Self {
        Self {
            actor,
            targets: EffectTargets(targets),
            name: template.name.clone(),
            fx: template.fx.clone(),
            check: template.check.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EffectTargets(pub Vec<Entity>);

#[derive(Debug, Clone)]
pub enum Order {
    EndActivation {
        actor: Entity,
        maneuver_speed: ManeuverSpeed,
    },
    MoveAlong {
        actor: Entity,
        path: Vec<MapPos>,
    },
    Action(ActionCommandData),
    AssignActivation {
        actor: Entity,
        card_index: usize,
    },
}

pub fn on_actor_activated_event(
    trigger: On<ActorActivatedEvent>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let ActorActivatedEvent(entity) = trigger.event();
    let input_workflow = UserInputWorkflow::new(*entity);

    commands.insert_resource(input_workflow);
    commands.trigger(StartInputWorkfowCommand);

    Ok(())
}

pub fn handle_action_triggered_event(
    trigger: On<ActionTriggeredEvent>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let ActionTriggeredEvent(action) = trigger.event();

    // info!(
    //     "[handle_action_selected_event] entity={:?}, action={:?}",
    //     entity, action,
    // );

    match action {
        Order::EndActivation {
            actor,
            maneuver_speed,
        } => {
            commands.trigger(EndActivationCommand {
                actor: *actor,
                maneuver_type: *maneuver_speed,
            });
        }

        Order::MoveAlong { actor, path } => {
            commands.trigger(MoveToCommand(*actor, path.clone()));
        }

        Order::Action(data) => {
            commands.trigger(ActionCommand(data.clone()));
        }

        Order::AssignActivation { actor, card_index } => {
            commands.trigger(AssignActivationCommand {
                actor: *actor,
                card_index: *card_index,
            });
        }
    }

    Ok(())
}

pub fn handle_begin_activation_command(
    trigger: On<BeginActivationCommand>,
    mut commands: Commands,
    mut actor_activation_q: Query<(Mut<Activations>, &Name)>,
) -> Result<(), BevyError> {
    let BeginActivationCommand(e) = trigger.event();
    let (mut activation, _name) = actor_activation_q.get_mut(*e)?;

    // info!(
    //     "[actor::handle_begin_activation_command] {} (entity={:?})",
    //     _name,
    //     trigger.target()
    // );

    let activation = activation.activate_next();

    commands.entity(*e).insert(Active(activation));
    commands.trigger(ActorActivatedEvent(*e));

    Ok(())
}
pub fn handle_end_activation_command(
    trigger: On<EndActivationCommand>,
    current_turn: Res<Turn>,
    mut commands: Commands,
    mut activations_q: Query<Mut<Activations>>,
    mut deck: ResMut<GameDeck>,
) -> Result<(), BevyError> {
    let EndActivationCommand {
        actor,
        maneuver_type,
    } = trigger.event();
    let mut activations = activations_q.get_mut(*actor)?;

    match maneuver_type {
        ManeuverSpeed::Free => {
            // A free action does not really end an activation
            // => just start a new input workflow
            commands.trigger(ActorActivatedEvent(*actor));
        }

        ManeuverSpeed::Normal => {
            if activations.next().is_none() {
                activations.set_next(&current_turn, deck.0.deal());
            }
            commands.entity(*actor).remove::<Active>();
            commands.trigger(ActivationEndedEvent(*actor));
        }
    }

    Ok(())
}

pub fn handle_move_to_command(trigger: On<MoveToCommand>, mut commands: Commands) {
    let MoveToCommand(actor, path) = trigger.event();

    if path.is_empty() {
        // No need to move along an empty Path
        return;
    }

    let end_pos = *path.last().unwrap();

    commands.entity(*actor).insert(end_pos);

    FxSequence::new()
        .then(FxEffect::walk_along(*actor, path))
        .wait_until_finished()
        .run(&mut commands);
}

pub fn handle_assign_activation_command(
    trigger: On<AssignActivationCommand>,
    current_turn: Res<Turn>,
    mut actor_q: Query<(Mut<Activations>, &Team, &Name)>,
    mut team_q: Query<Mut<Hand>>,
) -> Result<(), BevyError> {
    let AssignActivationCommand { actor, card_index } = trigger.event();
    let (mut activations, Team(team), _name) = actor_q.get_mut(*actor)?;
    let mut hand = team_q.get_mut(*team)?;
    let hand_card = hand.remove(*card_index);

    // info!("handle_assign_activation_command - actor={_name}, card={hand_card:?}",);
    activations.set_next(&current_turn, hand_card);
    Ok(())
}

pub fn handle_action_command(
    trigger: On<ActionCommand>,
    actor_q: Query<(
        &Attributes,
        &ActiveEffects,
        Option<&Active>,
        Option<&RiskComplications>,
    )>,
    def_q: Query<(&PassiveDefence, Option<&ActiveDefence>)>,
    mut deck: ResMut<GameDeck>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let ActionCommand(ActionCommandData {
        actor,
        targets,
        name,
        fx,
        check,
    }) = trigger.event();

    let (attributes, active_effects, activation, risky_stance) = actor_q.get(*actor)?;
    let (action_result, effects) = match check {
        ActionCheck::NoCheck(eff) => (CheckResult::no_check(), eff),
        ActionCheck::Check {
            effects,
            risky,
            attribute,
            keywords,
        } => {
            let mut action_check = SimpleAction {
                attribute: *attribute,
                risk_complication: *risky || risky_stance.is_some(),
                attributes: *attributes,
            }
            .into_check(active_effects.for_action(*keywords));

            if let Some(Active(effort)) = activation {
                action_check = action_check.effort(*effort);
            }

            let action_result = action_check.perform_check(&mut deck.0);
            (action_result, effects)
        }
    };

    let consequences = effects
        .effects_for_result(&action_result)
        .iter()
        .flat_map(|(tr, eff)| map_action_effect(*actor, targets, *tr, &eff, def_q))
        .collect();

    commands.trigger(ActionFinishedEvent {
        actor: *actor,
        targets: targets.clone(),
        action_name: name.clone(),
        consequences,
        result: action_result,
        fx: fx.clone(),
    });

    Ok(())
}

pub fn handle_action_finished_event(
    trigger: On<ActionFinishedEvent>,
    mut health_q: Query<(
        Mut<Health>,
        Mut<ActiveEffects>,
        Mut<PassiveDefence>,
        Mut<Items>,
    )>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let ActionFinishedEvent { consequences, .. } = trigger.event();

    for (e, c) in consequences {
        let (mut health, mut active_effects, mut protection, mut items) = health_q.get_mut(*e)?;
        match c {
            ActionConsequence::ArmorBreak => {
                let mut rng = rand::thread_rng();
                let protecting_item = protection
                    .0
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, r)| {
                        if let ResistanceSource::Feat(key, FeatType::Item) = r.source {
                            Some((idx, key))
                        } else {
                            None
                        }
                    })
                    .choose(&mut rng);

                if let Some((idx, item_feat_key)) = protecting_item {
                    let resistance = protection.0.get_mut(idx).unwrap();

                    resistance.resistance = resistance.resistance.checked_sub(1).unwrap_or(0);

                    let item = items.0.iter_mut().find(|r| r.key == item_feat_key).unwrap();
                    item.state = if resistance.resistance == 0 {
                        ItemState::Broken
                    } else {
                        ItemState::Damaged
                    };
                }
            }

            ActionConsequence::Effect {
                effect,
                keywords,
                turns,
                descr,
            } => {
                active_effects.add_temporary_eff(effect.clone(), *keywords, *turns, descr.clone());
            }
            ActionConsequence::Damage(damage_details) => {
                commands.entity(*e).remove::<ActiveDefence>();

                if damage_details.actual_damage > 0 {
                    health.damage(damage_details.actual_damage);
                }
            }

            ActionConsequence::Protection(resistance) => {
                commands.entity(*e).insert(ActiveDefence(Resistance::new(
                    ResistanceSource::ActiveDefence,
                    *resistance,
                )));
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum ActionConsequence {
    ArmorBreak,
    Damage(DamageDetails),
    Protection(u8),
    Effect {
        effect: Effect,
        keywords: KeywordSet<ActionKeyword>,
        turns: u8,
        descr: String,
    },
}

fn map_action_effect(
    actor: Entity,
    action_targets: &EffectTargets,
    trigger: ActionEffectTrigger,
    eff: &ActionEffect,
    def_q: Query<(&PassiveDefence, Option<&ActiveDefence>)>,
) -> Vec<(Entity, ActionConsequence)> {
    let consequence_targets = if matches!(trigger, ActionEffectTrigger::Complication) {
        // a complication always targets the one who did the action
        &vec![actor]
    } else {
        &action_targets.0
    };

    match eff {
        ActionEffect::ArmorBreak => consequence_targets
            .iter()
            .map(|e| (*e, ActionConsequence::ArmorBreak))
            .collect(),

        ActionEffect::Protection(p) => consequence_targets
            .iter()
            .map(|e| (*e, ActionConsequence::Protection(*p)))
            .collect(),

        ActionEffect::DamageTarget(dmg) => consequence_targets
            .iter()
            .map(|e| {
                let (passive_def, active_def) = def_q.get(*e).unwrap();
                let dmg_details = damage_calculation(*dmg, active_def, passive_def);
                (*e, ActionConsequence::Damage(dmg_details))
            })
            .collect(),

        ActionEffect::TempEffect {
            effect,
            descr,
            keywords,
            turns,
        } => consequence_targets
            .iter()
            .map(|e| {
                (
                    *e,
                    ActionConsequence::Effect {
                        effect: effect.clone(),
                        keywords: KeywordSet::new(keywords),
                        turns: *turns,
                        descr: descr.clone(),
                    },
                )
            })
            .collect(),
    }
}

fn damage_calculation(
    max_damage: u8,
    active_def: Option<&ActiveDefence>,
    passive_def: &PassiveDefence,
) -> DamageDetails {
    let defence = active_def.as_ref().map(|d| d.0.resistance).unwrap_or(0);
    let armor = passive_def.total_resistance();
    let actual_damage = max_damage
        .checked_sub(defence)
        .unwrap_or(0)
        .checked_sub(armor)
        .unwrap_or(0);

    DamageDetails {
        max_damage,
        defence,
        armor,
        actual_damage,
    }
}

#[derive(Debug)]
pub struct DamageDetails {
    pub max_damage: u8,
    pub defence: u8,
    pub armor: u8,
    pub actual_damage: u8,
}
