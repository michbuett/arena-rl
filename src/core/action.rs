use std::convert::TryInto;

use crate::components::{FxEffect, FxSequence};
use crate::core::ai::{attack_vector, AttackVector};
use crate::core::{DisplayStr, MapPos, Path, WorldPos};

use super::actors::{
    resolve_combat, Actor, AttackOption, AttackTarget, AttackType, GameObject, Hit, HitResult,
    Team, Trait, Wound, ID,
};
use super::ai::{can_attack_with, find_charge_path};
use super::{Change, CoreWorld, HitEffect, SuperLineIter};

#[derive(Debug, Clone)]
pub enum Action {
    StartTurn(),
    ResolvePendingActions(),
    EndTurn(Team),
    Done(String),
    Rest(),
    MoveTo(Path),
    Activate(ID),
    Attack(ID, AttackOption, AttackVector, String),
    Ambush(AttackOption),
    UseAbility(ID, String, Trait),
    // Disengage(Tile), // need conceptual overhaul
}

#[derive(Debug, Clone)]
pub struct Act {
    pub action: Action,
    pub delay: u8,
    pub allocated_effort: Option<u8>,
}

impl Act {
    pub fn new(a: Action) -> Self {
        Self {
            action: a,
            delay: 0,
            allocated_effort: None,
        }
    }

    pub fn delay(mut self, d: u8) -> Self {
        self.delay = d;
        self
    }

    pub fn effort(mut self, e: u8) -> Self {
        self.allocated_effort = Some(e);
        self
    }

    pub fn end_turn(t: Team) -> Self {
        Self::new(Action::EndTurn(t))
    }

    pub fn done(msg: impl ToString) -> Self {
        Self::new(Action::Done(msg.to_string())).effort(0)
    }

    pub fn pass() -> Self {
        Self::new(Action::Done("Waiting for next turn...".to_string())).delay(1)
    }

    pub fn rest() -> Self {
        Self::new(Action::Rest()).delay(1).effort(0)
    }

    pub fn activate(e: ID) -> Self {
        Self::new(Action::Activate(e))
    }

    pub fn move_to(p: Path) -> Self {
        Self::new(Action::MoveTo(p))
    }

    pub fn attack(target: ID, ao: AttackOption, av: AttackVector, n: String) -> Self {
        let e = ao.required_effort;
        Self::new(Action::Attack(target, ao, av, n))
            .delay(1)
            .effort(e)
    }

    pub fn ambush(attack: AttackOption) -> Self {
        Self::new(Action::Ambush(attack)).delay(1).effort(0)
    }

    // pub fn disengage(to_pos: Tile) -> Self {
    //     Self::new(Action::Disengage(to_pos)).delay(1).effort(0)
    // }

    pub fn use_ability(target: ID, key: impl ToString, t: Trait, delay: u8) -> Self {
        Self::new(Action::UseAbility(target, key.to_string(), t)).delay(delay)
    }
}

pub struct ActionResultBuilder<'a> {
    pub world: CoreWorld<'a>,
    pub fx_seq: FxSequence,
    pub log: Option<DisplayStr>,
    pub score: u64,
}

impl<'a> ActionResultBuilder<'a> {
    fn new(world: CoreWorld<'a>) -> Self {
        Self {
            world,
            fx_seq: FxSequence::new(),
            log: None,
            score: 0,
        }
    }

    fn into_result(self) -> ActionResult {
        ActionResult {
            changes: self.world.into_changes(),
            fx_seq: self.fx_seq,
            log: self.log,
            score: self.score,
        }
    }

