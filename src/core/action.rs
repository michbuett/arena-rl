use specs::prelude::*;

use crate::components::{Fx, GameObjectCmp, MovementModification, ObstacleCmp, Position};
use crate::core::ai::{attack_vector, find_movement_obstacles, AttackVector};
use crate::core::*;

use super::actors::*;
use super::dice::Roll;

#[derive(Debug, Clone)]
pub enum Action {
    StartTurn(),
    ResolvePendingActions(),
    EndTurn(Team),
    Done(),
    MoveTo(Path),
    Activate(Entity),
    MeleeAttack(Entity, AttackOption),
    RangeAttack(Entity, AttackOption, AttackVector),
    Charge(Entity, AttackOption),
    UseAbility(Entity, DisplayStr, Trait),
    Dodge(Tile),
}

impl Action {
    pub fn end_turn(t: Team) -> Act {
        (Self::EndTurn(t), 0)
    }

    pub fn done() -> Act {
        (Self::Done(), 0)
    }

    pub fn recover() -> Act {
        (Self::Done(), 1)
    }

    pub fn activate(e: Entity) -> Act {
        (Self::Activate(e), 0)
    }

    pub fn move_to(p: Path) -> Act {
        (Self::MoveTo(p), 0)
    }

    pub fn melee_attack(target: Entity, attack: AttackOption) -> Act {
        (Self::MeleeAttack(target, attack), 1)
    }

    pub fn ranged_attack(target: Entity, ao: AttackOption, av: AttackVector) -> Act {
        (Self::RangeAttack(target, ao, av), 1)
    }

    pub fn charge(target: Entity, attack: AttackOption) -> Act {
        (Self::Charge(target, attack), 1)
    }

    pub fn dodge(to_pos: Tile) -> Act {
        (Self::Dodge(to_pos), 0)
    }

    pub fn use_ability(target: Entity, name: DisplayStr, t: Trait, delay: u8) -> Act {
        (Self::UseAbility(target, name, t), delay)
    }
}

pub enum Change {
    Fx(Fx),
    Update(Entity, GameObject),
    Insert(GameObject),
    Remove(Entity),
}

pub type Act = (Action, u8);
pub type EA = (Entity, Actor);
pub type ActionResult = (Vec<Change>, Option<DisplayStr>);

pub fn act((entity, actor, action, delay): (Entity, Actor, Action, u8), w: &World) -> ActionResult {
    if delay > 0 {
        (
            vec![update_actor(entity, actor.prepare((action, delay - 1)))],
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

                (updates, None)
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
                        updates.push(update_actor(e, a.clone().prepare(Action::done())));
                    }
                }
            }

            (updates, None)
        }

        Action::Done() => no_op(),

        Action::UseAbility(target_entity, ability_name, t) => {
            if let Some(target_actor) = get_actor(target_entity, w) {
                let fx_pos = target_actor.pos.clone();
                let fx_str = t.name.to_string();
                let actor_name = target_actor.name.clone();

                (
                    vec![
                        update_actor(
                            target_entity,
                            target_actor
                                .add_traits(&mut vec![t])
                                .prepare(Action::done()),
                        ),
                        Change::Fx(Fx::say(DisplayStr::new(fx_str), fx_pos, 100, 1000)),
                    ],
                    Some(DisplayStr::new(format!(
                        "{} used ability: {}",
                        actor_name, ability_name
                    ))),
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
            (
                vec![
                    update_actor(
                        entity,
                        actor.move_to(*path.last().unwrap()).prepare(Action::done()),
                    ),
                    fx_move(entity, get_steps(sp, path), 0),
                ],
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
                    None,
                )
            } else {
                no_op()
            }
        }

        Action::RangeAttack(te, attack, _) => {
            if let Some(ta) = get_actor(te, w) {
                handle_attack((entity, actor), (te, ta), attack, w, 0, vec![])
            } else {
                no_op()
            }
        }

        Action::MeleeAttack(te, attack) => {
            if let Some(ta) = get_actor(te, w) {
                let move_steps = vec![actor.pos, ta.pos, actor.pos];
                let changes = vec![fx_move(entity, move_steps, 0)];

                handle_attack((entity, actor), (te, ta), attack, w, 200, changes)
            } else {
                no_op()
            }
        }

        Action::Charge(target_entity, attack) => {
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
                    let actor = actor.move_to(tile).add_traits(&mut vec![Trait {
                        name: DisplayStr::new("Charging"),
                        effects: vec![
                            Effect::AttrMod(Attr::ToWound, 1),
                            Effect::AttrMod(Attr::MeleeDefence, -1),
                        ],
                        source: TraitSource::Temporary(1),
                    }]);

                    let changes = vec![
                        update_actor(entity, actor.clone()),
                        fx_move(entity, vec![p1, p2, p3], 0),
                    ];

                    return handle_attack(
                        (entity, actor),
                        (target_entity, target_actor),
                        attack,
                        w,
                        200,
                        changes,
                    );
                }
            }

            return no_op();
        }

        Action::Dodge(target_pos) => {
            let actor_pos = actor.pos;
            let actor = actor.add_traits(&mut vec![Trait {
                name: DisplayStr::new("Dodging"),
                effects: vec![Effect::Defence(
                    DisplayStr::new("Dodge"),
                    3,
                    DefenceType::Dodge(target_pos),
                )],
                source: TraitSource::Temporary(1),
            }]).prepare(Action::done());

            let mut changes = vec![update_actor(entity, actor)];
            fx_say("Dodging", actor_pos, 0, &mut changes);

            (changes, None)
        }
    }
}

