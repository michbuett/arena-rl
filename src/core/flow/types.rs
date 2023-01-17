use std::time::Instant;

use specs::prelude::{Dispatcher, World};

use crate::core::{
    ai::PlayerActionOptions, Actor, DisplayStr, GameObject, MapPos, ObjectGenerator, PlayerAction,
    Team, TextureMap, ID,
};

#[derive(Debug, Clone)]
pub enum UserInput {
    Exit(),
    NewGame,
    SelectTeam(Vec<GameObject>),
    // SelectAction(Act),
    SelectPlayerAction(PlayerAction),
    // RunPreparedAction(Act),
    // DelayPreparedAction(Act),
    SelectWorldPos(MapPos),
    StartScrolling(),
    EndScrolling(),
    ScrollTo(i32, i32),
}

#[derive(Debug)]
pub enum InputContext {
    // SelectedArea(MapPos, Vec<GameObject>, Vec<Act>),

    // AllocateEffort {
    //     options: Vec<(DisplayStr, ID, Act, u8)>,
    //     remaining_effort: Vec<D6>
    // },

    // TriggerPreparedAction(Act),
    // Opportunity(Opportunity, Vec<(Action, u8)>),
    SelectActionAt {
        selected_pos: MapPos,
        objects_at_selected_pos: Vec<GameObject>,
        options: PlayerActionOptions,
    },
}

pub enum Game<'a, 'b> {
    Start(ObjectGenerator, TextureMap),
    TeamSelection(ObjectGenerator, TextureMap, Vec<GameObject>),
    Combat(CombatData<'a, 'b>),
}

pub struct CombatData<'a, 'b> {
    pub turn: u64,
    pub active_team_idx: usize,
    pub teams: Vec<Team>,
    pub state: CombatState,
    pub world: World,
    pub dispatcher: Dispatcher<'a, 'b>,
    pub log: Vec<DisplayStr>,
    pub score: u64,
    pub turn_data: TurnData,
}

impl<'a, 'b> CombatData<'a, 'b> {
    pub fn active_team(&self) -> Team {
        self.teams.get(self.active_team_idx).unwrap().clone()
    }
}

#[derive(Debug)]
pub enum CombatState {
    Init(Vec<GameObject>),
    StartTurn(),
    FindActor(),
    AdvanceGame(),
    // TriggerAction(EntityAction, Vec<EntityAction>),
    // SelectAction(ID),
    SelectPlayerAction(ID),
    // PrepareAction(ID),
    // ExecuteOrDelayAction(ID),
    WaitForUserAction(Actor, Option<InputContext>),
    WaitUntil(Instant, Vec<PlayerAction>),
    ResolveAction(Vec<PlayerAction>),
    // WaitUntil(Instant, Vec<EntityAction>),
    // ResolveAction(Vec<EntityAction>),
    // Win(Team),
}

// pub type EntityAction = (ID, Act);

#[derive(Clone, Debug)]
pub struct TurnData {
    pub turn_number: u64,
    pub active_team_idx: usize,
    pub phase: CombatPhase,
    pub teams: Vec<Team>,
}

#[derive(Clone, Debug)]
pub enum CombatPhase {
    Plan,
    React,
    Resolve,
}

impl TurnData {
    pub fn new(teams: Vec<Team>) -> Self {
        Self {
            teams,
            turn_number: 1,
            active_team_idx: 0,
            phase: CombatPhase::Plan,
        }
    }

    pub fn active_team(&self) -> &Team {
        &self.teams.get(self.active_team_idx).unwrap()
    }

    pub fn step(&self) -> Self {
        let mut result = self.clone();
        if let CombatPhase::Plan = result.phase {
            result.phase = CombatPhase::React;
        } else if let CombatPhase::React = result.phase {
            result.phase = CombatPhase::Resolve;
        } else if result.active_team_idx < result.teams.len() - 1 {
            result.active_team_idx += 1;
            result.phase = CombatPhase::Plan;
        } else {
            result.active_team_idx = 0;
            result.phase = CombatPhase::Plan;
            result.turn_number += 1;
        }

        println!("[DEBUG] TurnData::step => result {:?}", result);
        result
    }
}
