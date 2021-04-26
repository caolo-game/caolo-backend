use crate::indices::*;
use crate::profile;
use crate::storage::views::{DeferredDeleteEntityView, View};
use crate::{
    components::HpComponent,
    intents::{DeleteEntityIntent, Intents},
    prelude::UnwrapView,
};
use tracing::{debug, trace};

pub fn update(
    mut delete: DeferredDeleteEntityView,
    (hps, delete_intents): (
        View<EntityId, HpComponent>,
        UnwrapView<EmptyKey, Intents<DeleteEntityIntent>>,
    ),
) {
    profile!("DeathSystem update");
    debug!("update death system called");

    hps.iter().for_each(|(id, hp)| {
        if hp.hp == 0 {
            trace!("Entity {:?} has died, deleting", id);
            unsafe {
                delete.delete_entity(id);
            }
        }
    });

    delete_intents
        .0
        .iter()
        .for_each(|DeleteEntityIntent { id }| unsafe {
            delete.delete_entity(*id);
        });

    debug!("update death system done");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{intents, query, world::World};
    use crate::{storage::views::FromWorld, storage::views::FromWorldMut};

    #[test]
    fn can_kill_or_delete_entity_multiple_times() {
        let mut store = World::new();

        let entity_1 = store.insert_entity();
        query!(
            mutate
            store
            {
                EntityId, HpComponent, .insert_or_update(entity_1, HpComponent {
                    hp: 0,
                    hp_max: 123
                });
            }
        );
        let entities: Vec<_> = store
            .view::<EntityId, HpComponent>()
            .iter()
            .map(|(id, _)| id)
            .collect();

        assert_eq!(entities, vec![entity_1,]);

        intents::move_into_storage(
            &mut store,
            vec![
                intents::BotIntents {
                    delete_entity_intent: Some(intents::DeleteEntityIntent { id: entity_1 }),
                    ..Default::default()
                };
                3
            ],
        );

        update(FromWorldMut::from_world_mut(&mut *store), FromWorld::from_world(&mut *store));
        store.post_process();

        let entities: Vec<_> = store
            .view::<EntityId, HpComponent>()
            .iter()
            .map(|(id, _)| id)
            .collect();

        assert_eq!(entities, vec![]);
    }

    #[test]
    fn test_dead_entity_is_deleted() {
        let mut store = World::new();

        let entity_1 = store.insert_entity();
        let entity_2 = store.insert_entity();
        query!(
            mutate
            store
            {
                EntityId, HpComponent, .insert_or_update(entity_1, HpComponent {
                    hp: 0,
                    hp_max: 123
                });
                EntityId, HpComponent, .insert_or_update(entity_2, HpComponent {
                    hp: 50,
                    hp_max: 123
                });
            }
        );

        let entities: Vec<_> = store
            .view::<EntityId, HpComponent>()
            .iter()
            .map(|(id, _)| id)
            .collect();

        assert_eq!(entities, vec![entity_1, entity_2]);

        update(FromWorldMut::from_world_mut(&mut *store), FromWorld::from_world(&mut *store));
        store.post_process();

        let entities: Vec<_> = store
            .view::<EntityId, HpComponent>()
            .iter()
            .map(|(id, _)| id)
            .collect();

        assert_eq!(entities, vec![entity_2]);
    }
}