    fn chain<M>(self, f: M) -> Self
    where
        M: FnOnce(CoreWorld<'a>) -> Self,
    {
        let new_result = f(self.world);
        let log = match (self.log, new_result.log) {
            (Some(l), None) => Some(l),
            (None, Some(l)) => Some(l),
            (Some(l1), Some(l2)) => Some(DisplayStr::new(format!("{}\n{}", l1, l2))),
            _ => None,
        };

        Self {
            world: new_result.world,
            fx_seq: self.fx_seq.then_append(new_result.fx_seq),
            score: self.score + new_result.score,
            log,
        }
    }

    fn append_fx_seq(mut self, fx_seq: FxSequence) -> Self {
        self.fx_seq = self.fx_seq.then_append(fx_seq);
        self
    }

    fn add_fx(mut self, eff: FxEffect) -> Self {
        self.fx_seq = self.fx_seq.then(eff);
        self
    }

    fn append_log(mut self, other_log: Option<DisplayStr>) -> Self {
        self.log = match (self.log, other_log) {
            (Some(l), None) => Some(l),
            (None, Some(l)) => Some(l),
            (Some(l1), Some(l2)) => Some(DisplayStr::new(format!("{}\n{}", l1, l2))),
            _ => None,
        };
        self
    }

    fn score(mut self, s: u64) -> Self {
        self.score += s;
        self
    }
}

pub struct ActionResult {
    pub changes: Vec<Change>,
    pub fx_seq: FxSequence,
    pub log: Option<DisplayStr>,
    pub score: u64,
}

impl ActionResult {
    fn empty() -> Self {
        Self {
            changes: vec![],
            fx_seq: FxSequence::new(),
            log: None,
            score: 0,
        }
    }
}

pub fn act(actor_id: ID, a: Act, mut cw: CoreWorld) -> ActionResult {
    if let Some(actor) = cw.get_actor(actor_id).cloned() {
        let delay = a.delay;
        if delay > 0 {
            cw.update_actor(actor.prepare(a.delay(delay - 1)));

            ActionResultBuilder::new(cw).into_result()
        } else {
            run_action(actor, a, cw)
        }
    } else {
        no_op()
    }
}

fn run_action<'a>(actor: Actor, a: Act, mut cw: CoreWorld) -> ActionResult {
    let actor_id = actor.id;

    match a.action {
        Action::StartTurn() => {
            if let Some(actor) = cw.get_actor(actor.id).cloned() {
                // we need to get the most current actor since this action is executed immediatly
                // after resolving pending actions
                let a_pos = MapPos::from_world_pos(actor.pos);
                let engaged_in_combat = is_next_to_enemy(&actor.team, a_pos, &cw);
                let mut actor = actor.start_next_turn(engaged_in_combat);

                if !actor.is_concious() {
                    actor = actor.prepare(Act::rest().delay(0));
                }

                cw.update_actor(actor);

                ActionResultBuilder::new(cw).into_result()
            } else {
                no_op()
            }
        }

        Action::ResolvePendingActions() => {
            if let Some(pending_action) = actor.pending_action.clone() {
                act(actor_id, pending_action, cw)
            } else {
                no_op()
            }
        }

        Action::EndTurn(team) => {
            while let Some(a) = cw.find_actor(|a| a.team == team && a.pending_action.is_none()) {
                cw.update_actor(a.prepare(Act::pass()));
            }

            ActionResultBuilder::new(cw).into_result()
        }

        Action::Done(_) => no_op(),

        Action::Rest() => {
            let prev_pain = actor.health.pain;
            let actor = actor.rest();
            let mut fx_seq = FxSequence::new();

            if prev_pain == 0 {
                fx_seq = fx_seq
                    .then(FxEffect::say(
                        "I try summon extra strength".to_string(),
                        actor.pos,
                    ))
                    .wait(500);
            } else if prev_pain > actor.health.pain {
                let recover = prev_pain - actor.health.pain;
                fx_seq = fx_seq
                    .then(FxEffect::say(
                        format!("Pain is fading away ({})", recover),
                        actor.pos,
                    ))
                    .wait(500);
            }

            cw.update(actor.into());

            ActionResultBuilder::new(cw)
                .append_fx_seq(fx_seq)
                .into_result()
        }

        Action::UseAbility(target_entity, ability_name, t) => {
            if let Some(target_actor) = cw.get_actor(target_entity).cloned() {
                let fx_pos = target_actor.pos.clone();
                let fx_str = t.name.to_string();
                let actor_name = target_actor.name.clone();
                let log = DisplayStr::new(format!("{} used ability: {}", actor_name, ability_name));

                cw.update_actor(target_actor.use_ability(ability_name, t));

                ActionResultBuilder::new(cw)
                    .add_fx(FxEffect::say(fx_str, fx_pos))
                    .append_log(log.into())
                    .into_result()
            } else {
                no_op()
            }
        }

        Action::MoveTo(path) => {
            if path.is_empty() {
                return no_op();
            }

            let mut result_builder = ActionResultBuilder::new(cw);
            let mut num_steps = 0;

            for t in path {
                result_builder = result_builder.chain(|w| {
                    let (result, did_move) = take_step(actor_id, t.to_world_pos(), w);

                    if did_move {
                        num_steps += 1;
                    }

                    result
                });
            }

            result_builder
                .chain(|mut w| {
                    // test if actor actually exists (it is possible that the moving actor has been killed by now; e.g. from some ambush)
                    if let Some(actor) = w.get_actor(actor_id).cloned() {
                        let required_effort = num_steps - actor.attr(super::Attr::Movement).val();
                        w.update_actor(
                            actor.prepare(
                                Act::done("Did move...")
                                    .effort(required_effort.try_into().unwrap_or(0)),
                            ),
                        );
                    }
                    ActionResultBuilder::new(w)
                })
                .into_result()
        }

        Action::Activate(id) => {
            if let Some(target) = cw.get_actor(id).cloned() {
                cw.update_actor(actor.deactivate());
                cw.update_actor(target.activate());

                ActionResultBuilder::new(cw).into_result()
            } else {
                no_op()
            }
        }

        Action::Attack(target_id, attack, _, _) => {
            handle_attack(actor.id, target_id, attack, cw).into_result()
        }

        Action::Ambush(_attack) => {
            // an ambush is trigger when an enemy walks into the zone of danger
            // => if it has not been triggered yet, then ignore it
            no_op()
        }
    }
}

