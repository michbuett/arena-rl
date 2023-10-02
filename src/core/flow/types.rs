use core::panic;
use std::{cmp::Ordering, collections::HashMap, time::Instant};

use specs::prelude::*;

use crate::core::{
    ai::PlayerActionOptions, Action, Card, Deck, DisplayStr, GameObject, MapPos, ObjectGenerator,
    RndDeck, Team, TeamId, TextureMap, ID,
};

#[derive(Debug, Clone)]
pub enum UserInput {
    Exit(),
    NewGame,
    SelectTeam(Vec<GameObject>),
    SelectPlayerAction(Action),
    SelectActivationCard(usize),
    AssigneActivation(ID, usize),
    SelectWorldPos(MapPos),
    StartScrolling(),
    EndScrolling(),
    ScrollTo(i32, i32),
}

#[derive(Debug, Clone)]
pub enum InputContext {
    ActivateActor {
        hand: Vec<Card>,
        possible_actors: HashMap<MapPos, (ID, u8)>,
        selected_card_idx: Option<usize>,
    },
    SelectAction {
        options: PlayerActionOptions,
    },
}

pub enum Game<'a, 'b> {
    Start(ObjectGenerator, TextureMap),
    TeamSelection(ObjectGenerator, TextureMap, Vec<GameObject>),
    Combat(CombatData<'a, 'b>),
}

pub struct CombatData<'a, 'b> {
    dispatcher: Dispatcher<'a, 'b>,
    log: Vec<DisplayStr>,

    pub score: u64,
    pub state: CombatState,
    pub turn: TurnState,
    pub world: World,
}

impl<'a, 'b> CombatData<'a, 'b> {
    pub fn new(
        state: CombatState,
        world: World,
        dispatcher: Dispatcher<'a, 'b>,
        teams: Vec<Team>,
    ) -> Self {
        Self {
            state,
            world,
            dispatcher,
            log: vec![],
            score: 0,
            turn: TurnState::new(teams),
        }
    }

    pub fn get_turn(&self) -> &TurnState {
        &self.turn
    }

    pub fn get_state(&self) -> &CombatState {
        &self.state
    }

    pub fn get_world(&self) -> &World {
        &self.world
    }

    pub fn step(mut self, step_result: StepResult) -> Self {
        step_result.unwind(&mut self);

        self.dispatcher.dispatch(&mut self.world);
        self.world.maintain();
        self
    }
}

enum StepChange {
    SwitchState(CombatState),
    AddScore(u64),
    AppendLog(DisplayStr),
    AdvanceGame(TurnState),
    ModifyTeam(TeamData),
}

pub struct StepResult(Option<Vec<StepChange>>);

impl StepResult {
    pub fn new() -> Self {
        Self(None)
    }

    fn add_change(mut self, c: StepChange) -> Self {
        self.0.get_or_insert(vec![]).push(c);
        self
    }

    pub fn switch_state(self, s: CombatState) -> Self {
        self.add_change(StepChange::SwitchState(s))
    }

    pub fn add_score(self, s: u64) -> Self {
        self.add_change(StepChange::AddScore(s))
    }

    pub fn append_log(self, l: impl Into<Option<DisplayStr>>) -> Self {
        if let Some(l) = l.into() {
            self.add_change(StepChange::AppendLog(l))
        } else {
            self
        }
    }

    pub fn modify_team(self, td: TeamData) -> Self {
        self.add_change(StepChange::ModifyTeam(td))
    }

    pub fn advance_game(self, td: TurnState) -> Self {
        self.add_change(StepChange::AdvanceGame(td))
    }

