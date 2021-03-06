use std::time::Instant;

use specs::prelude::*;

use crate::core::*;

#[derive(Debug)]
pub enum UserInput {
    Exit(),
    NewGame,
    SelectTeam(Vec<GameObject>),
    SelectAction((Action, u8)),
    SelectWorldPos(WorldPos),
    StartScrolling(),
    EndScrolling(),
    ScrollTo(i32, i32),
}

#[derive(Debug)]
pub enum InputContext {
    SelectedArea(WorldPos, Vec<GameObject>, Vec<(Action, u8)>),
    // Opportunity(Opportunity, Vec<(Action, u8)>),
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
    // pub generator: ObjectGenerator,
}

impl<'a, 'b> CombatData<'a, 'b> {
    pub fn active_team(&self) -> Team {
        self.teams.get(self.active_team_idx).unwrap().clone()
    }
}

#[derive(Debug)]
pub enum CombatState {
    Init(Vec<GameObject>),
    FindActor(),
    SelectAction((Entity, Actor)),
    WaitForUserAction((Entity, Actor), Option<InputContext>),
    WaitUntil(Instant, Vec<EntityAction>),
    ResolveAction(EntityAction, Vec<EntityAction>),
    StartTurn(),
    // Win(Team),
}

pub type EntityAction = (Entity, Actor, Action, u8);
