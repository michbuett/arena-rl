use bevy::prelude::*;

use crate::{assets::Visual, core::Health};

use super::{
    actor::CombatFinishedEvent,
    combat_resolution::CombatConsequence,
    fx::{FxEffect, FxSequence},
    map::MapPos,
    ui::Z_LAYER_ACTOR,
};

pub fn handle_combat_finished_event(
    trigger: On<CombatFinishedEvent>,
    actor_data_q: Query<(&MapPos, &Health)>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let CombatFinishedEvent {
        attacker,
        target,
        result: combat_result,
    } = trigger.event();

    let [(attacker_mpos, _), (target_mpos, _)] = actor_data_q.get_many([*attacker, *target])?;
    let step_durration = 100;
    let attacker_pos = attacker_mpos.into_vec3().with_z(Z_LAYER_ACTOR);
    let target_pos = target_mpos.into_vec3().with_z(Z_LAYER_ACTOR);
    let path = vec![attacker_pos, target_pos, attacker_pos];

    let mut fx_seq = FxSequence::new()
        .then(FxEffect::MoveTo {
            entity: *attacker,
            path,
            movement_modification: crate::animations::MovementModification::None,
            step_durration,
        })
        .wait(step_durration)
        .then(FxEffect::hit(
            Visual::Single("fx-hit-1".to_string()),
            *target_mpos,
        ))
        .wait(200);

    for (entity, consequence) in combat_result.consequences.iter() {
        let (mpos, health) = actor_data_q.get(*entity)?;

        fx_seq = match consequence {
            CombatConsequence::ClumsyAttack => fx_seq.then(FxEffect::say("Fuck!", *mpos)),
            CombatConsequence::Wound { damage } => {
                if !health.is_alive() {
                    fx_seq = fx_seq.then(FxEffect::Remove(*entity));
                }

                for _ in damage.iter() {
                    fx_seq = fx_seq
                        .then(FxEffect::BloodSplatter(mpos.into_vec3()))
                        .wait(50);
                }

                fx_seq
            }
            CombatConsequence::ArmorBreak => fx_seq.then(FxEffect::say("Armor -1", *mpos)),
        };
    }

    fx_seq.wait(100).run(&mut commands);

    Ok(())
}
