use std::collections::HashMap;

use crate::components::{FxEffect, FxSequence};
use crate::core::ai::{attack_vector, AttackVector};
use crate::core::{DisplayStr, MapPos, Path, WorldPos};

use super::actors::{
    Actor, AttackFx, AttackOption, AttackTarget, GameObject, Hit, HitResult, Wound, ID,
};
use super::ai::find_charge_path;
use super::{resolve_combat_new, Card, CoreWorld, Deck, HitEffect, SuperLineIter, TeamId};

#[derive(Debug, Clone)]
pub enum Action {
    StartTurn(ID),
    BoostActivation(ID, Card),
    ActivateActor(ID),
    DoNothing(ID),

    MoveTo {
        actor: ID,
        path: Path,
    },

    Attack {
        attacker: ID,
        target: ID,
        attack: AttackOption,
        attack_vector: AttackVector,
        msg: String,
    },

    AddTrait {
        targets: Vec<ID>,
        trait_ref: String,
        msg: String,
    },

    SpawnActor {
        actor: Actor,
    },
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
        let (decks, updates) = self.world.into_changes();
        let mut fx_seq = FxSequence::new();

        for (id, go) in updates {
            if let Some(go) = go {
                fx_seq = fx_seq.then(FxEffect::Update(go));
            } else {
                fx_seq = fx_seq.then(FxEffect::Remove(id));
            }
        }

        ActionResult {
            decks,
            fx_seq: self.fx_seq.then_append(fx_seq),
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
    pub decks: Option<HashMap<TeamId, Deck>>,
    pub fx_seq: FxSequence,
    pub log: Option<DisplayStr>,
    pub score: u64,
}

pub fn run_player_action<'a>(action: Action, mut cw: CoreWorld) -> ActionResult {
    let result_builder = match action {
        Action::StartTurn(actor_id) => {
            if let Some(a) = cw.get_actor(actor_id) {
                // println!("[DEBUG] run_player_action - StartTurn {}", a.name);

                let a = a.clone();
                let deck = cw.decks_mut().get_mut(&a.team).unwrap();
                let a = a.start_next_turn(deck);

                // for _ in 1..=a.num_activation() {
                //     a = a.add_activation(deck.deal());
                // }

                cw.update_actor(a);
            }

            ActionResultBuilder::new(cw)
        }

        Action::BoostActivation(actor_id, card) => {
            cw.modify_actor(actor_id, |a| a.boost_activation(card));
            ActionResultBuilder::new(cw)
        }

        Action::ActivateActor(id) => {
            if let Some(actor) = cw.find_actor(|a| a.active) {
                cw.modify_actor(actor.id, |a| a.deactivate());
            }

            cw.modify_actor(id, |a| a.activate());

            ActionResultBuilder::new(cw)
        }

        Action::DoNothing(actor_id) => {
            cw.modify_actor(actor_id, Actor::done);
            ActionResultBuilder::new(cw)
        }

        Action::MoveTo { actor, path } => handle_move_action(actor, path, cw),

        Action::Attack {
            attacker,
            target,
            attack,
            ..
        } => handle_attack(attacker, target, attack, cw),

        Action::AddTrait {
            targets, trait_ref, ..
        } => handle_add_trait(targets, trait_ref, cw),

        Action::SpawnActor { actor } => {
            let fx = FxSequence::new().then(FxEffect::dust("fx-dust-1", actor.pos, 400));

            cw.update_actor(actor);

            ActionResultBuilder::new(cw).append_fx_seq(fx)
        }
    };

    result_builder.into_result()
}

/// filter items where there is no actual target
fn filter_attack_vector(input: &AttackVector, w: &CoreWorld) -> Vec<AttackTarget> {
    let mut input = input.to_vec();
    (&mut input)
        .drain(..)
        .fold(vec![], |mut result, (pos, is_target, cover, id)| {
            result.push(AttackTarget {
                pos,
                cover,
                actor: id.and_then(|id| w.get_actor(id).cloned()),
                is_target,
            });

            result
        })
}

