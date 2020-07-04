use std::time::Instant;

use specs::prelude::*;

use super::super::actors::GameObject;
use crate::core::{Actor, Team, Action, Reaction, WorldPos, Opportunity};

#[derive(Debug)]
pub enum UserInput {
    Exit(),
    NewGame,
    SelectTeam(Vec<GameObject>),
    SelectAction(Action),
    SelectWorldPos(WorldPos),
    StartScrolling(),
    EndScrolling(),
    ScrollTo(i32, i32),
}

#[derive(Debug)]
pub enum InputContext {
    SelectedArea(WorldPos, Vec<GameObject>, Vec<Action>),
    Opportunity(Opportunity, Vec<Action>),
}

pub enum Game<'a, 'b> {
    Start,
    TeamSelection(Vec<GameObject>),
    Combat(CombatData<'a, 'b>),
}

pub struct CombatData<'a, 'b> {
    pub turn: u64,
    pub state: CombatState,
    pub world: World,
    pub dispatcher: Dispatcher<'a, 'b>,
}

#[derive(Debug)]
pub enum CombatState {
    Init(Vec<GameObject>),
    FindActor(),
    WaitForUserAction((Entity, Actor), Option<InputContext>, Vec<Reaction>),
    WaitUntil(Instant, Vec<Reaction>),
    ResolveAction((Entity, Actor), Action, Vec<Reaction>),
    EndTurn(),
    Win(Team),
}
