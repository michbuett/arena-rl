use specs::prelude::*;

use crate::components::{Fx, GameObjectCmp, MovementModification, ObstacleCmp, Position};
use crate::core::ai::find_movement_obstacles;
use crate::core::*;

use super::actors::*;
use super::dice::Roll;

#[derive(Debug, Clone)]
pub enum Action {
    StartTurn(),
    Done(),
    MoveTo(Path),
    Activate(Entity),
    MeleeAttack(Entity, AttackOption),
    RangeAttack(Entity, AttackOption),
    Charge(Entity, AttackOption),
    UseAbility(Entity, DisplayStr, Trait),
    EndTurn(Team),
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

    pub fn ranged_attack(target: Entity, attack: AttackOption) -> Act {
        (Self::RangeAttack(target, attack), 1)
    }

    pub fn charge(target: Entity, attack: AttackOption) -> Act {
        (Self::Charge(target, attack), 1)
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
            let (actor, pending_action) = actor.start_next_turn();
            let mut updates = vec![update_actor(entity, actor.clone())];
            let mut log = None;

            if let Some(pending_action) = pending_action {
                let (action, delay) = pending_action;
                let (mut more_updates, log_entry) = act((entity, actor, action, delay), w);

                updates.append(&mut more_updates);
                log = log_entry;
            }

            (updates, log)
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

        Action::RangeAttack(target_entity, attack) => {
            if let Some(target_actor) = get_actor(target_entity, w) {
                let combat_result = super::actors::resolve_attack(attack, actor, target_actor);
                let mut changes = vec![];
                let log = changes_for_combat_result(
                    combat_result,
                    entity,
                    target_entity,
                    0,
                    &mut changes,
                );

                (changes, Some(log))
            } else {
                no_op()
            }
        }

        Action::MeleeAttack(target_entity, attack) => {
            if let Some(target_actor) = get_actor(target_entity, w) {
                let move_steps = vec![actor.pos, target_actor.pos, actor.pos];
                let from = MapPos::from_world_pos(actor.pos);
                let to = MapPos::from_world_pos(target_actor.pos);

                if from.distance(to) > attack.max_distance.into() {
                    // attacker cannot reach target => cancel attack
                    return no_op();
                }

                let mut changes = vec![fx_move(entity, move_steps, 0)];
                let log = {
                    // let combat_result = super::actors::combat(attack, actor, target_actor);
                    let combat_result = resolve_attack(attack, actor, target_actor);
                    changes_for_combat_result(
                        combat_result,
                        entity,
                        target_entity,
                        100,
                        &mut changes,
                    )
                };

                (changes, Some(log))
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
                            Effect::AttrMod(Attr::ToHit, 1),
                            Effect::AttrMod(Attr::ToWound, 1),
                            Effect::AttrMod(Attr::Defence, 1),
                        ],
                        source: TraitSource::Temporary(1),
                    }]);
                    let mut updates = vec![
                        update_actor(entity, actor.clone()),
                        fx_move(entity, vec![p1, p2, p3], 0),
                    ];
                    let combat_result = super::actors::resolve_attack(attack, actor, target_actor);
                    let log = changes_for_combat_result(
                        combat_result,
                        entity,
                        target_entity,
                        100,
                        &mut updates,
                    );

                    return (updates, Some(log));
                }
            }

            return no_op();
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

fn changes_for_combat_result(
    combat_result: CombatResult,
    attacker: Entity,
    target: Entity,
    mut fx_delay_ms: u64,
    mut changes: &mut Vec<Change>,
) -> DisplayStr {
    let attacker_pos = combat_result.attacker.pos;
    let target_pos = combat_result.target.pos;

    fx_delay_ms += add_fx_changes_for_hit(
        combat_result.attack,
        combat_result.hit_roll,
        attacker_pos,
        target_pos,
        fx_delay_ms,
        &mut changes,
    );

    add_fx_changes_for_wound(
        combat_result.wound_roll,
        attacker_pos,
        target_pos,
        fx_delay_ms,
        &mut changes,
    );

    changes_for_condition(attacker, combat_result.attacker, &mut changes);
    changes_for_condition(target, combat_result.target, &mut changes);

    DisplayStr::new("TODO")
}

fn add_fx_changes_for_hit(
    attack: Attack,
    hit_roll: Roll,
    attacker_pos: WorldPos,
    target_pos: WorldPos,
    mut fx_delay_ms: u64,
    mut changes: &mut Vec<Change>,
) -> u64 {
    match attack.attack_type {
        AttackType::Melee(sprite) => {
            fx_delay_ms += fx_sprite(sprite, target_pos, fx_delay_ms, 400, &mut changes);
        }

        AttackType::Ranged(sprite) => {
            fx_delay_ms += fx_projectile(sprite, attacker_pos, target_pos, fx_delay_ms, 250, &mut changes);
        }
    }

    if hit_roll.successes() == 0 {
            fx_delay_ms += fx_say("Curses!", attacker_pos, fx_delay_ms, &mut changes);
    }

    if hit_roll.successes() == 2 {
        fx_delay_ms += fx_say("Got ye!", attacker_pos, fx_delay_ms, &mut changes);
    }

    if hit_roll.successes() > 2 {
        fx_delay_ms += fx_scream("DIE!", attacker_pos, fx_delay_ms, &mut changes);
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
    changes.push(Change::Fx(Fx::projectile(sprite, from, to, delay, duration)));
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
