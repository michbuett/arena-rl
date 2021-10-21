use specs::prelude::{Entities, Entity, Join, Read, ReadStorage, World, WorldExt};

use crate::components::{FxEffect, FxSequence, GameObjectCmp, ObstacleCmp, Position};
use crate::core::ai::{attack_vector, find_movement_obstacles, AttackVector};
use crate::core::{DisplayStr, Map, MapPos, Obstacle, Path, Tile, WorldPos};

use super::actors::{
    resolve_combat, Actor, AttackOption, AttackTarget, AttackType, GameObject,
    Hit, HitResult, Team, Trait, Wound,
};

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
    Ambush(AttackOption),
    UseAbility(Entity, String, Trait),
    Disengage(Tile),
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
        Self::new(Action::Done("Recovering".to_string())).delay(1)
    }

    pub fn activate(e: Entity) -> Self {
        Self::new(Action::Activate(e))
    }

    pub fn move_to(p: Path) -> Self {
        Self::new(Action::MoveTo(p))
    }

    pub fn melee_attack(
        target: Entity,
        attack: AttackOption,
        target_name: String,
        effort: u8,
    ) -> Self {
        Self::new(Action::MeleeAttack(target, attack, target_name))
            .delay(1)
            .effort(effort)
    }

    pub fn ranged_attack(
        target: Entity,
        ao: AttackOption,
        av: AttackVector,
        n: String,
        effort: u8,
    ) -> Self {
        Self::new(Action::RangeAttack(target, ao, av, n))
            .delay(1)
            .effort(effort)
    }

    pub fn charge(target: Entity, attack: AttackOption, n: String) -> Self {
        Self::new(Action::Charge(target, attack, n)).effort(0)
    }

    pub fn ambush(attack: AttackOption) -> Self {
        Self::new(Action::Ambush(attack)).delay(1).effort(0)
    }

    pub fn disengage(to_pos: Tile) -> Self {
        Self::new(Action::Disengage(to_pos)).delay(1).effort(0)
    }

    pub fn use_ability(target: Entity, key: impl ToString, t: Trait, delay: u8) -> Self {
        Self::new(Action::UseAbility(target, key.to_string(), t)).delay(delay)
    }
}

pub enum Change {
    Update(Entity, GameObject),
    Insert(GameObject),
    Remove(Entity),
    Score(u64),
}

pub type EA = (Entity, Actor);
pub type ActionResult = (Vec<Change>, FxSequence, Option<DisplayStr>);

pub fn act(
    (entity, actor, a): (Entity, Actor, Act),
    // (entity, actor, action, delay, effort): (Entity, Actor, Action, u8, u8),
    w: &World,
) -> ActionResult {
    let delay = a.delay;
    if delay > 0 {
        (
            vec![update_actor(entity, actor.prepare(a.delay(delay - 1)))],
            FxSequence::new(),
            None,
        )
    } else {
        run_action((entity, actor), a, w)
    }
}

