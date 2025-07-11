use bevy::prelude::*;

use crate::assets::{AttackOption, Attacks};
use crate::core::{Attribute, AttributeValues, Card, Suite};

use super::GameDeck;
use super::combat_resolution::{Attack, CombatConsequence, CombatResult, Target, handle_attack};
use super::fx::{FxEffect, FxSequence};
use super::ui::Z_LAYER_ACTOR;
use super::{
    map::{HexMap, MapPos, Obstacle},
    ui::{Description, MapPosSelectedEvent, SelectedMapPos, UiState, UiStateTransitionedEvent},
};

#[derive(Component, PartialEq, Clone, Copy)]
pub struct Team(pub Entity);

// #[derive(Component, Debug)]
// pub struct TeamDeck(pub Deck);

#[derive(Component, Debug)]
pub struct TeamHand(pub Vec<Card>);

#[derive(Bundle)]
pub struct TeamBundle {
    name: Name,
    // deck: TeamDeck,
    hand: TeamHand,
    player_controlled: PlayerControlled,
}
impl TeamBundle {
    pub fn new(name: impl Into<Name>, is_pc: bool) -> Self {
        Self {
            name: name.into(),
            // deck: TeamDeck(Deck::new_rnd()),
            hand: TeamHand(vec![]),
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

    pub fn refresh(&mut self, mut new_activations: Vec<Card>) {
        self.remaining = new_activations.drain(..).map(|c| Activation(c)).collect();
        self.remaining
            .sort_unstable_by(|a1, a2| a1.speed().cmp(&a2.speed()));
    }

    pub fn remaining(&self) -> &[Activation] {
        &self.remaining
    }
}

#[derive(Component, Debug, Clone)]
pub struct Health {
    pub max_health: u8,
    pub wounds: Vec<Card>,
}

impl Health {
    pub fn new(max_health: u8) -> Self {
        Self {
            max_health,
            wounds: vec![],
        }
    }

    pub fn damage_total(&self) -> u8 {
        self.wounds.iter().map(|c| c.value_high()).sum()
    }

    pub fn is_alive(&self) -> bool {
        self.max_health > self.damage_total()
    }

    fn current_attribute_values(&self, base_values: AttributeValues) -> AttributeValues {
        let AttributeValues {
            mut physical_strength,
            mut physical_agility,
            mut mental_strength,
            mut methal_agility,
        } = base_values;

        for card in self.wounds.iter() {
            match card.suite() {
                Suite::Clubs => {
                    physical_strength -= 1;
                }
                Suite::Spades => {
                    physical_agility -= 1;
                }
                Suite::Hearts => {
                    mental_strength -= 1;
                }
                Suite::Diamonds => {
                    methal_agility -= 1;
                }
            }
        }

        AttributeValues {
            physical_strength,
            physical_agility,
            mental_strength,
            methal_agility,
        }
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

#[derive(Debug, Clone)]
pub struct AttackData {
    pub target: AttackTarget,
    pub name: String,
    pub attribute: Attribute,
    pub damage: i16,
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
                difficulty,
            } => Self {
                target: AttackTarget::MeleeAttack { target },
                name: name.clone(),
                attribute: *attribute,
                damage: *damage,
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
}

pub fn handle_select_map_pos(
    map: Res<HexMap>,
    active_actor_q: Query<
        (Entity, &Actor, &Activations, &MapPos, &Team, &Attacks),
        Without<AiBehaviour>,
    >,
    target_actor_q: Query<(Entity, &MapPos, &Team), With<Actor>>,

    mut commands: Commands,
    mut map_pos_selected_er: EventReader<MapPosSelectedEvent>,
    player_actions: Option<Res<SelectedMapPos>>,
) {
    let Some(MapPosSelectedEvent(hex)) = map_pos_selected_er.read().last() else {
        return;
    };

    if let Some((entity, actor_pos, team, attacks)) = find_active_player_actor(&active_actor_q) {
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

            // println!("[DEBUG] available actions: {:?}", available_actions);
            commands.insert_resource(SelectedMapPos::new(entity, *hex, available_actions));
        }
    }
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

fn find_enemy_at(
    q_actor_at: &Query<(Entity, &MapPos, &Team), With<Actor>>,
    attacker_team: &Team,
    target_pos: MapPos,
) -> Option<Entity> {
    for (target, map_pos, target_team) in q_actor_at.iter() {
        if target_team.0 != attacker_team.0 && *map_pos == target_pos {
            return Some(target);
        }
    }
    None
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
) {
    let ActionTriggeredEvent(action) = trigger.event();
    let entity = trigger.target();

    info!(
        "[handle_action_selected_event] entity={:?}, action={:?}",
        entity, action,
    );

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
) {
    // info!(
    //     "handle_end_activation_command - entity={:?}",
    //     trigger.target()
    // );

    if let Ok(mut activations) = activations_q.get_mut(trigger.target()) {
        activations.active = None;
    }
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
    combat_data_q: Query<(&Health, &Activations, &AttributeValues)>,
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
                (attacker_health, activation, attacker_av),
                (target_health, _, target_av),
            ] = combat_data_q.get_many([attacker, *target])?;

            let effort_card = activation.active.as_ref().cloned().unwrap().0;

            let combat_result = handle_attack(
                Attack {
                    speed: effort_card,
                    attribute: attack.attribute,
                    damage: attack.damage,
                    difficulty: attack.difficulty,
                    attacker: (
                        attacker,
                        attacker_health.current_attribute_values(*attacker_av),
                    ),
                    target: Target::SingleMelee((
                        *target,
                        target_health.current_attribute_values(*target_av),
                    )),
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

pub fn handle_combat_finished_event(
    trigger: Trigger<CombatFinishedEvent>,
    mut health_q: Query<Mut<Health>>,
) -> Result<(), BevyError> {
    let CombatFinishedEvent {
        result: combat_result,
        ..
    } = trigger.event();

    for (e, c) in combat_result.iter() {
        if let CombatConsequence::Hit { damage } = c {
            let mut health = health_q.get_mut(*e)?;
            for card in damage.iter() {
                health.wounds.push(*card);
            }
        }
    }

    Ok(())
}
