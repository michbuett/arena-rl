use bevy::prelude::*;

use crate::{
    assets::Visual,
    core::{ActionFx, Health},
};

use super::{
    actor::{ActionConsequence, ActionFinishedEvent},
    fx::{FxEffect, FxSequence},
    map::MapPos,
    ui::Z_LAYER_ACTOR,
};

pub fn handle_action_finished_event(
    trigger: On<ActionFinishedEvent>,
    actor_pos_q: Query<(&MapPos, &Health)>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let ActionFinishedEvent {
        actor,
        targets,
        fx,
        consequences,
        ..
    } = trigger.event();

    let (attacker_mpos, _) = actor_pos_q.get(*actor)?;
    // let [(attacker_mpos, _), (target_mpos, _)] = actor_data_q.get_many([*actor, *target])?;
    let step_durration = 100;
    let attacker_pos = attacker_mpos.into_vec3().with_z(Z_LAYER_ACTOR);

    let mut fx_seq = match fx {
        ActionFx::SingleTargetMeleeAttack(eff_name) => {
            let (target_mpos, _) = actor_pos_q.get(*targets.0.first().unwrap())?;
            let target_pos = target_mpos.into_vec3().with_z(Z_LAYER_ACTOR);
            let path = vec![attacker_pos, target_pos, attacker_pos];

            FxSequence::new()
                .then(FxEffect::MoveTo {
                    entity: *actor,
                    path,
                    movement_modification: crate::animations::MovementModification::None,
                    step_durration,
                })
                .wait(step_durration)
                .then(FxEffect::hit(
                    Visual::Single(eff_name.to_string()),
                    *target_mpos,
                ))
                .wait(200)
        }

        ActionFx::SelfTxt(txt) => FxSequence::new()
            .then(FxEffect::say(txt, *attacker_mpos))
            .wait(200),
    };

    for (entity, consequence) in consequences.iter() {
        let (mpos, health) = actor_pos_q.get(*entity)?;

        fx_seq = match consequence {
            ActionConsequence::Damage(d) => {
                if !health.is_alive() {
                    fx_seq = fx_seq.then(FxEffect::Remove(*entity));
                }

                for _ in 1..=d.actual_damage {
                    fx_seq = fx_seq
                        .then(FxEffect::BloodSplatter(mpos.into_vec3()))
                        .wait(50);
                }

                fx_seq
            }
            _ => fx_seq,
        };
    }

    fx_seq.wait(100).run(&mut commands);

    Ok(())
}