fn update_actor(e: Entity, a: Actor) -> Change {
    Change::Update(e, GameObject::Actor(a))
}

fn fx_move(e: Entity, p: Vec<WorldPos>, delay: u64) -> Change {
    let duration_ms = (p.len() - 1) as u64 * 200;
    Change::Fx(Fx::move_to(
        e,
        p,
        delay,
        duration_ms,
        MovementModification::ParabolaJump(96),
    ))
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
    (vec![], None)
}

fn changes_for_condition(e: Entity, a: Actor, changes: &mut Vec<Change>) {
    if a.is_alive() {
        changes.push(update_actor(e, a));
    } else {
        changes.push(Change::Remove(e));
        changes.push(Change::Insert(GameObject::Item(a.pos, a.corpse())));
    }
}

fn add_fx_changes_for_attack(
    attack: &Attack,
    attack_vector: &AttackVector,
    hits: &Vec<Hit<Entity>>,
    mut fx_delay_ms: u64,
    mut changes: &mut Vec<Change>,
) -> u64 {
    match &attack.attack_type {
        AttackType::Melee(sprite) => {
            for h in hits.iter() {
                fx_delay_ms +=
                    fx_sprite(sprite, h.pos.to_world_pos(), fx_delay_ms, 400, &mut changes);
            }
            fx_delay_ms
        }

        AttackType::Ranged(sprite) => {
            let first_pos = attack_vector.first().unwrap().0;
            let mut last_pos = attack_vector.last().unwrap().0;

            if let Some(hit) = hits.last() {
                let use_hit_pos = (hit.accicental_hit && hit.roll.normal_successes() == 0)
                    || (!hit.accicental_hit && hit.roll.fails() == 0);

                if use_hit_pos {
                    last_pos = hit.pos;
                }
            }

            let distance = first_pos.distance(last_pos);

            fx_projectile(
                sprite.to_string(),
                first_pos.to_world_pos(),
                last_pos.to_world_pos(),
                fx_delay_ms,
                (50 * distance) as u64,
                &mut changes,
            );

            0
        }
    }
}
fn add_fx_changes_for_hit<T>(
    attacker_pos: WorldPos,
    hit: &Hit<T>,
    mut fx_delay_ms: u64,
    mut changes: &mut Vec<Change>,
) -> u64 {
    if hit.successes() > 0 {
        fx_delay_ms += fx_sprite(
            "fx-impact-1",
            hit.pos.to_world_pos(),
            fx_delay_ms,
            300,
            &mut changes,
        );
    } else {
        fx_delay_ms += fx_say("Curses!", attacker_pos, fx_delay_ms, &mut changes);
    }

    fx_delay_ms
}

fn add_fx_changes_for_wound(
    wound_roll: Roll,
    attacker_pos: WorldPos,
    target_pos: WorldPos,
    mut fx_delay_ms: u64,
    mut changes: &mut Vec<Change>,
) -> u64 {
    if wound_roll.successes() == 0 {
        fx_delay_ms += fx_say("Klong", target_pos, fx_delay_ms, &mut changes);
    }

    if wound_roll.successes() == 1 {
        fx_delay_ms += fx_say("Uff!", target_pos, fx_delay_ms, &mut changes);
    }

    if wound_roll.successes() == 2 {
        fx_delay_ms += fx_blood(target_pos, fx_delay_ms, &mut changes);
        fx_delay_ms += fx_say("Arrgh!", target_pos, fx_delay_ms, &mut changes);
    }

    if wound_roll.successes() > 2 {
        fx_delay_ms += fx_blood(target_pos, fx_delay_ms, &mut changes);
        fx_delay_ms += fx_blood(target_pos, fx_delay_ms, &mut changes);
        fx_delay_ms += fx_scream("AIIEEE!", target_pos, fx_delay_ms, &mut changes);
        fx_delay_ms += fx_say("Yeah!", attacker_pos, fx_delay_ms, &mut changes);
    }

    fx_delay_ms
}