pub fn run_action<'a>((entity, actor): EA, a: Act, w: &World) -> ActionResult {
    match a.action {
        Action::StartTurn() => {
            if let Some(actor) = get_actor(entity, w) {
                // we need to get the most current actor since this action is executed immediatly
                // after resolving pending actions
                let a_pos = MapPos::from_world_pos(actor.pos);
                let engaged_in_combat = is_next_to_enemy(&actor.team, a_pos, w);
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
                act((entity, actor, pending_action), w)
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
                            a.clone().prepare(Act::done("Waiting for next turn...")),
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
            let na = actor.move_along(&path).prepare(Act::done("Did move..."));

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
                let effort = a.allocated_effort.unwrap_or(actor.available_effort());
                handle_attack(
                    (entity, actor, effort),
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
                let effort = a.allocated_effort.unwrap_or(actor.available_effort());
                let fx_seq = FxSequence::new()
                    .then(FxEffect::jump(entity, move_steps))
                    .wait(200);

                handle_attack((entity, actor, effort), (te, ta), attack, w, vec![], fx_seq)
            } else {
                no_op()
            }
        }

        Action::Charge(target_entity, attack, _) => {
            if let Some(target_actor) = get_actor(target_entity, w) {
                let from = MapPos::from_world_pos(actor.pos);
                let to = MapPos::from_world_pos(target_actor.pos);
                let (map, position_cmp, obstacle_cmp): (
                    Read<Map>,
                    ReadStorage<Position>,
                    ReadStorage<ObstacleCmp>,
                ) = w.system_data();

                let obstacles =
                    find_movement_obstacles(&position_cmp, &obstacle_cmp, &actor.team).ignore(to);

                if let Some(p) = map.find_straight_path(from, to, &obstacles) {
                    if p.len() != 2 {
                        // incorrect charge distance
                        // => cancel charge
                        return no_op();
                    }

                    let from = actor.pos; 
                    let to = p[0].to_world_pos(); 
                    let effort = actor.available_effort();
                    let attack = attack.inc_difficulty(3); // TODO: use tile/obstacles to determine difficulty
                    let attack_act = Act::melee_attack(target_entity, attack, target_actor.name.clone(), effort).delay(0);
                    let actor = actor.charge_to(to).prepare(attack_act);
                    let changes = vec![update_actor(entity, actor.clone())];
                    let fx_seq = FxSequence::new()
                        .then(FxEffect::scream("Charge!", from))
                        .wait(300)
                        .then(FxEffect::dust("fx-dust-1", from, 300))
                        .then(FxEffect::jump(entity, vec![from, to]))
                        .wait(200);

                    return (changes, fx_seq, None);
                }
            }

            return no_op();
        }

        Action::Ambush(attack) => {
            if !actor.can_move() {
                return no_op();
            }
            
            if let Some((target_e, target_a, to_pos)) = find_ambush_enemy(&actor, &w) {
                let from_pos = actor.pos;
                let actor = actor.charge_to(to_pos);
                let changes = vec![update_actor(entity, actor.clone())];
                let attack = attack.inc_difficulty(3); // TODO: use tile/obstacles to determine difficulty
                let effort = actor.available_effort();
                let fx_seq = FxSequence::new()
                    .then(FxEffect::scream("Charge!", from_pos))
                    .wait(300)
                    .then(FxEffect::dust("fx-dust-1", from_pos, 300))
                    .then(FxEffect::jump(entity, vec![from_pos, target_a.pos, to_pos]))
                    .wait(200);

                return handle_attack(
                    (entity, actor, effort),
                    (target_e, target_a),
                    attack,
                    w,
                    changes,
                    fx_seq,
                );
            }

            no_op()
        }
        
        Action::Disengage(target_tile) => {
            let mut next_candidate = Some(target_tile);
            let (map, position_cmp, obstacle_cmp): (
                Read<Map>,
                ReadStorage<Position>,
                ReadStorage<ObstacleCmp>,
            ) = w.system_data();

            let obstacles = find_movement_obstacles(&position_cmp, &obstacle_cmp, &actor.team);
            let from = map.find_tile(actor.pos).unwrap();
            let mut neighbors = map.neighbors(from, &obstacles);

            while let Some(t) = next_candidate {
                let p = t.to_map_pos();
                if obstacles.0.contains_key(&p) || is_next_to_enemy(&actor.team, p, w) {
                    next_candidate = neighbors.next();
                } else {
                    let from_pos = actor.pos;
                    let to_pos = t.to_world_pos();
                    let changes = vec![update_actor(entity, actor.move_along(&vec![t]))];
                    let fx_seq = FxSequence::new()
                        .then(FxEffect::jump(entity, vec![from_pos, to_pos]))
                        .wait_until_finished();

                    return (changes, fx_seq, None);
                }
            }

            let fx_seq = FxSequence::new()
                .then(FxEffect::say("I cannot get away!", actor.pos))
                .wait(500);

            (vec![], fx_seq, None)
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

fn changes_for_condition(e: Entity, a: Actor, changes: &mut Vec<Change>) {
    if a.is_alive() {
        changes.push(update_actor(e, a));
    } else {
        changes.push(Change::Remove(e));
        changes.push(Change::Insert(GameObject::Item(a.pos, a.corpse())));
        changes.push(Change::Score(100));
    }
}

fn get_steps(start: WorldPos, path: Path) -> Vec<WorldPos> {
    std::iter::once(start)
        .chain(path.iter().map(|t| t.to_world_pos()))
        .collect()
}

/// filter items where there is no actual target
fn filter_attack_vector(
    input: &Vec<(MapPos, bool, Option<(Entity, Obstacle)>)>,
    w: &World,
) -> Vec<AttackTarget<Entity>> {
    let mut input = input.to_vec();
    (&mut input)
        .drain(..)
        .fold(vec![], |mut result, (pos, is_target, o)| {
            if let Some((t, obs)) = o {
                result.push(AttackTarget {
                    pos,
                    target_ref: t,
                    obstacle: obs,
                    target_actor: get_actor(t, w),
                    is_target,
                });
            }

            result
        })
}

fn handle_attack(
    attacker: (Entity, Actor, u8),
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
    let attack = attack_option.into_attack(&attacker.1, attacker.2);
    let attack_targets = filter_attack_vector(&v, w);
    let combat_result = resolve_combat(&attack, attack_targets);
    let combat_fx_seq = create_combat_fx(attacker_pos, attack_end_pos, &combat_result);

    for h in combat_result.hits {
        if let Some(w) = h.wound {
            changes_for_condition(h.target, w.target, &mut changes);
        }
    }

    fx_seq = fx_seq.wait_until_finished().then_append(combat_fx_seq);

    (changes, fx_seq, Some(log))
}

fn is_next_to_enemy(a_team: &Team, a_pos: MapPos, w: &World) -> bool {
    let game_objects: ReadStorage<GameObjectCmp> = w.system_data();

    for go in (game_objects).join() {
        if let GameObjectCmp(GameObject::Actor(other)) = go {
            let o_pos = MapPos::from_world_pos(other.pos);

            if a_team != &other.team && a_pos.distance(o_pos) == 1 {
                // the current actor is next to an enemy
                // => it is engaged in melee combat
                return true;
            }
        }
    }

    false
}

fn create_combat_fx(
    attacker_pos: WorldPos,
    attack_end_pos: WorldPos,
    combat_result: &HitResult<Entity>,
) -> FxSequence {
    match &combat_result.attack.attack_type {
        AttackType::Melee(s) => {
            create_melee_combat_fx(s.to_string(), attacker_pos, &combat_result.hits)
        }

        AttackType::Ranged(s) => create_ranged_combat_fx(
            s.to_string(),
            attacker_pos,
            attack_end_pos,
            &combat_result.hits,
        ),
    }
}

fn create_melee_combat_fx(
    attack_fx: String,
    attacker_pos: WorldPos,
    hits: &Vec<Hit<Entity>>,
) -> FxSequence {
    let mut fx_seq = FxSequence::new();
    let mut all_misses = true;

    for hit in hits.iter() {
        let target_pos = hit.pos.to_world_pos();

        fx_seq = fx_seq
            .then(FxEffect::sprite(attack_fx.clone(), target_pos, 400))
            .wait_until_finished();

        if let Some(wound) = &hit.wound {
            all_misses = false;

            if let Some(wound) = &wound.wound {
                fx_seq = fx_seq
                    .then(FxEffect::sprite("fx-impact-1", target_pos, 300))
                    .wait(50)
                    .then_append(create_fx_changes_for_wound(wound, target_pos, 0))
                    .wait_until_finished();
            } else {
                fx_seq = fx_seq
                    .then(FxEffect::say("Defended", target_pos))
                    .wait_until_finished();
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

fn create_ranged_combat_fx(
    attack_fx: String,
    attacker_pos: WorldPos,
    last_pos: WorldPos,
    hits: &Vec<Hit<Entity>>,
) -> FxSequence {
    let mut fx_seq = FxSequence::new();
    let mut all_misses = true;
    let attacker_mpos = MapPos::from_world_pos(attacker_pos);

    fx_seq = fx_seq.then(FxEffect::projectile(attack_fx, attacker_pos, last_pos));

    for hit in hits.iter() {
        let target_pos = hit.pos.to_world_pos();
        let impact = hit.accicental_hit || hit.wound.is_some();

        if impact {
            let dur = 50 * attacker_mpos.distance(hit.pos) as u64;
            let impact_fx_dur = 300;

            fx_seq =
                fx_seq
                    .wait(dur)
                    .then(FxEffect::sprite("fx-impact-1", target_pos, impact_fx_dur));
        }

        if let Some(wound) = &hit.wound {
            if let Some(wound) = &wound.wound {
                all_misses = false;
                fx_seq = fx_seq.then_append(create_fx_changes_for_wound(wound, target_pos, 50));
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

fn find_ambush_enemy(attacker: &Actor, w: &World) -> Option<(Entity, Actor, WorldPos)> {
    let (entities, goc_storage, map, position_cmp, obstacle_cmp): (
        Entities,
        ReadStorage<GameObjectCmp>,
        Read<Map>,
        ReadStorage<Position>,
        ReadStorage<ObstacleCmp>,
    ) = w.system_data();

    let attacker_mpos = MapPos::from_world_pos(attacker.pos);
    let obstacles = find_movement_obstacles(&position_cmp, &obstacle_cmp, &attacker.team);

    for (e, goc) in (&entities, &goc_storage).join() {
        if let GameObjectCmp(GameObject::Actor(a)) = goc {
            let target_mpos = MapPos::from_world_pos(a.pos);
            let d = target_mpos.distance(attacker_mpos);

            if a.team != attacker.team && d == 2 {
                let os = obstacles.clone().ignore(target_mpos);
                if let Some(p) = map.find_straight_path(attacker_mpos, target_mpos, &os) {
                    return Some((e, a.clone(), p.first().unwrap().to_world_pos()));
                }
            }
        }
    }

    None
}
