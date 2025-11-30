use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use crate::core::{ActionKeyword, KeywordSet, RawActionTemplate};

use super::{
    actor::{ActionCommandData, ActionTriggeredEvent, Order, Team},
    fx::FxRunning,
    map::MapPos,
};

#[derive(Event, Debug)]
pub struct SelectManeuverCommand {
    pub actor: Entity,
    pub input_id: InputId,
    pub prompt: String,
    pub is_reaction: bool,
    pub filter: KeywordSet<ActionKeyword>,
}

#[derive(Event, Debug)]
pub struct SelectPathCommand {
    pub actor: Entity,
    pub input_id: InputId,
    pub prompt: String,
    pub length: u8,
}

#[derive(Event, Debug)]
pub struct SelectActorCommand {
    pub actor: Entity,
    pub input_id: InputId,
    pub prompt: String,
    pub filter: SelectActorFilterSet,
}

pub type SelectActorFilterParams<'a> = (Entity, &'a Team, &'a MapPos);

#[derive(Event, Debug)]
pub struct SelectActorFilterSet(pub Vec<SelectActorFilter>);

impl SelectActorFilterSet {
    pub fn compare(
        &self,
        candidate: SelectActorFilterParams,
        other: SelectActorFilterParams,
    ) -> bool {
        for filter in self.0.iter() {
            match filter {
                SelectActorFilter::OtherOnly => {
                    if candidate.0 == other.0 {
                        return false;
                    }
                }
                SelectActorFilter::EnemiesOnly => {
                    if candidate.1 == other.1 {
                        return false;
                    }
                }
                SelectActorFilter::WithinReach(reach) => {
                    let distance = candidate.2.distance(other.2);
                    if distance > *reach as i32 {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum SelectActorFilter {
    OtherOnly,
    EnemiesOnly,
    WithinReach(u8),
}

#[derive(Event)]
pub struct StartInputWorkfowCommand;

#[derive(Event)]
pub struct InputStepCompletedEvent {
    pub input_id: InputId,
    pub input_value: InputValue,
}

pub fn on_start_input_workflow_command(
    _trigger: On<StartInputWorkfowCommand>,
    mut input_workflow: ResMut<UserInputWorkflow>,
    mut commands: Commands,
) {
    input_workflow.advance(&mut commands);
}

pub fn on_fx_finished(
    _trigger: On<Remove, FxRunning>,
    input_workflow: Option<ResMut<UserInputWorkflow>>,
    mut commands: Commands,
) {
    if input_workflow.is_none() {
        return;
    }

    let mut input_workflow = input_workflow.unwrap();
    input_workflow.advance(&mut commands);
}

pub fn on_input_step_completed_event(
    trigger: On<InputStepCompletedEvent>,
    // action_queue: ResMut<ActionQueue>,
    input_workflow: Option<ResMut<UserInputWorkflow>>,
    mut commands: Commands,
) {
    if input_workflow.is_none() {
        warn!("no active input_workflow");
        return;
    }

    let mut input_workflow = input_workflow.unwrap();

    input_workflow.complete_step(trigger.input_id.clone(), trigger.input_value.clone());
    input_workflow.advance(&mut commands);
}

#[derive(Default, Debug, Resource)]
pub struct UserInputWorkflow {
    selected_values: HashMap<InputId, InputValue>,
    queued_actions: Vec<ActionOrder>,
    unresolved_inputs: Vec<(Entity, InputStep)>,
    next_action: Option<(ActionOrder, Vec<(Entity, ActionKind)>)>,
    awaiting_input: bool,
}

impl UserInputWorkflow {
    pub fn new(actor: Entity) -> Self {
        Self {
            selected_values: HashMap::from_iter(vec![("actor".into(), InputValue::Actor(actor))]),
            unresolved_inputs: vec![(
                actor,
                InputStep {
                    input_id: "initial_maneuver_select".into(),
                    prompt: "What do you want to do?".into(),
                    input_kind: InputKind::Maneuver {
                        actor: "actor".into(),
                        filter: vec![ActionKeyword::Action],
                        is_reaction: false,
                    },
                },
            )],
            ..default()
        }
    }

    fn complete_step(&mut self, input_id: InputId, input_value: InputValue) {
        if let InputValue::None = &input_value {
            println!(" (> skipping input '{input_id:?}')");
        } else {
            println!(" (> set input '{input_id:?}' to {input_value:?})");
        }

        if let InputValue::Maneuver {
            actor,
            is_reaction,
            template: ManeuverTemplate { actions, .. },
        } = &input_value
        {
            if *is_reaction && self.next_action.is_some() {
                // reaction are excuted right before with the action that tiggered them
                // (which is always the next action of the workflow)
                for a in actions.iter() {
                    let mut new_inputs = a.inputs.iter().map(|i| (*actor, i.clone())).collect();
                    self.unresolved_inputs.append(&mut new_inputs);
                }

                let (_, reactions) = self.next_action.as_mut().unwrap();
                let mut new_reactions = actions.iter().map(|a| (*actor, a.kind.clone())).collect();
                reactions.append(&mut new_reactions);
            } else {
                let mut new_actions = actions
                    .iter()
                    .map(|t| ActionOrder::from_template(t.clone(), *actor))
                    .collect();

                self.queued_actions.append(&mut new_actions);
            }
        }

        self.selected_values.insert(input_id, input_value);
        self.awaiting_input = false;
    }

    fn advance(&mut self, commands: &mut Commands) {
        println!(
            "Advancing input workflow - unresolved inputs: {}",
            self.unresolved_inputs
                .iter()
                .map(|(_, i)| i.input_id.0.clone())
                .collect::<Vec<_>>()
                .join(", ")
        );
        if self.awaiting_input {
            // There is already an active step
            // => skip
            // println!(" > Already active step: {:?}", self.awaiting_input);
            return;
        }

        if self.unresolved_inputs.is_empty() {
            for action in self.flush_next_actions() {
                println!(" > trigger next action: {action:?}");
                commands.trigger(ActionTriggeredEvent(action));
            }

            if self.queued_actions.is_empty() {
                // there are no more actions and no more unresolved inputs in the queue
                // -> The workflow is complete
                println!(" > Workflow is complete");
                commands.trigger(ActionTriggeredEvent(Order::EndActivation));
                commands.remove_resource::<UserInputWorkflow>();
                return;
            } else {
                let mut action_order = self.queued_actions.remove(0);
                for input_step in action_order.inputs {
                    if !self.selected_values.contains_key(&input_step.input_id) {
                        self.unresolved_inputs
                            .push((action_order.actor, input_step));
                    }
                }

                println!(" > Prepare next action: {:?}", action_order.kind);
                action_order.inputs = vec![];
                self.next_action = Some((action_order, vec![]));
            }
        };

        if self.unresolved_inputs.is_empty() {
            self.advance(commands);
        } else {
            let (actor, input_step) = self.unresolved_inputs.remove(0);
            println!(" > Next input: {:?}", input_step.input_id);
            self.trigger_step(actor, input_step, commands);
        }
    }

    fn trigger_step(
        &mut self,
        actor: Entity,
        input_step: InputStep,
        commands: &mut Commands<'_, '_>,
    ) {
        self.awaiting_input = true;

        let mut input_request_incomplete = true;
        let InputStep {
            input_id,
            prompt,
            input_kind,
        } = input_step;

        match input_kind {
            InputKind::Maneuver {
                actor,
                filter,
                is_reaction,
            } => {
                self.with_values([&actor], |[actor]| {
                    input_request_incomplete = false;
                    commands.trigger(SelectManeuverCommand {
                        filter: KeywordSet::new(&filter),
                        actor: actor.unwrap(),
                        input_id: input_id.clone(),
                        prompt: prompt.clone(),
                        is_reaction,
                    });
                });
            }

            InputKind::Path { length } => {
                input_request_incomplete = false;
                commands.trigger(SelectPathCommand {
                    actor,
                    input_id: input_id.clone(),
                    prompt: prompt.clone(),
                    length,
                });
            }

            InputKind::TargetActor { filter } => {
                input_request_incomplete = false;
                commands.trigger(SelectActorCommand {
                    actor,
                    input_id: input_id.clone(),
                    prompt: prompt.clone(),
                    filter: SelectActorFilterSet(filter.clone()),
                });
            }
        }

        if input_request_incomplete {
            commands.trigger(InputStepCompletedEvent {
                input_id,
                input_value: InputValue::None,
            });
        }
    }

    fn with_values<const N: usize, F, T>(&self, keys: [&InputId; N], f: F) -> Option<T>
    where
        F: FnOnce([&InputValue; N]) -> T,
    {
        let mut values = [&InputValue::None; N];
        for (idx, input_id) in keys.iter().enumerate() {
            if !self.selected_values.contains_key(input_id) {
                // Input value is used for further actions but has not been set or initialized
                // => this should not happen. (TODO maybe even panic here)
                warn!("Input with key '{input_id:?}' is missing. Further steps are skipped");
                return None;
            }

            if let Some(InputValue::None) = self.selected_values.get(input_id) {
                // Input step was completed with explicitly with None
                // => skip further actions that need this input value
                return None;
            }

            values[idx] = self.selected_values.get(input_id).unwrap();
        }

        Some(f(values))
    }

    fn flush_next_actions(&mut self) -> Vec<Order> {
        if self.next_action.is_none() {
            return vec![];
        }

        let mut ret = vec![];
        let (action, reactions) = self.next_action.as_ref().unwrap();

        for (actor, kind) in reactions.iter() {
            if let Some(a) = self.map_single_action_kind(*actor, kind) {
                ret.push(a);
            }
        }
        if let Some(a) = self.map_single_action_kind(action.actor, &action.kind) {
            ret.push(a);
        }

        self.next_action = None;
        ret
    }

    fn map_single_action_kind(&self, actor: Entity, kind: &ActionKind) -> Option<Order> {
        match kind {
            ActionKind::MoveTo { path } => self.with_values([path], |[path]| Order::MoveAlong {
                actor,
                path: path.unwrap(),
            }),

            ActionKind::Action { target, template } => match target {
                ActionTarget::SingleActor { target_actor } => {
                    self.with_values([target_actor], |[target_actor]| {
                        Order::Action(ActionCommandData::from_template(
                            actor,
                            vec![target_actor.unwrap()],
                            template,
                        ))
                    })
                }

                ActionTarget::Oneself => Some(Order::Action(ActionCommandData::from_template(
                    actor,
                    vec![actor],
                    template,
                ))),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct InputId(String);

impl From<&str> for InputId {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub enum InputKind {
    Path {
        length: u8,
    },
    TargetActor {
        filter: Vec<SelectActorFilter>,
    },
    Maneuver {
        actor: InputId,
        filter: Vec<ActionKeyword>,
        is_reaction: bool,
    },
}

#[derive(Clone, Debug, Default)]
pub enum InputValue {
    #[default]
    None,
    Path(Vec<MapPos>),
    // MapPos(MapPos),
    Actor(Entity),
    Maneuver {
        actor: Entity,
        template: ManeuverTemplate,
        is_reaction: bool,
    },
}

#[derive(Debug, Clone, Component)]
pub struct ActorManeuvers(pub Vec<ManeuverTemplate>);

#[derive(Clone, Debug)]
pub struct ManeuverTemplate {
    pub name: String,
    pub actions: Vec<ActionOrderTemplate>,
    pub keywords: KeywordSet<ActionKeyword>,
}

#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct ManeuverTemplates(pub Vec<(String, RawManeuverTemplate)>);

#[derive(Debug, Clone, Deserialize)]
pub struct RawManeuverTemplate {
    pub name: String,
    pub actions: Vec<ActionOrderTemplate>,
    pub keywords: Vec<ActionKeyword>,
}

impl Into<ManeuverTemplate> for RawManeuverTemplate {
    fn into(self) -> ManeuverTemplate {
        ManeuverTemplate {
            name: self.name,
            actions: self.actions,
            keywords: KeywordSet::new(&self.keywords),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ActionOrder {
    pub actor: Entity,
    pub kind: ActionKind,
    pub inputs: Vec<InputStep>,
}

impl ActionOrder {
    fn from_template(template: ActionOrderTemplate, actor: Entity) -> Self {
        Self {
            actor,
            kind: template.kind,
            inputs: template.inputs,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ActionOrderTemplate {
    pub kind: ActionKind,
    pub inputs: Vec<InputStep>,
}

#[derive(Clone, Debug, Deserialize)]
pub enum ActionKind {
    MoveTo {
        path: InputId,
    },

    Action {
        target: ActionTarget,
        template: RawActionTemplate,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub enum ActionTarget {
    SingleActor { target_actor: InputId },
    Oneself,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InputStep {
    pub input_id: InputId,
    pub input_kind: InputKind,
    pub prompt: String,
}

trait GetValue<T> {
    fn unwrap(&self) -> T;
}

// impl GetValue<MapPos> for InputValue {
//     fn unwrap(&self) -> MapPos {
//         match &self {
//             InputValue::MapPos(p) => *p,
//             _ => panic!("Expected MapPos, found {self:?}"),
//         }
//     }
// }

impl GetValue<Vec<MapPos>> for InputValue {
    fn unwrap(&self) -> Vec<MapPos> {
        match &self {
            InputValue::Path(p) => p.clone(),
            _ => panic!("Expected MapPos, found {self:?}"),
        }
    }
}

impl GetValue<Entity> for InputValue {
    fn unwrap(&self) -> Entity {
        match &self {
            InputValue::Actor(e) => *e,
            _ => panic!("Expected Entity, found {self:?}"),
        }
    }
}
