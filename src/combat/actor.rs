extern crate rand;

use bevy::prelude::*;
use rand::seq::IteratorRandom;
use rand::thread_rng;

use crate::core::{
    AttackOption, Attacks, AttributeType, Attributes, Card, FeatSource, Hand, Health, ItemState,
    Items, Protection,
};

use super::GameDeck;
use super::combat_resolution::{
    Attack, CombatConsequence, CombatResult, Combatant, Target, handle_attack,
};
use super::flow::{Turn, TurnPhase};
use super::fx::{FxEffect, FxSequence};
use super::ui::Z_LAYER_ACTOR;
use super::{
    map::{HexMap, MapPos, Obstacle},
    ui::{Description, MapPosSelectedEvent, SelectedMapPos, UiState, UiStateTransitionedEvent},
};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub struct Team(pub Entity);

#[derive(Component)]
pub struct TeamReady(pub bool);

#[derive(Bundle)]
pub struct TeamBundle {
    name: Name,
    hand: Hand,
    player_controlled: PlayerControlled,
    ready: TeamReady,
}
impl TeamBundle {
    pub fn new(name: impl Into<Name>, is_pc: bool) -> Self {
        Self {
            name: name.into(),
            hand: Hand::new(),
            player_controlled: PlayerControlled(is_pc),
            ready: TeamReady(!is_pc), // CPU player are always ready
        }
    }
}

#[derive(Component)]
pub struct PlayerControlled(pub bool);

#[derive(Component)]
pub struct Actor {}

impl Actor {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Component)]
pub enum AiBehaviour {
    Zombi,
}

#[derive(Debug, Clone)]
pub struct Activation(pub Card);

impl Activation {
    pub fn speed(&self) -> u8 {
        self.0.value_high()
    }
}

#[derive(Component, Debug)]
pub struct Activations {
    active: Option<Activation>,
    remaining: Vec<Activation>,
}

impl Activations {
    pub fn next_activation_initiative(&self) -> Option<u8> {
        self.remaining.iter().map(|a| a.speed()).max()
    }

    pub fn active(&self) -> Option<&Activation> {
        self.active.as_ref()
    }

    pub fn activate_next(&mut self) {
        self.active = self.remaining.pop();
    }

    pub fn add_card(&mut self, card: Card) {
        self.remaining.push(Activation(card));
        self.remaining
            .sort_unstable_by(|a1, a2| a1.speed().cmp(&a2.speed()));
    }

    pub fn refresh(&mut self, mut new_activations: Vec<Card>) {
        self.remaining = new_activations.drain(..).map(|c| Activation(c)).collect();
        self.remaining
            .sort_unstable_by(|a1, a2| a1.speed().cmp(&a2.speed()));
    }

    pub fn remaining(&self) -> &[Activation] {
        &self.remaining
    }
}

#[derive(Bundle)]
pub struct ActorBundle {
    pub actor: Actor,
    pub name: Name,
    pub description: Description,
    pub team: Team,
    pub play_controlled: PlayerControlled,
    pub map_pos: MapPos,
    // pub visual: Visual,
    pub activations: Activations,
    // pub health: Health,
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
            play_controlled: PlayerControlled(is_pc),
            map_pos,
            // visual,
            activations: Activations {
                active: None,
                remaining: vec![],
            },
            // health: Health { wounds: vec![] },
            obstacle: Obstacle(f32::MAX),
            transform: Transform::from_translation(map_pos.into_vec3().with_z(Z_LAYER_ACTOR)),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::default(),
        }
    }
}

#[derive(Debug, Event)]
pub struct ActionTriggeredEvent(pub Action);

#[derive(Debug, Event)]
pub struct ActionSelectedEvent(pub usize);

#[derive(Debug, Event)]
pub struct HandCardSelectedEvent(pub usize);

#[derive(Debug, Event)]
pub struct CombatFinishedEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub result: CombatResult,
}

#[derive(Debug, Event)]
pub struct BeginActivationCommand;

#[derive(Debug, Event)]
pub struct EndActivationCommand;

#[derive(Debug, Event)]
pub struct MoveToCommand(Vec<MapPos>);

#[derive(Debug, Event)]
pub struct AttackCommand(AttackData);