fn create_combat_fx(
    attacker: &Actor,
    attack_end_pos: WorldPos,
    combat_result: &HitResult,
) -> FxSequence {
    match &combat_result.attack.attack_fx {
        AttackFx::MeleeSingleTarget { name } => {
            create_melee_combat_fx(name.to_string(), attacker, &combat_result.hits)
        }

        AttackFx::Projectile { name } => create_ranged_combat_fx(
            name.to_string(),
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

            HitEffect::Block(pos, ..) => fx_seq.then(FxEffect::say("Defended", pos.to_world_pos())),

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
    let projectile_speed = 50;

    fx_seq = fx_seq.then(FxEffect::projectile(
        attack_fx,
        attacker_pos,
        last_pos,
        projectile_speed,
    ));

    for hit in hits.iter() {
        let target_pos = hit.pos.to_world_pos();
        let impact = !is_miss(hit);

        if impact {
            let dur = projectile_speed * attacker_mpos.distance(hit.pos) as u64;
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

    fx_seq.wait_until_finished()
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
            .wait(5)
            .then(FxEffect::blood_splatter(target_pos))
            .wait(50)
            .then(FxEffect::scream("AIIEEE!", target_pos)),
    }
}

// fn create_fx_changes_for_kill(a: &Actor, target_pos: WorldPos, delay: u64) -> FxSequence {
//     let mut fx_seq = FxSequence::new();

//     for vis_str in a.visuals() {
//         fx_seq = fx_seq.then(
//             FxEffect::custom(target_pos, delay + 2000)
//                 .sprite(vis_str)
//                 .fade_out()
//                 .build(),
//         );
//     }

//     fx_seq.wait(delay + 2000)
//     // .then(FxEffect::scream("AIIEEE!", target_pos))
//     // .then(FxEffect::blood_splatter(target_pos))
//     // .wait(5)
//     // .then(FxEffect::blood_splatter(target_pos))
//     // .wait(5)
//     // .then(FxEffect::blood_splatter(target_pos))
//     // .wait(50)
// }

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

    let mut attacker = attacker.unwrap();
    let target = target.unwrap();
    let max_distance = attacker.move_distance() + attack.max_distance;
    let from = MapPos::from_world_pos(attacker.pos);
    let to = MapPos::from_world_pos(target.pos);
    let d = from.distance(to);

    if !attacker.is_concious() {
        return ActionResultBuilder::new(cw).add_fx(FxEffect::say("Ahhh", attacker.pos));
    }

    if d > max_distance.into() {
        return ActionResultBuilder::new(cw).add_fx(FxEffect::say("It's too far!", attacker.pos));
    }

    let advance_distance = d.checked_sub(attack.max_distance.into()).unwrap_or(0);
    let partial_result = if advance_distance > 0 {
        let path = find_charge_path(&attacker, &target, &cw);
        if path.is_none() {
            return ActionResultBuilder::new(cw)
                .add_fx(FxEffect::say("The way is blocked!", attacker.pos));
        }

        let path = path.unwrap();
        let pos_start = from.to_world_pos();
        let pos_end = path.get(advance_distance).unwrap().to_world_pos();

        attacker.pos = pos_end;

        cw.update_actor(attacker);

        let mut charge_fx = FxSequence::new()
            .then(FxEffect::scream("Charge!", pos_start))
            .wait(500)
            .then(FxEffect::move_along(attacker_id, vec![pos_start, pos_end]));

        for (i, tile) in path.iter().enumerate() {
            charge_fx = charge_fx.then(FxEffect::dust("fx-dust-1", tile.to_world_pos(), 400));

            if i < path.len() - 1 {
                charge_fx = charge_fx.wait(200)
            }
        }

        ActionResultBuilder::new(cw).append_fx_seq(charge_fx)
    } else {
        ActionResultBuilder::new(cw)
    };

    partial_result
        .chain(|cw| perform_attack(attacker_id, target_id, attack, cw))
        .chain(|cw| post_attack_cleanup(target_id, cw))
        .chain(|mut cw| {
            cw.modify_actor(attacker_id, Actor::done);
            ActionResultBuilder::new(cw)
        })
}

fn perform_attack(
    attacker: ID,
    target: ID,
    attack_option: AttackOption,
    mut cw: CoreWorld,
) -> ActionResultBuilder {
    let attacker = cw.get_actor(attacker).cloned().unwrap();
    let target = cw.get_actor(target).cloned().unwrap();
    let v = attack_vector(&attacker, &target, &attack_option, &cw);

    // println!("\nATTACK VECTOR {:?}", v);

    if v.as_ref().map(|v| v.len()).unwrap_or(0) == 0 {
        // there are no targets or obstacles to hit
        // => cancel attack and do nothing
        return ActionResultBuilder::new(cw);
    }

    let v = v.unwrap();
    let attack_end_pos = v.last().unwrap().0.to_world_pos();
    let attack = attack_option.into_attack(&attacker);
    let attack_targets = filter_attack_vector(&v, &cw);

    // println!("Targets {:?}", attack_targets);
    // let mut attacker_deck = cw.deck(attacker.team);
    // let mut target_deck = cw.deck(target.team);
    let combat_result = resolve_combat_new(&attack, &attacker, attack_targets, cw.decks_mut());

    // let combat_result = resolve_combat(&attack, attack_targets);
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
        HitEffect::Block(mpos, id) => {
            let mut fx_seq = FxSequence::new();
            if let Some(t) = cw.get_actor(id).cloned() {
                let target = t;
                // let target = t.use_effort(1); // TODO pass number of blocks
                let target_pos = mpos.to_world_pos();

                cw.update(target.into());
                fx_seq = fx_seq.then(FxEffect::say("Blocked", target_pos));
            }

            ActionResultBuilder::new(cw).append_fx_seq(fx_seq)
        }

        HitEffect::Wound(w, id) => {
            let mut score = 0;
            let mut fx_seq = FxSequence::new();

            if let Some(t) = cw.get_actor(id).cloned() {
                fx_seq = fx_seq.then_insert(create_fx_changes_for_wound(&w, t.pos, 0));

                let target = t.wound(w);

                fx_seq = fx_seq.then(FxEffect::Update(GameObject::Actor(target.clone())));

                if target.is_alive() {
                    cw.update(target.into());
                } else {
                    cw.remove(id);
                    score += 100;
                }

                fx_seq = fx_seq.wait_until_finished();
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

fn handle_move_action(actor_id: ID, path: Path, cw: CoreWorld) -> ActionResultBuilder {
    if path.is_empty() {
        return ActionResultBuilder::new(cw);
    }

    let mut result_builder = ActionResultBuilder::new(cw);

    for t in path {
        result_builder = result_builder.chain(|w| take_step(actor_id, t.to_world_pos(), w));
    }

    result_builder.chain(|mut w| {
        w.modify_actor(actor_id, Actor::done);
        ActionResultBuilder::new(w)
    })
}

fn take_step(actor_id: ID, target_pos: WorldPos, cw: CoreWorld) -> ActionResultBuilder {
    let actor = cw.get_actor(actor_id);
    if actor.is_none() {
        // moving actor may have been killed by now
        return ActionResultBuilder::new(cw);
    }

    let actor = actor.unwrap();

    if !actor.can_move() {
        return ActionResultBuilder::new(cw);
    }

    move_to(actor.clone(), target_pos, cw, true)
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

    cw.update_actor(actor);

    // let eff_seq = ActionEffectSeq::new().add(ActionEffect::move_actor(
    // actor_id, jump, actor_pos, target_pos,
    // ));
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

    ActionResultBuilder::new(cw).append_fx_seq(fx_seq)
    // .append_effects(eff_seq)
    // .chain(|w| handle_aoo(actor_id, w))
}

// /// Check for attacks of opportunities (aoo)
// fn handle_aoo(moving_actor_id: ID, mut world: CoreWorld) -> ActionResultBuilder {
//     if let Some((attacker, attack)) = world.find_map(|go| can_attack(go, moving_actor_id, &world)) {
//         // println!("[DEBUG] handle_aoo attacker={:?} target={:?}", attacker, moving_actor_id);

//         world.modify_actor(attacker, |mut a| {
//             a.prepared_action = None;
//             a
//         });

//         handle_attack(attacker, moving_actor_id, attack, world)
//             .chain(|w| handle_aoo(moving_actor_id, w))
//     } else {
//         ActionResultBuilder::new(world)
//     }
// }

// fn can_attack(obj: &GameObject, target: ID, _world: &CoreWorld) -> Option<(ID, AttackOption)> {
//     if let GameObject::Actor(attacker) = obj {
//         match &attacker.prepared_action {
//             Some(ActorAction::Attack {
//                 target: attacker_target,
//                 attack,
//                 ..
//             }) => {
//                 if target == *attacker_target {
//                     return Some((attacker.id, attack.clone()));
//                 }
//             }

//             // Some(Act {
//             //     action: Action::Ambush(attack),
//             //     ..
//             // }) => {
//             //     if let Some(target) = world.get_actor(target) {
//             //         if target.team == attacker.team {
//             //             // do not ambush teammates
//             //             return None;
//             //         }

//             //         if can_attack_with(attacker, target, attack, world) {
//             //             return Some((attacker.id, attack.clone()));
//             //         }
//             //     }
//             // }
//             _ => {}
//         }
//     }

//     None
// }

fn post_attack_cleanup(_attacked_target: ID, cw: CoreWorld) -> ActionResultBuilder {
    // TODO: this may become neccessary again when adding features like overwath or ambush

    //     if cw.get_actor(attacked_target).is_none() {
    //         // the target has been eliminated
    //         // remove the perpared action of every other attacker

    //         let other_attackers = cw
    //             .game_objects()
    //             .filter_map(|go| {
    //                 if let GameObject::Actor(other_actor) = go {
    //                     if let Some(ActorAction::Attack { target, .. }) =
    //                         other_actor.prepared_action.as_ref()
    //                     {
    //                         if *target == attacked_target {
    //                             return Some(other_actor.id);
    //                         }
    //                     }
    //                 }

    //                 None
    //             })
    //             .collect::<Vec<_>>();

    //         for a_id in other_attackers {
    //             cw.modify_actor(a_id, Actor::cancel_action);
    //             // cw.modify_actor(a_id, |mut a| {
    //             //     a.prepared_action = None;
    //             //     a.state = ReadyState::SelectAction;
    //             //     a
    //             // });
    //         }
    //     }

    ActionResultBuilder::new(cw)
}

fn handle_add_trait(
    targets: Vec<ID>,
    trait_ref: String,
    mut world: CoreWorld,
) -> ActionResultBuilder {
    let t = world.traits().get(&trait_ref).clone();

    for id in targets.iter() {
        world.modify_actor(*id, |a| a.add_trait(trait_ref.clone(), t.clone()).done());
    }

    ActionResultBuilder::new(world)
}
