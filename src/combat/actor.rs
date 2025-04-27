use bevy::prelude::*;

use crate::combat::combat_resolution::Defence;
use crate::core::{Card, Challenge, Deck, Suite};

use super::combat_resolution::{handle_attack, Attack, CombatResult};
use super::fx::{FxEffect, FxSequence};
use super::ui::Z_LAYER_ACTOR;
use super::{
    map::{HexMap, MapPos, Obstacle},
    ui::{Description, MapPosSelectedEvent, PlayerActions, UiState, UiStateTransitionedEvent},
    Visual,
};

#[derive(Component, PartialEq, Clone, Copy)]
pub struct Team(pub Entity);

#[derive(Component, Debug)]
pub struct TeamDeck(pub Deck);

#[derive(Component, Debug)]
pub struct TeamHand();

#[derive(Bundle)]
pub struct TeamBundle {
    name: Name,
    deck: TeamDeck,
    hand: TeamHand,
    player_controlled: PlayerControlled,
}
impl TeamBundle {
    pub fn new(name: impl Into<Name>, is_pc: bool) -> Self {
        Self {
            name: name.into(),
            deck: TeamDeck(Deck::new_rnd()),
            hand: TeamHand(),
            player_controlled: PlayerControlled(is_pc),
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

#[derive(Component)]
pub struct Activations {
    pub active: Option<Activation>,
    pub remaining: Vec<Activation>,
}

impl Activations {
    pub fn next_activation_initiative(&self) -> Option<u8> {
        self.remaining.iter().map(|Activation(c)| c.value).min()
    }
}

#[derive(Component)]
pub struct Health {
    pub wounds: Vec<Card>,
}

#[derive(Bundle)]
pub struct ActorBundle {
    pub actor: Actor,
    pub name: Name,
    pub description: Description,
    pub team: Team,
    pub play_controlled: PlayerControlled,
    pub map_pos: MapPos,
    pub visual: Visual,
    pub activations: Activations,
    pub health: Health,
    pub obstacle: Obstacle,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
}

impl ActorBundle {
    pub fn new(name: Name, team: Team, is_pc: bool, map_pos: MapPos, visual: Visual) -> Self {
        Self {
            actor: Actor::new(),
            name,
            description: Description::new("Some description for an actor"),
            team,
            play_controlled: PlayerControlled(is_pc),
            map_pos,
            visual,
            activations: Activations {
                active: None,
                remaining: vec![],
            },
            health: Health { wounds: vec![] },
            obstacle: Obstacle(f32::MAX),
            transform: Transform::from_translation(map_pos.into_vec3().with_z(Z_LAYER_ACTOR)),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::default(),
        }
    }
}

#[derive(Bundle)]
pub struct AiActorBundle {
    actor_bundle: ActorBundle,
    ai_behaivour: AiBehaviour,
}

#[derive(Debug, Component)]
pub struct PreparedAction(pub Action);

#[derive(Debug, Event)]
pub struct ActionSelectedEvent(pub Action);

#[derive(Debug, Event)]
pub struct BeginActivationCommand;

#[derive(Debug, Event)]
pub struct EndActivationCommand;

#[derive(Debug, Event)]
pub struct MoveToCommand(Vec<MapPos>);

#[derive(Debug, Event)]
pub struct AttackCommand(AttackData);

#[derive(Debug, Clone)]
pub enum AttackData {
    MeleeAttack { target: Entity },
}

#[derive(Debug, Clone)]
pub enum Action {
    NoOp,
    MoveAlong { path: Vec<MapPos> },
    Attack(AttackData),
}

pub fn handle_select_map_pos(
    map: Res<HexMap>,
    active_actor_q: Query<(Entity, &Actor, &Activations, &MapPos, &Team), Without<AiBehaviour>>,
    target_actor_q: Query<(Entity, &MapPos, &Team), With<Actor>>,

    mut commands: Commands,
    mut map_pos_selected_er: EventReader<MapPosSelectedEvent>,
    mut player_actions: ResMut<PlayerActions>,
) {
    let Some(MapPosSelectedEvent(hex)) = map_pos_selected_er.read().last() else {
        return;
    };

    if let Some((entity, actor_pos, team)) = find_active_player_actor(&active_actor_q) {
        if let Some(action) = player_actions.get_selected_action_when_at(hex) {
            // there is already a selected action for this hex
            // => execute this action
            commands.trigger_targets(ActionSelectedEvent(action.clone()), entity);
            player_actions.set_available_actions(*hex, vec![]);
        } else if let Some(attack) = find_eligible_target_at(&target_actor_q, &team, *hex) {
            player_actions.set_available_actions(*hex, vec![Action::Attack(attack)]);
        } else if let Some(path) = map.find_path(actor_pos, *hex) {
            player_actions.set_available_actions(*hex, vec![Action::MoveAlong { path }]);
        } else {
            player_actions.set_available_actions(*hex, vec![]);
        }
    }
}

fn find_active_player_actor(
    q_active_actor: &Query<(Entity, &Actor, &Activations, &MapPos, &Team), Without<AiBehaviour>>,
) -> Option<(Entity, MapPos, Team)> {
    for (entity, _actor, activations, map_pos, team) in q_active_actor.iter() {
        if activations.active.is_some() {
            return Some((entity, *map_pos, *team));
        }
    }
    None
}

fn find_eligible_target_at(
    q_actor_at: &Query<(Entity, &MapPos, &Team), With<Actor>>,
    attacker_team: &Team,
    target_pos: MapPos,
) -> Option<AttackData> {
    // println!("[find_eligible_target_at] hex={:?}", target_pos);

    for (target, map_pos, target_team) in q_actor_at.iter() {
        if target_team.0 != attacker_team.0 && *map_pos == target_pos {
            // println!("  Found target: {:?}", target);
            return Some(AttackData::MeleeAttack { target });
        }
    }
    // println!("  No target found");
    None
}

pub fn handle_action_selected_event(trigger: Trigger<ActionSelectedEvent>, mut commands: Commands) {
    let ActionSelectedEvent(action) = trigger.event();
    let entity = trigger.entity();

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
            // Just do nothing
        }
    }

    commands.trigger_targets(EndActivationCommand, entity);
}

pub fn handle_begin_activation_command(
    trigger: Trigger<BeginActivationCommand>,
    mut commands: Commands,
    mut actor_activation_q: Query<(Mut<Activations>, &PlayerControlled, &MapPos)>,
) {
    // info!(
    //     "[handle_begin_activation_command] entity={:?}",
    //     trigger.entity()
    // );

    let e = trigger.entity();
    let (mut activation, PlayerControlled(is_pc), mpos) = actor_activation_q.get_mut(e).unwrap();

    activation.active = activation.remaining.pop();

    if *is_pc {
        commands.trigger(UiStateTransitionedEvent(UiState::await_input(*mpos)));
    } else {
        // commands.trigger_targets(ActionSelectedEvent(Action::NoOp), e)
    }
}

pub fn handle_end_activation_command(
    trigger: Trigger<EndActivationCommand>,
    mut activations_q: Query<Mut<Activations>>,
) {
    // info!(
    //     "handle_end_activation_command - entity={:?}",
    //     trigger.entity()
    // );

    if let Ok(mut activations) = activations_q.get_mut(trigger.entity()) {
        activations.active = None;
    }
}

pub fn handle_move_to_command(trigger: Trigger<MoveToCommand>, mut commands: Commands) {
    let MoveToCommand(path) = trigger.event();
    let end_pos = *path.last().unwrap();
    let moving_entity = trigger.entity();

    commands.entity(moving_entity).insert(end_pos);

    FxSequence::new()
        .then(FxEffect::walk_along(moving_entity, path))
        .wait_until_finished()
        .run(&mut commands);
}

pub fn handle_attack_command(
    trigger: Trigger<AttackCommand>,
    mut combat_data_q: Query<(&MapPos, &Team, &Activations, Mut<Health>)>,
    mut deck_q: Query<Mut<TeamDeck>>,
    mut commands: Commands,
) {
    let AttackCommand(attack) = trigger.event();
    let attacking_entity = trigger.entity();

    match attack {
        AttackData::MeleeAttack { target } => {
            let [(attacker_pos, attacker_team, activation, _), (target_pos, target_team, _, mut health)] =
                combat_data_q
                    .get_many_mut([attacking_entity, *target])
                    .unwrap();
            // combat_data_q.get(attacking_entity).unwrap();
            // let (target_pos, target_team, _) = combat_data_q.get(*target).unwrap();
            let effort_card = activation.active.as_ref().cloned().unwrap().0;
            let [mut attack_deck, mut defence_deck] = deck_q
                .get_many_mut([attacker_team.0, target_team.0])
                .unwrap();

            let attack = Attack {
                damage: 5,
                challenge: Challenge {
                    advantage: 0,
                    target_suite: Suite::PhysicalAg,
                    target_value: 10,
                },
            };

            let defence = Defence {
                armor: 0,
                challenge: Challenge {
                    advantage: 0,
                    target_suite: Suite::PhysicalAg,
                    target_value: 10,
                },
            };

            let combat_result = handle_attack(
                attack,
                effort_card,
                &mut attack_deck.0,
                defence,
                &mut defence_deck.0,
            );

            match &combat_result {
                CombatResult::Wounded(card) => {
                    health.wounds.push(*card);
                }
                _ => {}
            }

            create_melee_attack_fx_sequence(
                attacking_entity,
                *attacker_pos,
                *target,
                *target_pos,
                combat_result,
            )
            .run(&mut commands);
        }
    }
}

fn create_melee_attack_fx_sequence(
    attacking_entity: Entity,
    attacker_pos: MapPos,
    target_entity: Entity,
    target_pos: MapPos,
    combat_result: CombatResult,
) -> FxSequence {
    let step_durration = 100;
    let attacker_pos = attacker_pos.into_vec3().with_z(Z_LAYER_ACTOR);
    let target_pos = target_pos.into_vec3().with_z(Z_LAYER_ACTOR);
    let path = vec![attacker_pos, target_pos, attacker_pos];

    let mut fx_seq = FxSequence::new()
        .then(FxEffect::MoveTo {
            entity: attacking_entity,
            path,
            movement_modification: crate::animations::MovementModification::None,
            step_durration,
        })
        .wait(step_durration);

    fx_seq = match combat_result {
        CombatResult::Wounded(..) => fx_seq.then(FxEffect::BloodSplatter(target_pos)),
        CombatResult::OutOfAction => fx_seq
            .then(FxEffect::Remove(target_entity))
            .then(FxEffect::BloodSplatter(target_pos))
            .wait(50)
            .then(FxEffect::BloodSplatter(target_pos))
            .wait(50)
            .then(FxEffect::BloodSplatter(target_pos)),

        _ => fx_seq,
    };

    fx_seq.wait_until_finished()
}