#[derive(Debug, Event)]
pub struct AssignActivationCommand {
    actor: Entity,
    card_index: usize,
}

#[derive(Debug, Clone)]
pub struct AttackData {
    pub target: AttackTarget,
    pub name: String,
    pub attribute: AttributeType,
    pub damage: u8,
    pub penetration: u8,
    pub difficulty: u8,
}

#[derive(Debug, Clone)]
pub enum AttackTarget {
    MeleeAttack { target: Entity },
}

impl AttackData {
    pub fn new(target: Entity, attack_template: &AttackOption) -> Self {
        match attack_template {
            AttackOption::MeleeAttack {
                name,
                attribute,
                damage,
                penetration,
                difficulty,
            } => Self {
                target: AttackTarget::MeleeAttack { target },
                name: name.clone(),
                attribute: *attribute,
                damage: *damage,
                penetration: *penetration,
                difficulty: *difficulty,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    NoOp,
    MoveAlong { path: Vec<MapPos> },
    Attack(AttackData),
    EndPlanningPhase(Team),
    AssignActivation { actor: Entity, card_index: usize },
}

pub fn handle_select_map_pos(
    map: Res<HexMap>,
    turn: Res<Turn>,
    player_actions: Option<Res<SelectedMapPos>>,
    active_actor_q: Query<
        (Entity, &Actor, &Activations, &MapPos, &Team, &Attacks),
        Without<AiBehaviour>,
    >,
    target_actor_q: Query<(Entity, &MapPos, &Team), With<Actor>>,
    teams_q: Query<(Entity, &TeamReady, &PlayerControlled)>,
    hands_q: Query<&Hand>,

    mut commands: Commands,
    mut map_pos_selected_er: EventReader<MapPosSelectedEvent>,
) -> Result<(), BevyError> {
    let Some(MapPosSelectedEvent(hex)) = map_pos_selected_er.read().last() else {
        return Ok(());
    };

    match turn.turn_phase {
        TurnPhase::StartTurn => {
            // Nothing to do here
            return Ok(());
        }

        TurnPhase::BoostActivations => {
            let mut available_actions: Vec<Action> = vec![];

            if let Some((actor, team)) = find_actor_at(&target_actor_q, *hex) {
                let hand = hands_q.get(team.0)?;
                if let Some(card_index) = hand.selected_card() {
                    available_actions.push(Action::AssignActivation { actor, card_index });
                }
            }

            for (team_entity, TeamReady(is_ready), PlayerControlled(is_pc)) in teams_q.iter() {
                if *is_pc && !*is_ready {
                    available_actions.push(Action::EndPlanningPhase(Team(team_entity)));
                }
            }

            commands.insert_resource(SelectedMapPos::new(None, *hex, available_actions));
        }

        TurnPhase::PerformActions => {
            if let Some((entity, actor_pos, team, attacks)) =
                find_active_player_actor(&active_actor_q)
            {
                if let Some(action) =
                    player_actions.and_then(|pa| pa.get_selected_action_when_at(hex).cloned())
                {
                    // there is already a selected action for this hex
                    // => execute this action
                    commands.remove_resource::<SelectedMapPos>();
                    commands.trigger_targets(ActionTriggeredEvent(action), entity);
                } else {
                    let mut available_actions = vec![];

                    if let Some(target) = find_enemy_at(&target_actor_q, &team, *hex) {
                        let distance = actor_pos.distance(hex);

                        for attack_option in attacks.0.iter() {
                            if attack_option.can_attack(distance) {
                                let attack = AttackData::new(target, attack_option);
                                available_actions.push(Action::Attack(attack));
                            }
                        }

                        if actor_pos.distance(hex) > 1 {
                            if let Some(mut path) = map.find_path(actor_pos, *hex) {
                                if path.len() > 1 {
                                    let max_steps = (path.len() - 1).max(3);
                                    let path = path.drain(..max_steps).collect();

                                    available_actions.push(Action::MoveAlong { path });
                                }
                            }
                        }
                    } else if let Some(path) = map.find_path(actor_pos, *hex) {
                        available_actions.push(Action::MoveAlong { path });
                    }

                    available_actions.push(Action::NoOp);

                    commands.insert_resource(SelectedMapPos::new(
                        Some(entity),
                        *hex,
                        available_actions,
                    ));
                }
            }
        }
    }

    Ok(())
}

fn find_active_player_actor(
    q_active_actor: &Query<
        (Entity, &Actor, &Activations, &MapPos, &Team, &Attacks),
        Without<AiBehaviour>,
    >,
) -> Option<(Entity, MapPos, Team, Attacks)> {
    for (entity, _actor, activations, map_pos, team, attacks) in q_active_actor.iter() {
        if activations.active.is_some() {
            return Some((entity, *map_pos, *team, attacks.clone()));
        }
    }
    None
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

fn find_enemy_at(
    q_actor_at: &Query<(Entity, &MapPos, &Team), With<Actor>>,
    attacker_team: &Team,
    target_pos: MapPos,
) -> Option<Entity> {
    find_actor_at(q_actor_at, target_pos)
        .filter(|(_, target_team)| target_team != attacker_team)
        .map(|(entity, _)| entity)
}

pub fn handle_action_selected_event(
    trigger: Trigger<ActionSelectedEvent>,
    selected_map_pos: Option<ResMut<SelectedMapPos>>,
) {
    if let Some(mut sel_mp) = selected_map_pos {
        let ActionSelectedEvent(new_index) = trigger.event();
        sel_mp.select_action(*new_index);
    }
}

pub fn handle_action_triggered_event(
    trigger: Trigger<ActionTriggeredEvent>,
    mut commands: Commands,
    mut team_ready_q: Query<Mut<TeamReady>>,
) -> Result<(), BevyError> {
    let ActionTriggeredEvent(action) = trigger.event();
    let entity = trigger.target();

    // info!(
    //     "[handle_action_selected_event] entity={:?}, action={:?}",
    //     entity, action,
    // );

    match action {
        Action::MoveAlong { path } => {
            commands.trigger_targets(MoveToCommand(path.clone()), entity);
        }

        Action::Attack(attack) => {
            commands.trigger_targets(AttackCommand(attack.clone()), entity);
        }

        Action::NoOp => {
            // Do nothing, but progress ui state so next actor can be activated
            commands.trigger(UiStateTransitionedEvent(UiState::prossing()));
        }

        Action::EndPlanningPhase(Team(id)) => {
            let mut team_ready = team_ready_q.get_mut(*id)?;
            team_ready.0 = true;
            commands.trigger(UiStateTransitionedEvent(UiState::prossing()));
        }

        Action::AssignActivation { actor, card_index } => {
            commands.trigger(AssignActivationCommand {
                actor: *actor,
                card_index: *card_index,
            });
        }
    }

    if entity != Entity::PLACEHOLDER {
        commands.trigger_targets(EndActivationCommand, entity);
    }
    Ok(())
}

pub fn handle_begin_activation_command(
    trigger: Trigger<BeginActivationCommand>,
    mut commands: Commands,
    mut actor_activation_q: Query<(Mut<Activations>, &PlayerControlled, &MapPos, &Name)>,
) -> Result<(), BevyError> {
    let e = trigger.target();
    let (mut activation, PlayerControlled(is_pc), mpos, _name) = actor_activation_q.get_mut(e)?;

    // info!(
    //     "[actor::handle_begin_activation_command] {} (entity={:?})",
    //     _name,
    //     trigger.target()
    // );

    activation.activate_next();

    if *is_pc {
        commands.trigger(UiStateTransitionedEvent(UiState::await_input(*mpos)));
    }

    Ok(())
}

pub fn handle_end_activation_command(
    trigger: Trigger<EndActivationCommand>,
    mut activations_q: Query<Mut<Activations>>,
) -> Result<(), BevyError> {
    // info!(
    //     "handle_end_activation_command - entity={:?}",
    //     trigger.target()
    // );

    let mut activations = activations_q.get_mut(trigger.target())?;
    activations.active = None;
    Ok(())
}

pub fn handle_move_to_command(trigger: Trigger<MoveToCommand>, mut commands: Commands) {
    let MoveToCommand(path) = trigger.event();

    if path.is_empty() {
        // No need to move along an empty Path
        return;
    }

    let end_pos = *path.last().unwrap();
    let moving_entity = trigger.target();

    commands.entity(moving_entity).insert(end_pos);

    FxSequence::new()
        .then(FxEffect::walk_along(moving_entity, path))
        .wait_until_finished()
        .run(&mut commands);
}

pub fn handle_attack_command(
    trigger: Trigger<AttackCommand>,
    combat_data_q: Query<(&Health, &Activations, &Attributes, &Protection)>,
    mut deck: ResMut<GameDeck>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let AttackCommand(attack) = trigger.event();
    let attacker = trigger.target();

    let combat_finished_event = match &attack.target {
        AttackTarget::MeleeAttack { target } => {
            // info!(
            //     "[handle_attack_command] attacker={:?}, target={:?}",
            //     attacking_entity, target
            // );

            let [
                (health_a, activation_a, attributes_a, protection_a),
                (health_t, _, attributes_t, protection_t),
            ] = combat_data_q.get_many([attacker, *target])?;

            let effort_card = activation_a.active.as_ref().cloned().unwrap().0;

            let combat_result = handle_attack(
                Attack {
                    speed: effort_card,
                    attribute: attack.attribute,
                    damage: attack.damage,
                    penetration: attack.penetration,
                    difficulty: attack.difficulty,
                    attacker: Combatant {
                        id: attacker,
                        attributes: *attributes_a,
                        health: health_a.clone(),
                        protection: protection_a.clone(),
                    },
                    target: Target::SingleMelee(Combatant {
                        id: *target,
                        attributes: *attributes_t,
                        health: health_t.clone(),
                        protection: protection_t.clone(),
                    }),
                },
                &mut deck.0,
            );

            CombatFinishedEvent {
                attacker,
                target: *target,
                result: combat_result,
            }
        }
    };

    commands.trigger(combat_finished_event);

    Ok(())
}

pub fn handle_assign_activation_command(
    trigger: Trigger<AssignActivationCommand>,
    mut commands: Commands,
    mut actor_q: Query<(Mut<Activations>, &Team, &Name)>,
    mut team_q: Query<Mut<Hand>>,
) -> Result<(), BevyError> {
    let AssignActivationCommand { actor, card_index } = trigger.event();
    let (mut activations, Team(team), name) = actor_q.get_mut(*actor)?;
    let mut hand = team_q.get_mut(*team)?;
    let card = hand.remove(*card_index);

    info!(
        "handle_assign_activation_command - actor={}, card={:?}",
        name, card
    );

    activations.add_card(card);
    commands.trigger(UiStateTransitionedEvent(UiState::prossing()));

    Ok(())
}

pub fn handle_combat_finished_event(
    trigger: Trigger<CombatFinishedEvent>,
    mut health_q: Query<(Mut<Health>, Mut<Protection>, Mut<Items>)>,
) -> Result<(), BevyError> {
    let CombatFinishedEvent {
        result: combat_result,
        ..
    } = trigger.event();

    for (e, c) in combat_result.iter() {
        let (mut health, mut protection, mut items) = health_q.get_mut(*e)?;
        match c {
            CombatConsequence::Wound { damage } => {
                for card in damage.iter() {
                    health.wounds.push(*card);
                }
            }

            CombatConsequence::ArmorBreak => {
                let mut rng = thread_rng();
                let protecting_item = protection
                    .0
                    .iter()
                    .filter_map(|r| {
                        if matches!(r.source.1, FeatSource::Item) {
                            Some(r.source.0.to_string())
                        } else {
                            None
                        }
                    })
                    .choose(&mut rng);

                if let Some(item_name) = protecting_item {
                    let resistance = protection
                        .0
                        .iter_mut()
                        .find(|r| r.source.0 == item_name)
                        .unwrap();

                    resistance.resistance = resistance.resistance.checked_sub(1).unwrap_or(0);

                    let item = items.0.iter_mut().find(|r| r.key == item_name).unwrap();
                    item.state = if resistance.resistance == 0 {
                        ItemState::Broken
                    } else {
                        ItemState::Damaged
                    };
                }
            }
            _ => {}
        }
    }

    Ok(())
}
