use super::System;
use rand::Rng;
use crate::model::{self, EntityId, Point, Circle};
use crate::storage::views::{UnsafeView, View};
use crate::tables::JoinIterator;

pub struct MineralSystem;

impl<'a> System<'a> for MineralSystem {
    type Mut = (
        UnsafeView<EntityId, model::PositionComponent>,
        UnsafeView<EntityId, model::EnergyComponent>,
    );
    type Const = (
        View<'a, Point, model::EntityComponent>,
        View<'a, EntityId, model::ResourceComponent>,
    );

    fn update(
        &mut self,
        (mut entity_positions, mut energy): Self::Mut,
        (position_entities, resources): Self::Const,
    ) {
        debug!("update minerals system called");

        let mut rng = rand::thread_rng();

        let minerals = resources.iter().filter(|(_, r)| match r.0 {
            model::Resource::Mineral => true,
        });
        let changeset = JoinIterator::new(
            JoinIterator::new(minerals, entity_positions.iter()),
            energy.iter(),
        )
        .filter_map(|(id, ((_resource, position), energy))| {
            if energy.energy > 0 {
                return None;
            }

            let mut energy = energy.clone();
            let mut position = position.clone();

            energy.energy = energy.energy_max;

            position.0 = random_uncontested_pos_in_range(&*position_entities, &mut rng, -14, 15);

            Some((id, position, energy))
        })
        .collect::<Vec<_>>();

        for (id, pos, en) in changeset.into_iter() {
            debug!(
                "Mineral [{:?}] has been depleted, respawning at {:?}",
                id, pos
            );
            unsafe {
                entity_positions.as_mut().insert_or_update(id, pos);
                energy.as_mut().insert_or_update(id, en);
            }
        }

        debug!("update minerals system done");
    }
}

fn random_uncontested_pos_in_range<T: crate::tables::PositionTable>(
    positions_table: &T,
    rng: &mut rand::rngs::ThreadRng,
    from: i32,
    to: i32,
) -> Point {
    let mut pos = model::Point::default();
    loop {
        pos.x = rng.gen_range(from, to);
        pos.y = rng.gen_range(from, to);

        let circle = Circle {
            center: pos,
            radius: 1,
        };
        if positions_table.count_entities_in_range(&circle) == 0 {
            break;
        }
    }
    pos
}

