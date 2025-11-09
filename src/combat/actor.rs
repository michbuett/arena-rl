extern crate rand;

use bevy::prelude::*;
use rand::seq::IteratorRandom;
use rand::thread_rng;

use crate::core::{
    AttackData, AttackOption, AttackTargetType, Attacks, Attributes, Card, FeatSource, Hand,
    Health, ItemState, Items, Protection,
};

use super::GameDeck;
use super::combat_resolution::{
    Attack, CombatConsequence, CombatResult, Combatant, Target, handle_attack,
};
use super::fx::{FxEffect, FxSequence};
use super::ui::{SelectedMapPos, Z_LAYER_ACTOR};
use super::{
    map::{HexMap, MapPos, Obstacle},
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
        self.remaining.iter().map(|card| card.value_low()).min()
    }

    pub fn activate_next(&mut self) -> Card {
        assert!(!self.remaining.is_empty());

        let (index, _) = self
            .remaining
            .iter()
            .enumerate()
            .min_by_key(|(_, c)| c.value_low())
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
    actions: Vec<(Option<MapPos>, Action)>,
}

impl PossibleUserActions {
    pub fn new(actions: Vec<(Option<MapPos>, Action)>) -> Self {
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

    pub fn get_selected_action(&self) -> &Action {
        self.actions
            .get(self.selected_action)
            .map(|(_, action)| action)
            .unwrap()
    }

    pub fn get_selected_action_at(&self, target_pos: &MapPos) -> Option<&Action> {
        self.actions
            .get(self.selected_action)
            .map(|(pos, action)| {
                if pos.is_some_and(|p| p == *target_pos) {
                    Some(action)
                } else {
                    None
                }
            })
            .flatten()
    }

    pub fn available_actions(&self) -> impl Iterator<Item = &Action> {
        self.actions.iter().map(|(_, a)| a)
    }
}

#[derive(Debug, Event)]
pub struct ActionTriggeredEvent(pub Action);

#[derive(Debug, Event)]
pub struct ActionSelectedEvent(pub usize);

#[derive(Debug, Event)]
pub struct CombatFinishedEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub result: CombatResult,
}

#[derive(Debug, EntityEvent)]
pub struct BeginActivationCommand(pub Entity);

#[derive(Debug, Event)]
pub struct ActivationEndedEvent(pub Entity);

#[derive(Debug, Event)]
pub struct ActorActivatedEvent(pub Entity);

#[derive(Debug, Event)]
pub struct MoveToCommand(Entity, Vec<MapPos>);

#[derive(Debug, Event)]
pub struct AttackCommand(AttackCommandData);

#[derive(Debug, Event)]
pub struct AssignActivationCommand {
    actor: Entity,
    card_index: usize,
}

#[derive(Debug, Clone)]
pub struct AttackCommandData {
    pub attacker: Entity,
    pub target: AttackTarget,
    pub name: String,
    pub activation: Card,
    pub data: AttackData,
}

#[derive(Debug, Clone)]
pub enum AttackTarget {
    MeleeAttack { target: Entity },
}

impl AttackCommandData {
    pub fn new(
        attacker: Entity,
        target: Entity,
        attack_template: &AttackOption,
        activation: Card,
    ) -> Self {
        let target = match attack_template.target_type {
            AttackTargetType::MeleeSingle => AttackTarget::MeleeAttack { target },
        };

        Self {
            attacker,
            target,
            name: attack_template.name.clone(),
            activation,
            data: attack_template.data,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    NoOp(Entity),
    MoveAlong { actor: Entity, path: Vec<MapPos> },
    Attack(AttackCommandData),
    EndPlanningPhase(Team),
    AssignActivation { actor: Entity, card_index: usize },
}

impl Action {
    fn active_actor(&self) -> Option<Entity> {
        match self {
            Action::NoOp(entity) => Some(*entity),
            Action::MoveAlong { actor, .. } => Some(*actor),
            Action::AssignActivation { actor, .. } => Some(*actor),
            Action::Attack(attack) => Some(attack.attacker),
            _ => None,
        }
    }
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
                available_actions.push((Some(hex), Action::AssignActivation { actor, card_index }));
            }
        }
    }

    for (team_entity, TeamReady(is_ready), Controller(is_pc)) in teams_q.iter() {
        if *is_pc && !*is_ready {
            available_actions.push((None, Action::EndPlanningPhase(Team(team_entity))));
        }
    }

    if available_actions.len() > 0 {
        commands.insert_resource(PossibleUserActions::new(available_actions));
    }

    Ok(())
}

