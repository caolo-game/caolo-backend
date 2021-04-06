use crate::components::{HpComponent, MeleeAttackComponent};
use crate::indices::*;
use crate::intents::*;
use crate::profile;
use crate::storage::views::{UnsafeView, UnwrapViewMut, View};
use rayon::prelude::*;
use tracing::{debug, error};

type Mut = (
    UnsafeView<EntityId, HpComponent>,
    UnwrapViewMut<EmptyKey, Intents<MeleeIntent>>,
);
type Const<'a> = (View<'a, EntityId, MeleeAttackComponent>,);

pub fn update((mut hp_table, mut intents): Mut, (attack_table,): Const) {
    profile!("AttackSystem update");

    pre_process(&mut intents.0);

    for intent in intents.iter() {
        let attack = match attack_table.get_by_id(intent.attacker) {
            Some(s) => s,
            None => {
                error!("Attacker has no attack component. {:?}", intent);
                continue;
            }
        };
        let hp = match hp_table.get_by_id_mut(intent.defender) {
            Some(s) => s,
            None => {
                error!("Defender has no hp component. {:?}", intent);
                continue;
            }
        };
        // hp can not fall below 0
        hp.hp -= hp.hp.min(attack.strength);
    }
}

fn pre_process(intents: &mut Vec<MeleeIntent>) {
    let len = intents.len();
    if len < 2 {
        return;
    }
    // dedupe
    intents.par_sort_unstable_by_key(|intent| intent.attacker);
    for current in (0..len).rev() {
        let last = current + 1;
        let a = &intents[last];
        let b = &intents[current];
        if a.attacker == b.attacker {
            debug!("Duplicated attacker, removing {:?}", a);
            intents.swap_remove(last);
        }
    }
}
