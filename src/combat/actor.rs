extern crate rand;

use bevy::prelude::*;

use crate::core::{
    ActionCheck, ActionEffect, ActionEffectTrigger, ActionFx, ActionKeyword, ActiveDefence,
    ActiveEffects, Attributes, Card, CheckResult, Effect, FeatType, Hand, Health, KeywordSet,
    PassiveDefence, RawActionTemplate, Resistance, SimpleAction,
};

use super::GameDeck;
use super::commands::{StartInputWorkfowCommand, UserInputWorkflow};
use super::fx::{FxEffect, FxSequence};
use super::ui::{SelectedMapPos, Z_LAYER_ACTOR};
use super::{
    map::{MapPos, Obstacle},
    ui::{Description, MapPosSelectedEvent},
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

#[derive(Component, Debug)]
pub struct Activations {
    remaining: Vec<Card>,
}

impl Activations {
    pub fn next_activation_initiative(&self) -> Option<u8> {
        self.remaining.iter().map(|card| card.value()).min()
    }

    pub fn activate_next(&mut self) -> Card {
        assert!(!self.remaining.is_empty());

        let (index, _) = self
            .remaining
            .iter()
            .enumerate()
            .min_by_key(|(_, c)| c.value())
            .unwrap();

        self.remaining.remove(index)
    }

    pub fn add_card(&mut self, card: Card) {
        self.remaining.push(card);
    }

    pub fn refresh(&mut self, new_activations: Vec<Card>) {
        self.remaining = new_activations;
    }

    pub fn remaining(&self) -> &[Card] {
        &self.remaining
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
    pub activations: Activations,
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
            // visual,
            activations: Activations { remaining: vec![] },
            // health: Health { wounds: vec![] },
            obstacle: Obstacle(f32::MAX),
            transform: Transform::from_translation(map_pos.into_vec3().with_z(Z_LAYER_ACTOR)),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::default(),
        }
    }
}

#[derive(Resource)]
pub struct PossibleUserActions {
    selected_action: usize,
    actions: Vec<(Option<MapPos>, Order)>,
}

impl PossibleUserActions {
    pub fn new(actions: Vec<(Option<MapPos>, Order)>) -> Self {
        assert!(!actions.is_empty());

        Self {
            actions,
            selected_action: 0,
        }
    }

    pub fn select_action(&mut self, new_index: usize) {
        self.selected_action = new_index.clamp(0, self.actions.len() - 1);
    }

    pub fn get_selected_action_index(&self) -> usize {
        self.selected_action
    }

    pub fn get_selected_action(&self) -> &Order {
        self.actions
            .get(self.selected_action)
            .map(|(_, action)| action)
            .unwrap()
    }

    pub fn get_selected_action_at(&self, target_pos: &MapPos) -> Option<&Order> {
        self.actions
            .get(self.selected_action)
            .and_then(|(pos, action)| {
                if pos.is_some_and(|p| p == *target_pos) {
                    Some(action)
                } else {
                    None
                }
            })
    }

    pub fn available_actions(&self) -> impl Iterator<Item = &Order> {
        self.actions.iter().map(|(_, a)| a)
    }
}

// #[derive(Debug, Event)]
// pub struct ActorDataChangedEvent(pub Entity);

// pub fn setup_actor_changed(app: &mut App) {
//     app.world_mut()
//         .register_component_hooks::<Health>()
//         .on_insert(|world, context| println!("Changed {:?}", context));
// }

#[derive(Debug, Event)]
pub struct ActionTriggeredEvent(pub Order);

#[derive(Debug, Event)]
pub struct ActionSelectedEvent(pub usize);

#[derive(Debug, EntityEvent)]
pub struct BeginActivationCommand(pub Entity);

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
    // pub effect: ActionEffects,
    // pub keywords: KeywordSet<ActionKeyword>,
    // pub attribute: AttributeType,
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
            // effect: template.effect.clone().into(),
            // keywords: KeywordSet::new(&template.keywords),
            // attribute: template.attribute,
            check: template.check.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EffectTargets(pub Vec<Entity>);