fn fx_say(s: &str, p: WorldPos, delay: u64, changes: &mut Vec<Change>) -> u64 {
    changes.push(Change::Fx(Fx::say(DisplayStr::new(s), p, delay, 1000)));
    300
}

fn fx_scream(s: &str, p: WorldPos, delay: u64, changes: &mut Vec<Change>) -> u64 {
    changes.push(Change::Fx(Fx::scream(DisplayStr::new(s), p, delay, 1000)));
    300
}

fn fx_sprite(
    sprite: impl ToString,
    p: WorldPos,
    delay: u64,
    duration: u64,
    changes: &mut Vec<Change>,
) -> u64 {
    changes.push(Change::Fx(Fx::sprite(sprite, p, delay, duration)));
    duration
}

fn fx_projectile(
    sprite: String,
    from: WorldPos,
    to: WorldPos,
    delay: u64,
    duration: u64,
    changes: &mut Vec<Change>,
) -> u64 {
    changes.push(Change::Fx(Fx::projectile(
        sprite, from, to, delay, duration,
    )));
    duration
}

fn fx_blood(p: WorldPos, delay: u64, changes: &mut Vec<Change>) -> u64 {
    changes.push(Change::Fx(Fx::rnd_blood_splatter(p, delay, 1000)));
    50
}

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
    mut delay: u64,
    mut changes: Vec<Change>,
) -> ActionResult {
    let log = DisplayStr::new("TODO");
    let v = attack_vector(&attacker.1, &target.1, &attack_option, w.system_data());

    if v.is_empty() {
        // there are no targets or obstacles to hit
        // => cancel attack and do nothing
        return no_op();
    }
    
    let attack = attack_option.into_attack(&attacker.1);
    let mut hits = resolve_to_hit(&attack, filter_attack_vector(&v));

    // println!("DEBUG ATTACK");
    // println!("hits={:?}", hits);
    // println!("attacker pos = {:?}", actor.pos);
    // println!("attack_vector = {:?}", v);
    delay = add_fx_changes_for_attack(&attack, &v, &hits, delay, &mut changes);

    for h in hits.drain(..) {
        delay = add_fx_changes_for_hit(attacker.1.pos, &h, delay, &mut changes);

        if let Some(target) = get_actor(h.target, w) {
            let target_entity = h.target;
            let hit_roll = h.roll.clone();
            let w = resolve_to_wound(h.set_target(target));

            if let Some(d) = w.defence {
                let a = (attacker.0, &attacker.1);
                let t = (target_entity, w.target);

                delay = handle_defence(a, t, d, hit_roll, delay, &mut changes);
            } else {
                delay = add_fx_changes_for_wound(
                    w.roll,
                    attacker.1.pos,
                    w.target.pos,
                    delay,
                    &mut changes,
                );

                changes_for_condition(target_entity, w.target, &mut changes);
            }
        }
    }
    (changes, Some(log))
}

fn handle_defence(
    attacker: (Entity, &Actor),
    target: (Entity, Actor),
    defence: (Defence, Roll),
    hit_roll: Roll,
    mut delay: u64,
    changes: &mut Vec<Change>,
) -> u64 {
    // println!("[DEBUG] handle_defence {:?}", defence);
    // println!("  - defence roll: ({}/{}) - {:?}", defence_roll.successes(), defence_roll.fails(), defence_roll);
    // println!("  - hit roll: ({}) - {:?}", hit_roll.successes(), hit_roll);
    let mut target_actor = target.1;

    match defence.0.defence_type {
        DefenceType::Dodge(tile) => {
            let target_pos = target_actor.pos;
            target_actor = target_actor
                .clone()
                .move_to(tile);

            changes.push(fx_move(target.0, vec![target_pos, tile.to_world_pos()], delay));
            delay += fx_say("Dodged", target_pos, delay, changes);
            delay += 100;
        }

        DefenceType::Parry => {
            if defence.1.successes() >= hit_roll.successes() {
                let attack = target_actor.melee_attack();
                let target_pos = target_actor.pos;

                target_actor = target_actor
                    .clone()
                    .prepare((Action::MeleeAttack(attacker.0, attack), 0));

                delay += fx_say("Counter Attack!", target_pos, delay, changes);
            }
        }

        _ => {}
    }

    changes_for_condition(target.0, target_actor, changes);

    delay
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
