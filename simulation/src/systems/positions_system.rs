use super::System;
use crate::model::{self, EntityId, Point};
use crate::storage::views::{UnsafeView, View};

pub struct PositionSystem;

impl<'a> System<'a> for PositionSystem {
    type Mut = UnsafeView<Point, model::EntityComponent>;
    type Const = View<'a, EntityId, model::PositionComponent>;

    fn update(&mut self, mut position_entities: Self::Mut, positions: Self::Const) {
        debug!("update positions system called");
        let positions = positions
            .iter()
            .map(|(id, pos)| (pos.0, model::EntityComponent(id)))
            .collect::<Vec<_>>();

        unsafe {
            position_entities.as_mut().clear();

            for (point, entity) in positions.into_iter() {
                position_entities.as_mut().insert(point, entity);
            }
        }
        debug!("update positions system done");
    }
}
