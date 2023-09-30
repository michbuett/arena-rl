use core::panic;
use std::{cmp::Ordering, collections::HashMap, time::Instant};

use specs::prelude::*;
use specs::prelude::{Dispatcher, World};

use crate::core::{
    ai::PlayerActionOptions, Card, Deck, DisplayStr, GameObject, MapPos, ObjectGenerator,
    PlayerAction, RndDeck, Team, TeamId, TextureMap, ID,
};

#[derive(Debug, Clone)]
pub enum UserInput {
    Exit(),
    NewGame,
    SelectTeam(Vec<GameObject>),
    // SelectAction(Act),
    SelectPlayerAction(PlayerAction),
    SelectActivationCard(usize),
    // RunPreparedAction(Act),
    // DelayPreparedAction(Act),
    AssigneActivation(ID, usize),
    SelectWorldPos(MapPos),
    StartScrolling(),
    EndScrolling(),
    ScrollTo(i32, i32),
}

#[derive(Debug, Clone)]
pub enum InputContext {
    // SelectedArea(MapPos, Vec<GameObject>, Vec<Act>),

    // AllocateEffort {
    //     options: Vec<(DisplayStr, ID, Act, u8)>,
    //     remaining_effort: Vec<D6>
    // },

    // TriggerPreparedAction(Act),
    // Opportunity(Opportunity, Vec<(Action, u8)>),
    ActivateActor {
        hand: Vec<Card>,
        possible_actors: HashMap<MapPos, (ID, u8)>,
        selected_card_idx: Option<usize>,
    },
    SelectAction {
        // active_actor: Actor,
        options: PlayerActionOptions,
    },
    // SelectActionAt {
    //     selected_pos: MapPos,
    //     objects_at_selected_pos: Vec<GameObject>,
    //     options: PlayerActionOptions,
    // },
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

    // pub fn active_team(&self) -> Team {
    //     self.teams.get(self.active_team_idx).unwrap().clone()
    // }
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

        // (state, turn, score, log)
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
    SelectInitiative(),
    SelectPlayerAction(ID),
    // PrepareAction(ID),
    // ExecuteOrDelayAction(ID),
    WaitForUserInput(InputContext, Option<SelectedPos>),
    // WaitForUserAction(Actor, Option<InputContext>),
    WaitUntil(Instant, Vec<PlayerAction>),
    ResolveAction(Vec<PlayerAction>),
    // WaitUntil(Instant, Vec<EntityAction>),
    // ResolveAction(Vec<EntityAction>),
    // Win(Team),
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

    // TODO: find more elgant solution to mutate team data (e.g. after assigning activations)
    // pub fn modify_team<F>(&mut self, team_id: TeamId, f: F)
    // where
    //     F: FnOnce(TeamData) -> TeamData,
    // {
    //     let idx = self.teams.iter().position(|t| t.team.id == team_id);
    //     if let Some(idx) = idx {
    //         self.teams[idx] = f(self.teams[idx].clone());
    //     }
    // }

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

    // fn activate_next_team(&mut self) -> Option<&TeamData> {
    //     if self.teams_left == 0 {
    //         return None;
    //     }

    //     println!(
    //         "activate next team - teams left: {}, active team: {}",
    //         self.teams_left, self.active_team_idx
    //     );
    //     self.teams_left -= 1;
    //     self.active_team_idx = (self.active_team_idx + 1) % self.teams.len();

    //     println!(
    //         "activate next team - teams left: {}, active team: {}",
    //         self.teams_left, self.active_team_idx
    //     );
    //     self.teams.get(self.active_team_idx)
    // }

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

            // let next_team = self.activate_next_team();
            // if next_team.is_none() {
            //     self.phase = CombatPhase::Action;
            // }
        } else {
            self.priority_team_idx = (self.priority_team_idx + 1) % self.teams.len();
            self.active_team_idx = self.priority_team_idx;
            self.teams_left = self.teams.len();
            self.phase = CombatPhase::Planning;
            self.turn_number += 1;
            // self.reset_team_data();
        }

        println!("[DEBUG] TurnData::step - current turn {}, phase: {:?}, active team index: {}, priority team index: {}", self.turn_number, self.phase, self.active_team_idx, self.priority_team_idx);
        self
    }

    // fn reset_team_data(&mut self) {
    //     for t in self.teams.iter_mut() {
    //         t.ready = false;
    //         if t.team.is_pc {
    //             let num = std::cmp::min(5, 10 - t.hand.len());
    //             let mut new_cards = (1..=num).map(|_| t.deck.deal()).collect::<Vec<_>>();

    //             t.hand.append(&mut new_cards);
    //         }
    //     }
    // }

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
