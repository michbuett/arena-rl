use core::panic;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    time::Instant,
};

use specs::prelude::*;

use crate::core::{
    ai::PlayerActionOptions, Action, ActorTemplateName, Card, Deck, DisplayStr, GameObject, MapPos,
    ObjectGenerator, Team, TeamId, TextureMap, ID,
};

#[derive(Debug, Clone)]
pub enum UserInput {
    Exit(),
    NewGame,
    SelectTeam(Vec<GameObject>),
    SelectPlayerAction(Action),
    SelectActivationCard(usize),
    AssigneActivation(ID, TeamId, Card),
    SelectWorldPos(MapPos),
    StartScrolling(),
    EndScrolling(),
    ScrollTo(i32, i32),
}

#[derive(Debug, Clone)]
pub enum InputContext {
    ActivateActor {
        team: TeamId,
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
        mut world: World,
        dispatcher: Dispatcher<'a, 'b>,
        teams: Vec<Team>,
    ) -> Self {
        let turn = TurnState::new(&teams);
        let teams = TeamSet::new(teams);

        world.insert(teams);

        Self {
            state,
            world,
            dispatcher,
            log: vec![],
            score: 0,
            turn,
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

        // println!("world.maintain();");
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
    UpdateDeck(TeamId, Deck),
    StartTurn(TeamId, u8),
    RemoveCardFromHand(TeamId, Card),
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

    pub fn start_new_turn(self, team_id: TeamId, num_of_actors: u8) -> Self {
        self.add_change(StepChange::StartTurn(team_id, num_of_actors))
    }

    pub fn remove_card_from_hand(self, team_id: TeamId, card: Card) -> Self {
        self.add_change(StepChange::RemoveCardFromHand(team_id, card))
    }

    pub fn update_deck(self, team_id: TeamId, deck: Deck) -> Self {
        self.add_change(StepChange::UpdateDeck(team_id, deck))
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
                        let mut teams_mut = combat_data.world.fetch_mut::<TeamSet>();
                        teams_mut.set(team);
                    }

                    StepChange::StartTurn(team_id, num_actors) => {
                        let mut teams_mut = combat_data.world.fetch_mut::<TeamSet>();
                        let td = teams_mut.get_mut(&team_id);
                        td.start_new_turn(num_actors);
                    }

                    StepChange::RemoveCardFromHand(team_id, card) => {
                        let mut teams_mut = combat_data.world.fetch_mut::<TeamSet>();
                        let td = teams_mut.get_mut(&team_id);
                        td.hand.retain(|c| *c != card);
                    }

                    StepChange::UpdateDeck(team_id, deck) => {
                        let mut teams_mut = combat_data.world.fetch_mut::<TeamSet>();
                        let td = teams_mut.get_mut(&team_id);
                        td.deck = deck;
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

#[derive(Clone, Debug)]
pub struct TeamData {
    pub team: Team,
    pub deck: Deck,
    pub hand: Vec<Card>,
    pub ready: bool,
}

impl TeamData {
    fn new(t: Team) -> Self {
        TeamData {
            team: t,
            deck: Deck::new_rnd(),
            hand: vec![],
            ready: false,
        }
    }

    pub fn start_new_turn(&mut self, num_actor: u8) {
        // pub fn start_new_turn(&mut self, num_actor: u8) -> Self {
        let mut new_cards = (1..=num_actor)
            .map(|_| self.deck.deal())
            .collect::<Vec<_>>();

        self.hand.append(&mut new_cards);
        self.ready = false;
        // self
    }
}

#[derive(Clone, Debug, Default)]
pub struct TeamSet(BTreeMap<TeamId, TeamData>, Vec<TeamId>);

impl TeamSet {
    fn new(mut teams: Vec<Team>) -> Self {
        let mut btree_map = BTreeMap::new();
        let mut team_ids = vec![];

        for t in teams.drain(..) {
            team_ids.push(t.id);
            btree_map.insert(t.id, TeamData::new(t));
        }

        Self(btree_map, team_ids)
    }

    pub fn get(&self, team_id: &TeamId) -> &TeamData {
        self.0.get(team_id).unwrap()
    }

    pub fn get_mut(&mut self, team_id: &TeamId) -> &mut TeamData {
        self.0.get_mut(team_id).unwrap()
    }

    pub fn decks(&self) -> HashMap<TeamId, Deck> {
        let mut ret = HashMap::new();
        for td in self.0.values() {
            ret.insert(td.team.id, td.deck.clone());
        }
        ret
    }

    fn set(&mut self, td: TeamData) {
        self.0.insert(td.team.id, td);
    }

    // fn modify<F>(&mut self, t_id: TeamId, modify_fn: F)
    // where
    //     F: FnOnce(TeamData) -> TeamData,
    // {
    //     if let Some(td) = self.0.take(t_id) {
    //         self.0.insert(modify_fn(td));
    //     }
    // }
}

#[derive(Clone, Debug)]
pub struct TurnState {
    pub turn_number: u64,
    pub phase: CombatPhase,
    pub next_reinforcements: Option<u64>,

    teams: Vec<TeamId>,
    active_team_idx: usize,
    priority_team_idx: usize,
    teams_left: usize,
    reinforcements: Vec<(u64, TeamId, MapPos, ActorTemplateName)>,
}

impl TurnState {
    pub fn new(teams: &Vec<Team>) -> Self {
        let mut reinforcements = vec![];
        let mut team_ids = vec![];
        let mut next_reinforcements = None;

        for team in teams.iter() {
            team_ids.push(team.id);

            if let Some(team_reinforcements) = &team.reinforcements {
                for (turn, mpos, template) in team_reinforcements.iter() {
                    reinforcements.push((*turn, team.id, *mpos, template.clone()));

                    if let Some(v) = next_reinforcements {
                        if (turn - 1) < v {
                            next_reinforcements = Some(turn - 1);
                        }
                    } else {
                        next_reinforcements = Some(turn - 1)
                    }
                }
            }
        }

        Self {
            next_reinforcements,
            teams: team_ids,
            turn_number: 1,
            active_team_idx: 0,
            priority_team_idx: 0,
            teams_left: teams.len(),
            phase: CombatPhase::Planning,
            reinforcements,
        }
    }

    pub fn get_active_team(&self) -> Option<TeamId> {
        if let CombatPhase::Planning = self.phase {
            self.teams.get(self.active_team_idx).copied()
        } else {
            None
        }
    }

    fn next_team_idx(&self, idx: usize) -> usize {
        (idx + 1) % self.teams.len()
    }

    pub fn step(mut self) -> Self {
        if let CombatPhase::Planning = self.phase {
            self.teams_left -= 1;

            if self.teams_left == 0 {
                self.phase = CombatPhase::Action;
                self.active_team_idx = self.priority_team_idx;
            } else {
                self.active_team_idx = self.next_team_idx(self.active_team_idx);
            }
        } else {
            self.priority_team_idx = self.next_team_idx(self.priority_team_idx);
            self.active_team_idx = self.priority_team_idx;
            self.teams_left = self.teams.len();
            self.phase = CombatPhase::Planning;
            self.turn_number += 1;
            self.next_reinforcements = self.next_reinforcements();
        }

        self
    }

    pub fn cmp_team_by_priority(&self, ta: TeamId, tb: TeamId) -> Ordering {
        if ta == tb {
            Ordering::Equal
        } else {
            let mut idx = self.priority_team_idx;
            let mut steps = 1;

            while steps <= self.teams.len() {
                if self.teams[idx] == ta {
                    return Ordering::Greater;
                }
                if self.teams[idx] == tb {
                    return Ordering::Less;
                }

                steps += 1;
                idx = self.next_team_idx(idx);
            }

            panic!(
                "Fn should have returned a value by now. Maybe comparing unknown teams ({:?}, {:?})",
                ta, tb
            );
        }
    }

    pub fn reinforcements(&self) -> Vec<(TeamId, MapPos, ActorTemplateName)> {
        self.reinforcements
            .iter()
            .filter_map(|(turn, team_id, mpos, template)| {
                if *turn == self.turn_number {
                    Some((*team_id, *mpos, template.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn next_reinforcements(&self) -> Option<u64> {
        self.reinforcements
            .iter()
            .filter_map(|(turn, ..)| {
                if *turn >= self.turn_number {
                    Some(*turn - self.turn_number)
                } else {
                    None
                }
            })
            .min()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CombatPhase {
    Planning,
    Action,
}

#[test]
fn test_stepping_turn_state_activates_teams_correctly() {
    let teams = vec![
        Team {
            id: TeamId::new(1),
            name: "Team #1",
            is_pc: true,
            reinforcements: None,
        },
        Team {
            id: TeamId::new(2),
            name: "Team #2",
            is_pc: true,
            reinforcements: None,
        },
        Team {
            id: TeamId::new(3),
            name: "Team #3",
            is_pc: true,
            reinforcements: None,
        },
    ];

    // 1st turn
    let turn_state = TurnState::new(&teams);

    assert_eq!(turn_state.phase, CombatPhase::Planning);
    assert_eq!(turn_state.priority_team_idx, 0);
    assert_eq!(turn_state.get_active_team().unwrap(), TeamId::new(1));

    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Planning);
    assert_eq!(turn_state.priority_team_idx, 0);
    assert_eq!(turn_state.get_active_team().unwrap(), TeamId::new(2));

    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Planning);
    assert_eq!(turn_state.priority_team_idx, 0);
    assert_eq!(turn_state.get_active_team().unwrap(), TeamId::new(3));

    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Action);
    assert_eq!(turn_state.priority_team_idx, 0);
    assert!(turn_state.get_active_team().is_none());

    // start 2nd turn: the next team should get priority and should start planning
    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Planning);
    assert_eq!(turn_state.priority_team_idx, 1);
    assert_eq!(turn_state.get_active_team().unwrap(), TeamId::new(2));

    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Planning);
    assert_eq!(turn_state.priority_team_idx, 1);
    assert_eq!(turn_state.get_active_team().unwrap(), TeamId::new(3));

    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Planning);
    assert_eq!(turn_state.priority_team_idx, 1);
    assert_eq!(turn_state.get_active_team().unwrap(), TeamId::new(1));

    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Action);
    assert_eq!(turn_state.priority_team_idx, 1);
    assert!(turn_state.get_active_team().is_none());

    // start 3rd turn: the last team should get priority
    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Planning);
    assert_eq!(turn_state.priority_team_idx, 2);
    assert_eq!(turn_state.get_active_team().unwrap(), TeamId::new(3));

    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Planning);
    assert_eq!(turn_state.priority_team_idx, 2);
    assert_eq!(turn_state.get_active_team().unwrap(), TeamId::new(1));

    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Planning);
    assert_eq!(turn_state.priority_team_idx, 2);
    assert_eq!(turn_state.get_active_team().unwrap(), TeamId::new(2));

    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Action);
    assert_eq!(turn_state.priority_team_idx, 2);
    assert!(turn_state.get_active_team().is_none());

    // start 4th turn: the first team should have priority again
    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Planning);
    assert_eq!(turn_state.priority_team_idx, 0);
    assert_eq!(turn_state.get_active_team().unwrap(), TeamId::new(1));

    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Planning);
    assert_eq!(turn_state.priority_team_idx, 0);
    assert_eq!(turn_state.get_active_team().unwrap(), TeamId::new(2));

    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Planning);
    assert_eq!(turn_state.priority_team_idx, 0);
    assert_eq!(turn_state.get_active_team().unwrap(), TeamId::new(3));

    let turn_state = turn_state.step();

    assert_eq!(turn_state.phase, CombatPhase::Action);
    assert_eq!(turn_state.priority_team_idx, 0);
    assert!(turn_state.get_active_team().is_none());
}