#[derive(Debug, Clone)]
pub enum Order {
    EndActivation,
    // NoOp(Entity),
    MoveAlong { actor: Entity, path: Vec<MapPos> },
    Action(ActionCommandData),
    EndPlanningPhase(Team),
    AssignActivation { actor: Entity, card_index: usize },
}

pub fn clear_user_actions_on_turn_phase_change(mut commands: Commands) {
    commands.remove_resource::<PossibleUserActions>();
}

pub fn handle_actor_activated_event(_trigger: On<ActorActivatedEvent>, mut commands: Commands) {
    commands.remove_resource::<PossibleUserActions>();
}

pub fn handle_map_pos_selected_event(
    trigger: On<MapPosSelectedEvent>,
    player_actions: Option<Res<PossibleUserActions>>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let MapPosSelectedEvent(hex) = trigger.event();

    if let (Some(player_actions), Some(hex)) = (player_actions, hex) {
        if let Some(action) = player_actions.get_selected_action_at(hex) {
            // there is already a selected action for this hex
            // => execute this action
            commands.trigger(ActionTriggeredEvent(action.clone()));

            return Ok(());
        }
    }

    commands.remove_resource::<PossibleUserActions>();

    Ok(())
}

pub fn collect_actions_for_assigning_activations(
    user_actions: Option<Res<PossibleUserActions>>,
    selected_map_pos: Option<ResMut<SelectedMapPos>>,
    target_actor_q: Query<(Entity, &MapPos, &Team), With<Actor>>,
    teams_q: Query<(Entity, &TeamReady, &Controller)>,
    hands_q: Query<&Hand>,

    mut commands: Commands,
) -> Result<(), BevyError> {
    if user_actions.is_some() {
        // The user actions are already available
        // => nothing more to do
        return Ok(());
    }

    let hex = selected_map_pos.map(|sel_map_pos| sel_map_pos.0);

    let mut available_actions = vec![];

    if let Some(hex) = hex {
        if let Some((actor, team)) = find_actor_at(&target_actor_q, hex) {
            let hand = hands_q.get(team.0)?;

            if let Some(card_index) = hand.selected_card() {
                available_actions.push((Some(hex), Order::AssignActivation { actor, card_index }));
            }
        }
    }

    for (team_entity, TeamReady(is_ready), Controller(is_pc)) in teams_q.iter() {
        if *is_pc && !*is_ready {
            available_actions.push((None, Order::EndPlanningPhase(Team(team_entity))));
        }
    }

    if !available_actions.is_empty() {
        commands.insert_resource(PossibleUserActions::new(available_actions));
    }

    Ok(())
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

fn find_actor_at(
    q_actor_at: &Query<(Entity, &MapPos, &Team), With<Actor>>,
    target_pos: MapPos,
) -> Option<(Entity, Team)> {
    for (target, map_pos, target_team) in q_actor_at.iter() {
        if *map_pos == target_pos {
            return Some((target, *target_team));
        }
    }
    None
}

pub fn handle_action_selected_event(
    trigger: On<ActionSelectedEvent>,
    selected_map_pos: Option<ResMut<PossibleUserActions>>,
) {
    if let Some(mut sel_mp) = selected_map_pos {
        let ActionSelectedEvent(new_index) = trigger.event();
        sel_mp.select_action(*new_index);
    }
}

pub fn handle_action_triggered_event(
    trigger: On<ActionTriggeredEvent>,
    active_actor_q: Query<Entity, With<Active>>,
    mut commands: Commands,
    mut team_ready_q: Query<Mut<TeamReady>>,
) -> Result<(), BevyError> {
    let ActionTriggeredEvent(action) = trigger.event();

    // info!(
    //     "[handle_action_selected_event] entity={:?}, action={:?}",
    //     entity, action,
    // );

    // Reset user actions so they can be refreshed after the action is resolved
    commands.remove_resource::<PossibleUserActions>();

    match action {
        Order::EndActivation => {
            for e in active_actor_q.iter() {
                commands.entity(e).remove::<Active>();
                commands.trigger(ActivationEndedEvent(e));
            }
        }
        Order::MoveAlong { actor, path } => {
            commands.trigger(MoveToCommand(*actor, path.clone()));
        }

        Order::Action(data) => {
            commands.trigger(ActionCommand(data.clone()));
        }

        Order::EndPlanningPhase(Team(id)) => {
            let mut team_ready = team_ready_q.get_mut(*id)?;
            team_ready.0 = true;
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
    mut actor_q: Query<(Mut<Activations>, &Team, &Name)>,
    mut team_q: Query<Mut<Hand>>,
) -> Result<(), BevyError> {
    let AssignActivationCommand { actor, card_index } = trigger.event();
    let (mut activations, Team(team), _name) = actor_q.get_mut(*actor)?;
    let mut hand = team_q.get_mut(*team)?;
    let card = hand.remove(*card_index);

    // info!(
    //     "handle_assign_activation_command - actor={}, card={:?}",
    //     _name, card
    // );

    activations.add_card(card);

    Ok(())
}

// pub fn handle_combat_finished_event(
//     trigger: On<CombatFinishedEvent>,
//     mut health_q: Query<(
//         Mut<Health>,
//         Mut<PassiveDefence>,
//         Mut<Items>,
//         Mut<ActiveEffects>,
//     )>,
// ) -> Result<(), BevyError> {
//     let CombatFinishedEvent {
//         result: combat_result,
//         ..
//     } = trigger.event();

//     for (e, c) in combat_result.consequences.iter() {
//         let (mut health, mut protection, mut items, mut active_effects) = health_q.get_mut(*e)?;

//         match c {
//             CombatConsequence::Wound { damage } => {
//                 health.damage(*damage);
//                 // for card in damage.iter() {
//                 //     health.wounds.push(*card);
//                 // }
//             }

//             CombatConsequence::OffBalance => {
//                 active_effects.add_temporary_eff(
//                     Effect::BoonOrBane(-1), // the following attacks will be more difficult
//                     keyword_set!(ActionKeyword::Action, ActionKeyword::Physical),
//                     1,
//                     "Off balance (-1 to attacks)".to_string(),
//                 );
//             }

//             CombatConsequence::Vulnerable => {
//                 active_effects.add_temporary_eff(
//                     Effect::BoonOrBane(-1), // it becomes harder to defend further attacks
//                     keyword_set!(ActionKeyword::Reaction, ActionKeyword::Physical),
//                     1,
//                     "Vulnerable (-1 to defence)".to_string(),
//                 );
//             }

//             CombatConsequence::ArmorBreak => {
//                 let mut rng = thread_rng();
//                 let protecting_item = protection
//                     .0
//                     .iter()
//                     .filter_map(|r| {
//                         if matches!(r.source.1, FeatType::Item) {
//                             Some(r.source.0.to_string())
//                         } else {
//                             None
//                         }
//                     })
//                     .choose(&mut rng);

//                 if let Some(item_name) = protecting_item {
//                     let resistance = protection
//                         .0
//                         .iter_mut()
//                         .find(|r| r.source.0 == item_name)
//                         .unwrap();

//                     resistance.resistance = resistance.resistance.checked_sub(1).unwrap_or(0);

//                     let item = items
//                         .0
//                         .iter_mut()
//                         .find(|r| r.feat_ref == item_name)
//                         .unwrap();
//                     item.state = if resistance.resistance == 0 {
//                         ItemState::Broken
//                     } else {
//                         ItemState::Damaged
//                     };
//                 }
//             } // _ => {
//               //     warn!("Not implemented yet: {:?}", c);
//               // }
//         }
//     }

//     Ok(())
// }

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
        // effect,
        // keywords,
        // attribute,
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
    mut health_q: Query<(Mut<Health>, Mut<ActiveEffects>)>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let ActionFinishedEvent { consequences, .. } = trigger.event();

    for (e, c) in consequences {
        let (mut health, mut active_effects) = health_q.get_mut(*e)?;
        match c {
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
                    ("Active Defence".into(), FeatType::Intrinsic),
                    *resistance,
                )));
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum ActionConsequence {
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

        // ActionEffect::Multi(effects) => effects
        //     .iter()
        //     .flat_map(|eff| map_action_effect(targets, eff, def_q))
        //     .collect(),
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
        _ => vec![],
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