pub fn collect_actions_for_performin_actions(
    user_actions: Option<Res<PossibleUserActions>>,
    selected_map_pos: Option<ResMut<SelectedMapPos>>,
    map: Res<HexMap>,
    active_actor_q: Query<(Entity, &Active, &Controller, &MapPos, &Team, &Attacks), With<Actor>>,
    target_actor_q: Query<(Entity, &MapPos, &Team), With<Actor>>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    if user_actions.is_some() {
        // The user actions are already available
        // => nothing more to do
        return Ok(());
    }

    if let Some((entity, actor_pos, team, attacks, activation)) =
        find_active_player_actor(&active_actor_q)
    {
        let mut actions = vec![];

        let hex = selected_map_pos.map(|sel_map_pos| sel_map_pos.0);
        if let Some(hex) = hex {
            if let Some(target) = find_enemy_at(&target_actor_q, &team, hex) {
                let distance = actor_pos.distance(&hex);

                for attack_option in attacks.0.iter() {
                    if attack_option.can_attack(distance) {
                        let attack =
                            AttackCommandData::new(entity, target, attack_option, activation);
                        actions.push((Some(hex), Action::Attack(attack)));
                    }
                }

                if actor_pos.distance(&hex) > 1 {
                    if let Some(mut path) = map.find_path(actor_pos, hex) {
                        if path.len() > 1 {
                            let max_steps = (path.len() - 1).max(3);
                            let path = path.drain(..max_steps).collect();

                            actions.push((
                                Some(hex),
                                Action::MoveAlong {
                                    actor: entity,
                                    path,
                                },
                            ));
                        }
                    }
                }
            } else if let Some(path) = map.find_path(actor_pos, hex) {
                actions.push((
                    Some(hex),
                    Action::MoveAlong {
                        actor: entity,
                        path,
                    },
                ));
            }
        }

        actions.push((None, Action::NoOp(entity)));

        commands.insert_resource(PossibleUserActions::new(actions));
    }

    Ok(())
}

fn find_active_player_actor(
    active_actor_q: &Query<(Entity, &Active, &Controller, &MapPos, &Team, &Attacks), With<Actor>>,
) -> Option<(Entity, MapPos, Team, Attacks, Card)> {
    for (entity, Active(activation), controller, map_pos, team, attacks) in active_actor_q.iter() {
        if controller.is_pc() {
            return Some((entity, *map_pos, *team, attacks.clone(), *activation));
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
        Action::MoveAlong { actor, path } => {
            commands.trigger(MoveToCommand(*actor, path.clone()));
        }

        Action::Attack(attack) => {
            commands.trigger(AttackCommand(attack.clone()));
        }

        Action::NoOp(..) => {
            // Do nothing, but progress ui state so next actor can be activated
        }

        Action::EndPlanningPhase(Team(id)) => {
            let mut team_ready = team_ready_q.get_mut(*id)?;
            team_ready.0 = true;
        }

        Action::AssignActivation { actor, card_index } => {
            commands.trigger(AssignActivationCommand {
                actor: *actor,
                card_index: *card_index,
            });
        }
    }

    if let Some(entity) = action.active_actor() {
        commands.entity(entity).remove::<Active>();
        commands.trigger(ActivationEndedEvent(entity));
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

pub fn handle_attack_command(
    trigger: On<AttackCommand>,
    combat_data_q: Query<(
        &Health,
        &Attributes,
        &Protection,
        Option<&RiskComplications>,
    )>,
    mut deck: ResMut<GameDeck>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let AttackCommand(attack) = trigger.event();
    let attacker = attack.attacker;

    let combat_finished_event = match &attack.target {
        AttackTarget::MeleeAttack { target } => {
            // info!(
            //     "[handle_attack_command] attacker={:?}, target={:?}",
            //     attacking_entity, target
            // );

            let [
                (health_a, attributes_a, protection_a, risk_complications),
                (health_t, attributes_t, protection_t, _),
            ] = combat_data_q.get_many([attacker, *target])?;

            let combat_result = handle_attack(
                Attack {
                    effort: attack.activation,
                    risky_complication: risk_complications.is_some(),
                    attacker: Combatant {
                        id: attacker,
                        attributes: attributes_a.effectiv_attributes(health_a),
                        protection: protection_a.clone(),
                    },
                    target: Target::SingleMelee(Combatant {
                        id: *target,
                        attributes: attributes_t.effectiv_attributes(health_t),
                        protection: protection_t.clone(),
                    }),
                    data: attack.data.clone(),
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

pub fn handle_combat_finished_event(
    trigger: On<CombatFinishedEvent>,
    mut health_q: Query<(Mut<Health>, Mut<Protection>, Mut<Items>)>,
) -> Result<(), BevyError> {
    let CombatFinishedEvent {
        result: combat_result,
        ..
    } = trigger.event();

    for (e, c) in combat_result.consequences.iter() {
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
