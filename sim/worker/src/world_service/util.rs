use std::collections::HashMap;

use caolo_sim::prelude::Axial;

use crate::protos::{cao_common, cao_world};

pub fn push_room_pl<Selector, T>(
    out: &mut HashMap<Axial, cao_world::RoomEntities>,
    room_id: Axial,
    f: Selector,
    accumulator: T,
    time: i64,
) where
    Selector: Fn(&mut cao_world::RoomEntities) -> &mut T,
{
    let pl = out.entry(room_id).or_insert_with(Default::default);
    let room_id = cao_common::Axial {
        q: room_id.q,
        r: room_id.r,
    };
    pl.room_id = Some(room_id);
    pl.world_time = time;
    *f(pl) = accumulator;
}