fn no_op() -> ActionResult {
    ActionResult::empty()
}

/// filter items where there is no actual target
fn filter_attack_vector(input: &AttackVector, w: &CoreWorld) -> Vec<AttackTarget> {
    let mut input = input.to_vec();
    (&mut input)
        .drain(..)
        .fold(vec![], |mut result, (pos, is_target, o)| {
            if let Some((obs, id)) = o {
                result.push(AttackTarget {
                    pos,
                    obstacle: obs,
                    target: id.and_then(|id| w.get_actor(id).cloned()),
                    is_target,
                });
            }

            result
        })
}

fn is_next_to_enemy(a_team: &Team, a_pos: MapPos, w: &CoreWorld) -> bool {
    w.find_actor(|other| {
        let o_pos = MapPos::from_world_pos(other.pos);
        other.is_concious() && a_team != &other.team && a_pos.distance(o_pos) == 1
    })
    .is_some()
}

fn create_combat_fx(
    attacker: &Actor,
    attack_end_pos: WorldPos,
    combat_result: &HitResult,
) -> FxSequence {
    match &combat_result.attack.attack_type {
        AttackType::Melee(s) => {
            create_melee_combat_fx(s.to_string(), attacker, &combat_result.hits)
        }

        AttackType::Ranged(s) => create_ranged_combat_fx(
            s.to_string(),
            attacker.pos,
            attack_end_pos,
            &combat_result.hits,
        ),
    }
}

fn create_melee_combat_fx(attack_fx: String, attacker: &Actor, hits: &Vec<Hit>) -> FxSequence {
    let mut fx_seq = FxSequence::new();
    let attacker_id = attacker.id;
    let attacker_pos = attacker.pos;

    for hit in hits.iter() {
        let target_pos = hit.pos.to_world_pos();
        let move_steps = vec![attacker_pos, target_pos, attacker_pos];
        let attack_move_fx = FxEffect::move_along(attacker_id, move_steps);
        let attack_move_dur = attack_move_fx.duration();

        fx_seq = fx_seq
            .then(attack_move_fx)
            .wait(attack_move_dur.as_millis() as u64 / 2)
            .then(FxEffect::sprite(attack_fx.clone(), target_pos, 400));
    }

    if all_misses(hits) {
        fx_seq = fx_seq
            .wait_until_finished()
            .then(FxEffect::say("Curses!", attacker_pos))
            .wait(300)
    }

    fx_seq
}

