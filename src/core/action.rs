use specs::prelude::*;

use crate::components::{FxEffect, FxSequence, GameObjectCmp, ObstacleCmp, Position};
use crate::core::ai::{attack_vector, find_movement_obstacles, AttackVector};
use crate::core::*;

use super::actors::*;
use super::dice::Roll;

#[derive(Debug, Clone)]
pub enum Action {
    StartTurn(),
    ResolvePendingActions(),
    EndTurn(Team),
    Done(String),
    MoveTo(Path),
    Activate(Entity),
    MeleeAttack(Entity, AttackOption, String),
    RangeAttack(Entity, AttackOption, AttackVector, String),
    Charge(Entity, AttackOption, String),
    UseAbility(Entity, String, Trait),
    Dodge(Tile),
}

impl Action {
    pub fn end_turn(t: Team) -> Act {
        (Self::EndTurn(t), 0)
    }

    pub fn done(msg: impl ToString) -> Act {
        (Self::Done(msg.to_string()), 0)
    }

    pub fn recover() -> Act {
        (Self::Done("Recovering".to_string()), 1)
    }

    pub fn activate(e: Entity) -> Act {
        (Self::Activate(e), 0)
    }

    pub fn move_to(p: Path) -> Act {
        (Self::MoveTo(p), 0)
    }

    pub fn melee_attack(target: Entity, attack: AttackOption, target_name: String) -> Act {
        (Self::MeleeAttack(target, attack, target_name), 1)
    }

    pub fn ranged_attack(target: Entity, ao: AttackOption, av: AttackVector, n: String) -> Act {
        (Self::RangeAttack(target, ao, av, n), 1)
    }

    pub fn charge(target: Entity, attack: AttackOption, n: String) -> Act {
        (Self::Charge(target, attack, n), 1)
    }

    pub fn dodge(to_pos: Tile) -> Act {
        (Self::Dodge(to_pos), 0)
    }

    pub fn use_ability(target: Entity, key: impl ToString, t: Trait, delay: u8) -> Act {
        (Self::UseAbility(target, key.to_string(), t), delay)
    }
}

pub enum Change {
    Update(Entity, GameObject),
    Insert(GameObject),
    Remove(Entity),
    Score(u64),
}

pub type Act = (Action, u8);
pub type EA = (Entity, Actor);
pub type ActionResult = (Vec<Change>, FxSequence, Option<DisplayStr>);

pub fn act((entity, actor, action, delay): (Entity, Actor, Action, u8), w: &World) -> ActionResult {
    if delay > 0 {
        (
            vec![update_actor(entity, actor.prepare((action, delay - 1)))],
            FxSequence::new(),
            None,
        )
    } else {
        run_action((entity, actor), action, w)
    }
}

