use super::System;
use crate::model::{
    components,
    geometry::{Circle, Point},
    EntityId,
};
use crate::storage::views::{UnsafeView, View};
use crate::tables::JoinIterator;
use rand::Rng;

pub struct MineralSystem;

impl<'a> System<'a> for MineralSystem {
    type Mut = (
        UnsafeView<EntityId, components::PositionComponent>,
        UnsafeView<EntityId, components::EnergyComponent>,
    );
    type Const = (
        View<'a, Point, components::EntityComponent>,
        View<'a, EntityId, components::ResourceComponent>,
    );

    fn update(
        &mut self,
        (mut entity_positions, mut energy): Self::Mut,
        (position_entities, resources): Self::Const,
    ) {
        debug!("update minerals system called");

        let mut rng = rand::thread_rng();

        let minerals_it = resources.iter().filter(|(_, r)| match r.0 {
            components::Resource::Energy => true,
        });
        let entity_positions_it = unsafe { entity_positions.as_mut().iter_mut() };
        let energy_iter = unsafe { energy.as_mut().iter_mut() };
        JoinIterator::new(
            JoinIterator::new(minerals_it, entity_positions_it),
            energy_iter,
        )
        .for_each(|(id, ((_resource, position), energy))| {
            if energy.energy > 0 {
                return;
            }
            let pos = random_uncontested_pos_in_range(&*position_entities, &mut rng, -14, 15);
            debug!(
                "Mineral [{:?}] has been depleted, respawning at {:?}",
                id, pos
            );

            energy.energy = energy.energy_max;
            position.0 = pos;
        });

        debug!("update minerals system done");
    }
}

fn random_uncontested_pos_in_range<T: crate::tables::PositionTable>(
    positions_table: &T,
    rng: &mut rand::rngs::ThreadRng,
    from: i32,
    to: i32,
) -> Point {
    let mut pos = Point::default();
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