fn is_miss(hit: &Hit) -> bool {
    for e in hit.effects.iter() {
        if let HitEffect::Miss() = e {
            return true;
        }
    }
    false
}

fn all_misses(hits: &Vec<Hit>) -> bool {
    hits.iter().fold(true, |result, h| result && is_miss(h))
}

fn create_hit_fx(effects: &Vec<HitEffect>, target_pos: WorldPos) -> FxSequence {
    let mut fx_seq = FxSequence::new();

    for eff in effects.iter() {
        fx_seq = match eff {
            HitEffect::Wound(wound, ..) => {
                fx_seq.then_append(create_fx_changes_for_wound(&wound, target_pos, 0))
            }

            HitEffect::Defence(..) => fx_seq.then(FxEffect::say("Defended", target_pos)),

            HitEffect::Miss() => fx_seq.then(FxEffect::say("Missed", target_pos)),

            _ => fx_seq,
        }
    }

    fx_seq
}

fn create_ranged_combat_fx(
    attack_fx: String,
    attacker_pos: WorldPos,
    last_pos: WorldPos,
    hits: &Vec<Hit>,
) -> FxSequence {
    let mut fx_seq = FxSequence::new();
    let attacker_mpos = MapPos::from_world_pos(attacker_pos);

    fx_seq = fx_seq.then(FxEffect::projectile(attack_fx, attacker_pos, last_pos));

    for hit in hits.iter() {
        let target_pos = hit.pos.to_world_pos();
        let impact = !is_miss(hit);

        if impact {
            let dur = 50 * attacker_mpos.distance(hit.pos) as u64;
            let impact_fx_dur = 300;

            fx_seq = fx_seq
                .wait(dur)
                .then(FxEffect::sprite("fx-impact-1", target_pos, impact_fx_dur))
                .then_insert(create_hit_fx(&hit.effects, target_pos));
        }
    }

    if all_misses(&hits) {
        fx_seq = fx_seq
            .wait_until_finished()
            .then(FxEffect::say("Curses!", attacker_pos))
            .wait(300)
    }

    fx_seq
}

fn create_fx_changes_for_wound(wound: &Wound, target_pos: WorldPos, delay: u64) -> FxSequence {
    let fx_seq = FxSequence::new().wait(delay);

    match wound {
        Wound { pain: 0, wound: 0 } => fx_seq.then(FxEffect::say("Klong", target_pos)),

        Wound { pain, wound: 0 } if *pain > 0 => fx_seq.then(FxEffect::say("Uff!", target_pos)),

        Wound { wound: 1, .. } => fx_seq
            .then(FxEffect::blood_splatter(target_pos))
            .wait(50)
            .then(FxEffect::say("Arrgh!", target_pos)),

        _ => fx_seq
            .then(FxEffect::blood_splatter(target_pos))
            .wait(50)
            .then(FxEffect::blood_splatter(target_pos))
            .wait(50)
            .then(FxEffect::scream("AIIEEE!", target_pos)),
    }
}