pub fn run_action<'a>((entity, actor): EA, action: Action, w: &World) -> ActionResult {
    match action {
        Action::StartTurn() => {
            if let Some(actor) = get_actor(entity, w) {
                // we need to get the most current actor since this action is executed immediatly
                // after resolving pending actions
                let engaged_in_combat = check_engaged_in_combat(&actor, w);
                let updates = vec![update_actor(
                    entity,
                    actor.start_next_turn(engaged_in_combat),
                )];

                (updates, FxSequence::new(), None)
            } else {
                no_op()
            }
        }

        Action::ResolvePendingActions() => {
            if let Some(pending_action) = actor.clone().pending_action {
                let (action, delay) = pending_action;
                act((entity, actor, action.clone(), delay), w)
            } else {
                no_op()
            }
        }

        Action::EndTurn(team) => {
            let mut updates = vec![];
            let (entities, actors): (Entities, ReadStorage<GameObjectCmp>) = w.system_data();

            for (e, o) in (&entities, &actors).join() {
                if let GameObject::Actor(a) = &o.0 {
                    if a.team == team && a.pending_action.is_none() {
                        updates.push(update_actor(
                            e,
                            a.clone().prepare(Action::done("Waiting for next turn...")),
                        ));
                    }
                }
            }

            (updates, FxSequence::new(), None)
        }

        Action::Done(_) => no_op(),

        Action::UseAbility(target_entity, ability_name, t) => {
            if let Some(target_actor) = get_actor(target_entity, w) {
                let fx_pos = target_actor.pos.clone();
                let fx_str = t.name.to_string();
                let actor_name = target_actor.name.clone();
                let log = DisplayStr::new(format!("{} used ability: {}", actor_name, ability_name));
                let target_actor = target_actor.use_ability(ability_name, t);

                (
                    vec![update_actor(target_entity, target_actor)],
                    FxSequence::new().then(FxEffect::say(fx_str, fx_pos)),
                    Some(log),
                )
            } else {
                no_op()
            }
        }

        Action::MoveTo(path) => {
            if path.is_empty() {
                return no_op();
            }

            let sp = actor.pos;
            let na = actor
                .move_to(path.last().unwrap().to_map_pos())
                .prepare(Action::done("Did move..."));

            (
                vec![update_actor(entity, na)],
                FxSequence::new().then(FxEffect::jump(entity, get_steps(sp, path))),
                None,
            )
        }

        Action::Activate(target_e) => {
            if let Some(target_a) = get_actor(target_e, w) {
                (
                    vec![
                        update_actor(entity, actor.deactivate()),
                        update_actor(target_e, target_a.activate()),
                    ],
                    FxSequence::new(),
                    None,
                )
            } else {
                no_op()
            }
        }

        Action::RangeAttack(te, attack, _, _) => {
            if let Some(ta) = get_actor(te, w) {
                handle_attack(
                    (entity, actor),
                    (te, ta),
                    attack,
                    w,
                    vec![],
                    FxSequence::new(),
                )
            } else {
                no_op()
            }
        }

        Action::MeleeAttack(te, attack, _) => {
            if let Some(ta) = get_actor(te, w) {
                let move_steps = vec![actor.pos, ta.pos, actor.pos];
                let fx_seq = FxSequence::new()
                    .then(FxEffect::jump(entity, move_steps))
                    .wait(200);

                handle_attack((entity, actor), (te, ta), attack, w, vec![], fx_seq)
            } else {
                no_op()
            }
        }

        Action::Charge(target_entity, attack, _) => {
            if let Some(target_actor) = get_actor(target_entity, w) {
                let from = MapPos::from_world_pos(actor.pos);
                let to = MapPos::from_world_pos(target_actor.pos);
                let steps_needed = from.distance(to) - 1;
                let move_distance: usize = actor.move_distance().into();

                if steps_needed <= 0 || steps_needed > move_distance {
                    // cannot reach opponent
                    // => cancel charge
                    return no_op();
                }

                let (map, position_cmp, obstacle_cmp): (
                    Read<Map>,
                    ReadStorage<Position>,
                    ReadStorage<ObstacleCmp>,
                ) = w.system_data();

                let obstacles =
                    find_movement_obstacles(&position_cmp, &obstacle_cmp, &actor.team).ignore(to);

                if let Some(p) = map.find_straight_path(from, to, &obstacles) {
                    let tile = p[steps_needed - 1];
                    let p1 = actor.pos; // start movement at the original postion of the attacer
                    let p2 = p.last().unwrap().to_world_pos(); // step on the target tile to visualise impact
                    let p3 = tile.to_world_pos(); // one tile back to the place where the attacker actually stands
                    let actor = actor.charge_to(tile.to_map_pos());
                    let changes = vec![update_actor(entity, actor.clone())];
                    let fx_seq = FxSequence::new()
                        .then(FxEffect::scream("Charge!", p1))
                        .wait(200)
                        .then(FxEffect::dust("fx-dust-1", p1, 300))
                        .then(FxEffect::jump(entity, vec![p1, p2, p3]))
                        .wait(200);

                    return handle_attack(
                        (entity, actor),
                        (target_entity, target_actor),
                        attack,
                        w,
                        changes,
                        fx_seq,
                    );
                }
            }

            return no_op();
        }

        Action::Dodge(target_tile) => {
            let actor_pos = actor.pos;
            let actor = actor.use_ability(
                "ability#Dodge",
                Trait {
                    name: DisplayStr::new("Dodging"),
                    effects: vec![Effect::Defence(3, DefenceType::Dodge(target_tile.column(), target_tile.row()))],
                    source: TraitSource::Temporary(1),
                    visuals: None,
                },
            );

            let changes = vec![update_actor(entity, actor)];
            let fx_seq = FxSequence::new()
                .then(FxEffect::say("Dodging", actor_pos))
                .wait(300);

            (changes, fx_seq, None)
        }
    }
}