    fn unwind(self, combat_data: &mut CombatData) {
        if let Some(mut changes) = self.0 {
            for c in changes.drain(..) {
                match c {
                    StepChange::SwitchState(s) => {
                        combat_data.state = s;
                    }

                    StepChange::AddScore(s) => {
                        combat_data.score += s;
                    }

                    StepChange::AdvanceGame(td) => {
                        combat_data.turn = td;
                    }

                    StepChange::ModifyTeam(team) => {
                        combat_data.turn.set_team(team);
                    }

                    StepChange::AppendLog(l) => {
                        combat_data.log.insert(0, l);

                        if combat_data.log.len() > 10 {
                            combat_data.log.pop();
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum CombatState {
    Init(Vec<GameObject>),
    StartTurn(),
    FindActor(),
    AdvanceGame(),
    SelectInitiative(),
    SelectPlayerAction(ID),
    WaitForUserInput(InputContext, Option<SelectedPos>),
    WaitUntil(Instant, Vec<Action>),
    ResolveAction(Vec<Action>),
}

#[derive(Debug, Clone)]
pub struct SelectedPos {
    pub pos: MapPos,
    pub objects: Vec<GameObject>,
}

// pub type EntityAction = (ID, Act);
#[derive(Clone, Debug)]
pub struct TeamData {
    pub team: Team,
    pub deck: RndDeck,
    pub hand: Vec<Card>,
    pub ready: bool,
}

impl TeamData {
    fn new(t: Team) -> Self {
        TeamData {
            team: t,
            deck: RndDeck::new(),
            hand: vec![],
            ready: false,
        }
    }

    pub fn start_new_turn(mut self, num_actor: u8) -> Self {
        let mut new_cards = (1..=num_actor)
            .map(|_| self.deck.deal())
            .collect::<Vec<_>>();

        self.hand.append(&mut new_cards);
        self.ready = false;
        self
    }
}

#[derive(Clone, Debug)]
pub struct TurnState {
    pub turn_number: u64,
    pub phase: CombatPhase,
    pub teams: Vec<TeamData>,

    active_team_idx: usize,
    priority_team_idx: usize,
    teams_left: usize,
}

impl TurnState {
    pub fn new(mut teams: Vec<Team>) -> Self {
        let teams_left = teams.len();
        let teams = teams.drain(..).map(|t| TeamData::new(t)).collect();

        Self {
            teams,
            turn_number: 1,
            active_team_idx: 0,
            priority_team_idx: 0,
            teams_left,
            phase: CombatPhase::Planning,
        }
    }

    pub fn get_active_team(&self) -> &TeamData {
        &self.teams.get(self.active_team_idx).unwrap()
    }

    pub fn get_team(&self, team_id: TeamId) -> &TeamData {
        let idx = self.teams.iter().position(|t| t.team.id == team_id);

        if let Some(idx) = idx {
            &self.teams[idx]
        } else {
            panic!("Unknown team: '{:?}'", team_id)
        }
    }

    fn set_team(&mut self, td: TeamData) {
        // let mut result = self.clone();
        let id = td.team.id;
        let idx = self.teams.iter().position(|t| t.team.id == id);

        if let Some(idx) = idx {
            self.teams[idx] = td;
        } else {
            panic!("Modifying unknown team: '{:?}'", id);
        }
    }

    pub fn step(mut self) -> Self {
        // println!("\n[DEBUG] TurnData::step - current turn {}, phase: {:?}, active team index: {}, priority team index: {}", self.turn_number, self.phase, self.active_team_idx, self.priority_team_idx);
        if let CombatPhase::Planning = self.phase {
            if self.teams_left == 0 {
                self.phase = CombatPhase::Action;
                // return None;
            } else {
                self.teams_left -= 1;
                self.active_team_idx = (self.active_team_idx + 1) % self.teams.len();
            }
        } else {
            self.priority_team_idx = (self.priority_team_idx + 1) % self.teams.len();
            self.active_team_idx = self.priority_team_idx;
            self.teams_left = self.teams.len();
            self.phase = CombatPhase::Planning;
            self.turn_number += 1;
        }

        println!("[DEBUG] TurnData::step - current turn {}, phase: {:?}, active team index: {}, priority team index: {}", self.turn_number, self.phase, self.active_team_idx, self.priority_team_idx);
        self
    }

    pub fn cmp_team_by_priority(&self, ta: TeamId, tb: TeamId) -> Ordering {
        if ta == tb {
            Ordering::Equal
        } else {
            let mut idx = self.priority_team_idx;
            let mut steps = 1;

            while steps <= self.teams.len() {
                if self.teams[idx].team.id == ta {
                    return Ordering::Greater;
                }
                if self.teams[idx].team.id == tb {
                    return Ordering::Less;
                }

                steps += 1;
                idx = (idx + 1) % self.teams.len();
            }

            panic!(
                "Fn should have returned a value by now. Maybe comparing unknown teams ({}, {})",
                ta, tb
            );
        }
    }
}

#[derive(Clone, Debug)]
pub enum CombatPhase {
    Planning,
    Action,
}