fn handle_attack<'a>(
    attacker_id: ID,
    target_id: ID,
    attack: AttackOption,
    mut cw: CoreWorld<'a>,
) -> ActionResultBuilder<'a> {
    let attacker = cw.get_actor(attacker_id).cloned();
    let target = cw.get_actor(target_id).cloned();

    if attacker.is_none() || target.is_none() {
        return ActionResultBuilder::new(cw);
    }

    let attacker = attacker.unwrap();
    let target = target.unwrap();
    let max_distance = attack.advance + attack.max_distance;
    let from = MapPos::from_world_pos(attacker.pos);
    let to = MapPos::from_world_pos(target.pos);

    if !attacker.is_concious() {
        return ActionResultBuilder::new(cw).add_fx(FxEffect::say("Ahhh", attacker.pos));
    }

    if from.distance(to) > max_distance.into() {
        return ActionResultBuilder::new(cw).add_fx(FxEffect::say("It's too far!", attacker.pos));
    }

    let partial_result = if attack.advance > 0 {
        let p = find_charge_path(&attacker, &target, &cw);
        if p.is_none() {
            return ActionResultBuilder::new(cw)
                .add_fx(FxEffect::say("The way is blocked!", attacker.pos));
        }

        let mut p = p.unwrap();
        p.pop(); // ignore last tile which is where the target is

        // TODO it is possible that attack.advance > 1 and attack.distance > 1. In
        // this case following code would not work correctly (it would move the
        // attacker all the way). So far I think an advance and reach attack makes no sence
        let p0 = from.to_world_pos();
        let p1 = p.last().unwrap().to_world_pos();

        cw.update_actor(attacker.charge_to(p1));

        let mut charge_fx = FxSequence::new()
            .then(FxEffect::scream("Charge!", p0))
            .wait(500)
            .then(FxEffect::move_along(attacker_id, vec![p0, p1]));

        for (i, tile) in p.iter().enumerate() {
            charge_fx = charge_fx.then(FxEffect::dust("fx-dust-1", tile.to_world_pos(), 400));

            if i < p.len() - 1 {
                charge_fx = charge_fx.wait(200)
            }
        }

        ActionResultBuilder::new(cw).append_fx_seq(charge_fx)
    } else {
        ActionResultBuilder::new(cw)
    };

    partial_result.chain(|cw| perform_attack(attacker_id, target_id, attack, cw))
}

fn perform_attack(
    attacker: ID,
    target: ID,
    attack_option: AttackOption,
    cw: CoreWorld,
) -> ActionResultBuilder {
    let attacker = cw.get_actor(attacker).cloned().unwrap();
    let target = cw.get_actor(target).cloned().unwrap();
    let v = attack_vector(&attacker, &target, &attack_option, &cw);

    // println!("\nATTACK VECTOR {:?}", v);

    if v.is_none() {
        // there are no targets or obstacles to hit
        // => cancel attack and do nothing
        return ActionResultBuilder::new(cw);
    }

    let v = v.unwrap();
    let attack_end_pos = v.last().unwrap().0.to_world_pos();
    let attack = attack_option.into_attack(&attacker);
    let attack_targets = filter_attack_vector(&v, &cw);

    // println!("Targets {:?}", attack_targets);

    let combat_result = resolve_combat(&attack, attack_targets);
    let combat_fx_seq = create_combat_fx(&attacker, attack_end_pos, &combat_result);

    let mut result = ActionResultBuilder::new(cw)
        .append_fx_seq(combat_fx_seq)
        .append_log(DisplayStr::new("TODO").into());

    for h in combat_result.hits {
        for eff in h.effects {
            result = result.chain(|w| apply_hit_effect(eff, w))
        }
    }

    result
}

fn apply_hit_effect(eff: HitEffect, mut cw: CoreWorld) -> ActionResultBuilder {
    match eff {
        HitEffect::Defence(roll, id) => {
            let mut fx_seq = FxSequence::new();
            if let Some(t) = cw.get_actor(id).cloned() {
                let target = t.use_effort(roll.num_successes + roll.num_fails);
                let target_pos = target.pos;

                cw.update(target.into());
                fx_seq = fx_seq.then(FxEffect::say("Blocked", target_pos));
            }

            ActionResultBuilder::new(cw).append_fx_seq(fx_seq)
        }

        HitEffect::Wound(w, id) => {
            let mut score = 0;
            let mut fx_seq = FxSequence::new();

            if let Some(t) = cw.get_actor(id).cloned() {
                fx_seq = fx_seq.then_append(create_fx_changes_for_wound(&w, t.pos, 0));

                let mut target = t.wound(w);

                if target.is_alive() {
                    if !target.is_concious() {
                        target = target.prepare(Act::rest());
                    }

                    cw.update(target.into());
                } else {
                    cw.remove(id);
                    cw.update(GameObject::Item(target.pos, target.corpse()));

                    score += 100;
                }
            }

            ActionResultBuilder::new(cw)
                .score(score)
                .append_fx_seq(fx_seq)
        }

        HitEffect::ForceMove {
            id,
            dx,
            dy,
            distance,
        } => force_move(id, dx, dy, distance, cw),

        _ => ActionResultBuilder::new(cw),
    }
}