fn update_actor(e: Entity, a: Actor) -> Change {
    Change::Update(e, GameObject::Actor(a))
}

fn get_actor(e: Entity, w: &World) -> Option<Actor> {
    w.read_storage::<GameObjectCmp>()
        .get(e)
        .and_then(|o| match o {
            GameObjectCmp(GameObject::Actor(a)) => Some(a.clone()),
            _ => None,
        })
}

fn no_op() -> ActionResult {
    (vec![], FxSequence::new(), None)
}

fn add_changes_for_wound(
    attacker: Entity,
    hit: &Hit<Entity>,
    wound: &ToWoundResult,
    changes: &mut Vec<Change>,
) {
    let target_entity = hit.target;
    let mut target_actor = wound.target.clone();

    if let Some(defence) = &wound.defence {
        target_actor = match defence.0.defence_type {
            DefenceType::Dodge(col, row) => target_actor.move_to(MapPos(col as i32, row as i32)).remove_trait("ability#Dodge"),

            DefenceType::Parry => {
                if defence.1.successes() >= hit.successes() {
                    let attack = target_actor.melee_attack();
                    let name = target_actor.name.clone();

                    target_actor.prepare((Action::MeleeAttack(attacker, attack, name), 0))
                } else {
                    target_actor
                }
            }

            _ => target_actor,
        };
    }

    changes_for_condition(target_entity, target_actor, changes);
}

fn changes_for_condition(e: Entity, a: Actor, changes: &mut Vec<Change>) {
    if a.is_alive() {
        changes.push(update_actor(e, a));
    } else {
        changes.push(Change::Remove(e));
        changes.push(Change::Insert(GameObject::Item(a.pos, a.corpse())));
        changes.push(Change::Score(100));
    }
}

// fn add_fx_changes_for_attack(
//     attack: &Attack,
//     attack_vector: &AttackVector,
//     hits: &Vec<Hit<Entity>>,
//     mut fx_seq: FxSequence,
// ) -> FxSequence {
//     match &attack.attack_type {
//         AttackType::Melee(s) => {
//             for h in hits.iter() {
//                 fx_seq = fx_seq
//                     .then(FxEffect::sprite(s, h.pos.to_world_pos(), 400))
//                     .wait_until_finished();
//             }

//             fx_seq
//         }

//         AttackType::Ranged(s) => {
//             let first_pos = attack_vector.first().unwrap().0;
//             let mut last_pos = attack_vector.last().unwrap().0;

//             if let Some(hit) = hits.last() {
//                 let use_hit_pos = (hit.accicental_hit && hit.roll.normal_successes() == 0)
//                     || (!hit.accicental_hit && hit.roll.fails() == 0);

//                 if use_hit_pos {
//                     last_pos = hit.pos;
//                 }
//             }

//             fx_seq.then(FxEffect::projectile(
//                 s,
//                 first_pos.to_world_pos(),
//                 last_pos.to_world_pos(),
//             ))
//         }
//     }
// }