fn take_step(actor_id: ID, target_pos: WorldPos, mut cw: CoreWorld) -> (ActionResultBuilder, bool) {
    let actor = cw.get_actor(actor_id);
    if actor.is_none() {
        // moving actor may have been killed by now
        return (ActionResultBuilder::new(cw), false);
    }

    let actor = actor.unwrap();
    let actor_pos = actor.pos;

    if !actor.can_move() {
        return (ActionResultBuilder::new(cw), false);
    }

    if is_next_to_enemy(&actor.team, MapPos::from_world_pos(actor_pos), &cw) {
        let mut actor = actor.clone();
        actor.engaged_in_combat = true;
        cw.update_actor(actor);
        return (ActionResultBuilder::new(cw), false);
    }

    (move_to(actor.clone(), target_pos, cw, true), true)
}

fn force_move(actor_id: ID, dx: i32, dy: i32, distance: u8, cw: CoreWorld) -> ActionResultBuilder {
    let actor = cw.get_actor(actor_id);
    if actor.is_none() {
        // moving actor may have been killed by now
        return ActionResultBuilder::new(cw);
    }

    let actor = actor.unwrap();
    let p1 = MapPos::from_world_pos(actor.pos);
    let p2 = MapPos(p1.0 + dx, p1.1 + dy);
    let obstacles = cw.collect_obstacles();
    let mut target_pos = p1;
    let mut positions_along = SuperLineIter::new(p1, p2);

    // ignore first pos because that is where the actor already is
    let _ = positions_along.next();

    for p in positions_along {
        if let Some(_) = obstacles.get(&p) {
            // stopped by an obstancle
            break;
        }

        if p1.distance(p) > distance.into() {
            break;
        }

        target_pos = p;
    }

    if p1.distance(target_pos) == 0 {
        ActionResultBuilder::new(cw)
    } else {
        move_to(actor.clone(), target_pos.to_world_pos(), cw, false)
    }
}

fn move_to(
    mut actor: Actor,
    target_pos: WorldPos,
    mut cw: CoreWorld,
    jump: bool,
) -> ActionResultBuilder {
    let actor_id = actor.id;
    let actor_pos = actor.pos;

    actor.pos = target_pos;
    actor.engaged_in_combat =
        is_next_to_enemy(&actor.team, MapPos::from_world_pos(target_pos), &cw);

    cw.update_actor(actor);

    let move_fx = if jump {
        FxEffect::jump(actor_id, vec![actor_pos, target_pos])
    } else {
        FxEffect::move_along(actor_id, vec![actor_pos, target_pos])
    };

    let move_fx_dur = move_fx.duration();
    let fx_seq = FxSequence::new()
        .then(move_fx)
        .wait(move_fx_dur.as_millis() as u64)
        .then(FxEffect::dust("fx-dust-1", target_pos, 400));

    ActionResultBuilder::new(cw)
        .append_fx_seq(fx_seq)
        .chain(|w| handle_post_move_actions(actor_id, w))
}

fn handle_post_move_actions(id: ID, mut world: CoreWorld) -> ActionResultBuilder {
    if let Some((ambusher, attack)) = world.find_map(|a| can_ambush(a, id, &world)) {
        if let Some(a) = world.get_actor(ambusher).cloned() {
            world.update_actor(a.prepare(Act::done("Did ambush ...")));
        }
        handle_attack(ambusher, id, attack, world).chain(|w| handle_post_move_actions(id, w))
    } else {
        ActionResultBuilder::new(world)
    }
}

fn can_ambush(obj: &GameObject, target: ID, world: &CoreWorld) -> Option<(ID, AttackOption)> {
    if let GameObject::Actor(ambusher) = obj {
        if let Some(Act {
            action: Action::Ambush(attack),
            ..
        }) = &ambusher.pending_action
        {
            if let Some(target) = world.get_actor(target) {
                if target.team == ambusher.team {
                    // do not ambush teammates
                    return None;
                }

                if can_attack_with(ambusher, target, attack, world) {
                    return Some((ambusher.id, attack.clone()));
                }
            }
        }
    }

    None
}