// fn add_fx_changes_for_hit<T>(
//     attacker_pos: WorldPos,
//     hit: &Hit<T>,
//     fx_seq: FxSequence,
// ) -> FxSequence {
//     if hit.successes() > 0 {
//         fx_seq
//             .then(FxEffect::sprite("fx-impact-1", hit.pos.to_world_pos(), 300))
//             .wait_until_finished()
//     } else {
//         fx_seq
//             .then(FxEffect::say("Curses!", attacker_pos))
//             .wait(300)
//     }
// }

fn get_steps(start: WorldPos, path: Path) -> Vec<WorldPos> {
    std::iter::once(start)
        .chain(path.iter().map(|t| t.to_world_pos()))
        .collect()
}

/// filter items where there is no actual target
fn filter_attack_vector<T: Clone>(
    input: &Vec<(MapPos, bool, Option<(T, Obstacle)>)>,
) -> Vec<(MapPos, T, Obstacle, bool)> {
    let mut input = input.to_vec();
    (&mut input)
        .drain(..)
        .fold(vec![], |mut result, (pos, is_target, o)| {
            if let Some((t, obs)) = o {
                result.push((pos, t, obs, is_target));
            }
            result
        })
}

fn handle_attack(
    attacker: (Entity, Actor),
    target: (Entity, Actor),
    attack_option: AttackOption,
    w: &World,
    mut changes: Vec<Change>,
    mut fx_seq: FxSequence,
) -> ActionResult {
    let log = DisplayStr::new("TODO");
    let v = attack_vector(&attacker.1, &target.1, &attack_option, w.system_data());

    if v.is_empty() {
        // there are no targets or obstacles to hit
        // => cancel attack and do nothing
        return no_op();
    }

    let attacker_pos = attacker.1.pos;
    let attack_end_pos = v.last().unwrap().0.to_world_pos();
    let attack = attack_option.into_attack(&attacker.1);
    let mut hits = resolve_to_hit(&attack, filter_attack_vector(&v));
    let mut fx_data = vec![];

    for h in hits.drain(..) {
        if let Some(target) = get_actor(h.target, w) {
            let wound = resolve_to_wound(h.clone().set_target(target));

            add_changes_for_wound(attacker.0, &h, &wound, &mut changes);

            fx_data.push((h, Some(wound)));
        } else {
            fx_data.push((h, None));
        }
    }

    let combat_fx_seq = create_combat_fx(attack, attacker_pos, attack_end_pos, fx_data);
    fx_seq = fx_seq.wait_until_finished().then_append(combat_fx_seq);

    (changes, fx_seq, Some(log))
}

fn check_engaged_in_combat(a: &Actor, w: &World) -> bool {
    let game_objects: ReadStorage<GameObjectCmp> = w.system_data();
    let a_pos = MapPos::from_world_pos(a.pos);

    for go in (game_objects).join() {
        if let GameObjectCmp(GameObject::Actor(other)) = go {
            let o_pos = MapPos::from_world_pos(other.pos);

            if a.team != other.team && a_pos.distance(o_pos) == 1 {
                // the current actor is next to an enemy
                // => it is engaged in melee combat
                return true;
            }
        }
    }

    false
}

type HitWoundData = Vec<(Hit<Entity>, Option<ToWoundResult>)>;

fn create_combat_fx(
    attack: Attack,
    attacker_pos: WorldPos,
    attack_end_pos: WorldPos,
    hits: HitWoundData,
) -> FxSequence {
    match attack.attack_type {
        AttackType::Melee(s) => create_melee_combat_fx(s, attacker_pos, hits),
        AttackType::Ranged(s) => create_ranged_combat_fx(s, attacker_pos, attack_end_pos, hits),
    }
}

fn create_melee_combat_fx(
    attack_fx: String,
    attacker_pos: WorldPos,
    hits: HitWoundData,
) -> FxSequence {
    let mut fx_seq = FxSequence::new();
    let mut all_misses = true;

    for (hit, wound) in hits.iter() {
        let target_pos = hit.pos.to_world_pos();

        fx_seq = fx_seq
            .then(FxEffect::sprite(attack_fx.clone(), target_pos, 400))
            .wait_until_finished();

        if hit.successes() > 0 {
            all_misses = false;
            fx_seq = fx_seq
                .then(FxEffect::sprite("fx-impact-1", target_pos, 300))
                .wait(50);
        }

        if let Some(wound) = wound {
            if let Some(defence) = &wound.defence {
                fx_seq = fx_seq
                    .then_append(create_defence_fx(
                        hit.target,
                        target_pos,
                        defence.clone(),
                        hit.roll.clone(),
                        0,
                    ))
                    .wait_until_finished();
            }

            fx_seq = fx_seq
                .then_append(create_fx_changes_for_wound(&wound.roll, target_pos, 0))
                .wait_until_finished();
        } else {
            fx_seq = fx_seq.wait_until_finished();
        }
    }

    if all_misses {
        fx_seq = fx_seq
            .then(FxEffect::say("Curses!", attacker_pos))
            .wait(300)
    }

    fx_seq
}

fn create_ranged_combat_fx(
    attack_fx: String,
    attacker_pos: WorldPos,
    last_pos: WorldPos,
    hits: HitWoundData,
) -> FxSequence {
    let mut fx_seq = FxSequence::new();
    let mut all_misses = true;
    let attacker_mpos = MapPos::from_world_pos(attacker_pos);

    fx_seq = fx_seq.then(FxEffect::projectile(attack_fx, attacker_pos, last_pos));

    for (hit, wound) in hits.iter() {
        let target_pos = hit.pos.to_world_pos();

        if hit.successes() > 0 {
            let dur = 50 * attacker_mpos.distance(hit.pos) as u64;
            let impact_fx_dur = 300;

            all_misses = false;

            fx_seq =
                fx_seq
                    .wait(dur)
                    .then(FxEffect::sprite("fx-impact-1", target_pos, impact_fx_dur));

            if let Some(wound) = wound {
                fx_seq =
                    fx_seq.then_append(create_fx_changes_for_wound(&wound.roll, target_pos, 50));
            }
        }
    }

    if all_misses {
        fx_seq = fx_seq
            .then(FxEffect::say("Curses!", attacker_pos))
            .wait(300)
    }

    fx_seq
}

fn create_fx_changes_for_wound(wound_roll: &Roll, target_pos: WorldPos, delay: u64) -> FxSequence {
    let fx_seq = FxSequence::new().wait(delay);

    match wound_roll.successes() {
        0 => fx_seq.then(FxEffect::say("Klong", target_pos)),

        1 => fx_seq.then(FxEffect::say("Uff!", target_pos)),

        2 => fx_seq
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

fn create_defence_fx(
    target_entity: Entity,
    target_pos: WorldPos,
    defence: (Defence, Roll),
    hit_roll: Roll,
    delay: u64,
) -> FxSequence {
    // println!("[DEBUG] handle_defence {:?}", defence);
    // println!("  - defence roll: ({}/{}) - {:?}", defence_roll.successes(), defence_roll.fails(), defence_roll);
    // println!("  - hit roll: ({}) - {:?}", hit_roll.successes(), hit_roll);
    let mut fx_seq = FxSequence::new().wait(delay);

    match defence.0.defence_type {
        DefenceType::Dodge(c, r) => {
            let dodge_path = vec![target_pos, MapPos(c as i32, r as i32).to_world_pos()];

            fx_seq = fx_seq
                .then(FxEffect::dust("fx-dust-1", target_pos, 300))
                .then(FxEffect::jump(target_entity, dodge_path))
                .then(FxEffect::say("Dodged!", target_pos))
                .wait(100);
        }

        DefenceType::Parry => {
            if defence.1.successes() >= hit_roll.successes() {
                fx_seq = fx_seq
                    .then(FxEffect::say("Counter Attack!", target_pos))
                    .wait(300);
            }
        }

        _ => {}
    }

    fx_seq
}
