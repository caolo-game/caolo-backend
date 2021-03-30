use super::morton::{MortonKey, MortonTable};
use super::hex_grid::HexGrid;
use super::*;
use crate::geometry::Axial;
use crate::indices::{Room, WorldPosition};
use rayon::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::{convert::TryFrom, marker::PhantomData};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ExtendFailure {
    #[error("Failed to extend the room level {0:?}")]
    RoomExtendFailure(super::morton::ExtendFailure),
    #[error("Failed to insert poision {0:?}")]
    InvalidPosition(WorldPosition),
    #[error("Room {0:?} does not exist")]
    RoomNotExists(Axial),
    #[error("Extending room {room:?} failed with error {error}")]
    InnerExtendFailure { room: Axial, error: String },
}

pub type MortonMortonTable<T> = RoomMortonTable<MortonTable<T>, T>;
pub type MortonGridTable<T> = RoomMortonTable<HexGrid<T>, T>;

pub trait SpacialStorage<Row: TableRow>:
    Table<Id = Axial, Row = Row> + Clone + std::fmt::Debug + 'static + Default
{
    type ExtendFailure: std::fmt::Display;

    fn clear(&mut self);
    fn contains_key(&self, pos: Axial) -> bool;
    fn at(&self, pos: Axial) -> Option<&Row>;
    fn at_mut(&mut self, pos: Axial) -> Option<&mut Row>;
    fn insert(&mut self, id: Axial, row: Row) -> Result<(), Self::ExtendFailure>;
    fn extend<It>(&mut self, it: It) -> Result<(), Self::ExtendFailure>
    where
        It: Iterator<Item = (Axial, Row)>;
}

/// Holds an inner morton_table that holds other spacial data structures for hierarchycal spacial
/// storage
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RoomMortonTable<InnerTable, Row>
where
    InnerTable: SpacialStorage<Row>,
    Row: TableRow,
{
    pub table: MortonTable<InnerTable>,
    _m: PhantomData<Row>,
}

impl<InnerTable, Row> RoomMortonTable<InnerTable, Row>
where
    InnerTable: SpacialStorage<Row>,
    Row: TableRow,
{
    pub fn new() -> Self {
        Self {
            table: MortonTable::new(),
            _m: Default::default(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            table: MortonTable::with_capacity(cap),
            _m: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter_rooms(&self) -> impl Iterator<Item = (Room, &InnerTable)> {
        self.table.iter().map(|(room, table)| (Room(room), table))
    }

    pub fn iter_rooms_mut(&mut self) -> impl Iterator<Item = (Room, &mut InnerTable)> {
        self.table
            .iter_mut()
            .map(|(room, table)| (Room(room), table))
    }

    /// Shallow clear,
    /// leaves the 'overworld' level intact and clears the rooms.
    pub fn clear(&mut self) {
        self.table.iter_mut().for_each(|(_, table)| {
            table.clear();
        });
    }

    /// Clear the whole table
    pub fn deep_clear(&mut self) {
        self.table.clear();
    }

    pub fn contains_room(&self, id: Room) -> bool {
        self.table.contains_key(id.0)
    }

    pub fn contains_key(&self, id: &WorldPosition) -> bool {
        self.table
            .at(id.room)
            .map(|room| room.contains_key(id.pos))
            .unwrap_or(false)
    }

    /// Inserts the item at the given position. Creates a table for the room if it's not found
    pub fn insert(&mut self, id: WorldPosition, val: Row) -> Result<(), ExtendFailure> {
        let mut room = self.table.at_mut(id.room);
        if room.is_none() {
            self.table
                .insert(id.room, InnerTable::default())
                .map_err(ExtendFailure::RoomExtendFailure)?;
            room = self.table.at_mut(id.room);
        }
        room.unwrap()
            .insert(id.pos, val)
            .map_err(|error| ExtendFailure::InnerExtendFailure {
                error: error.to_string(),
                room: id.room,
            })
    }

    pub fn at_mut(&mut self, id: WorldPosition) -> Option<&mut Row> {
        self.table
            .at_mut(id.room)
            .and_then(|room| room.at_mut(id.pos))
    }

    pub fn get_by_id(&self, id: WorldPosition) -> Option<&Row> {
        self.table.at(id.room).and_then(|room| room.at(id.pos))
    }

    pub fn extend_rooms<It>(&mut self, iter: It) -> Result<&mut Self, ExtendFailure>
    where
        It: Iterator<Item = Room>,
    {
        self.table
            .extend(iter.map(|Room(p)| (p, Default::default())))
            .map_err(ExtendFailure::RoomExtendFailure)?;
        Ok(self)
    }

    /// Extend the map by the items provided.
    pub fn extend_from_slice(
        &mut self,
        values: &mut [(WorldPosition, Row)],
    ) -> Result<(), ExtendFailure>
    where
        InnerTable: Send,
        Row: Sync,
    {
        {
            // produce a key list from the rooms of the values
            let mut keys = values
                .iter()
                .map(|(wp, _)| {
                    MortonKey::new(
                        u16::try_from(wp.room.q).expect("expected room q to fit into u16"),
                        u16::try_from(wp.room.r).expect("expected room r to fit into u16"),
                    )
                })
                .collect::<Vec<_>>();

            // use the morton sorting to sort these values by their rooms
            morton::sorting::sort(&mut keys, values);
        }

        // values no longer has to be mutable
        let values = values as &_;

        // collect the groups into a hashmap
        let groups: std::collections::HashMap<Axial, &[(WorldPosition, Row)]> =
            GroupByRooms::new(&values).collect();
        let groups = &groups;

        // clippy will flag the collect, however we must collect otherwise the &self reference
        // isn't freed
        #[allow(clippy::needless_collect)]
        {
            let new_rooms = groups
                .keys()
                .filter(|room_id| !self.contains_room(Room(**room_id)))
                .map(|rid| Room(*rid))
                .collect::<Vec<_>>();

            self.extend_rooms(new_rooms.into_iter())?;
        }

        self.table
            .par_iter_mut()
            .try_for_each(move |(room_id, room)| {
                let items = match groups.get(&room_id) {
                    Some(i) => i,
                    // no inserts in this room
                    None => return Ok(()),
                };
                // extend each group by their corresponding values
                room.extend(
                    items
                        .iter()
                        .map(|(WorldPosition { pos, .. }, row)| (*pos, row.clone())),
                )
                .map_err(|error| ExtendFailure::InnerExtendFailure {
                    room: room_id,
                    error: error.to_string(),
                })
            })?;

        Ok(())
    }
}

struct GroupByRooms<'a, Row> {
    items: &'a [(WorldPosition, Row)],
    group_begin: usize,
}

impl<'a, Row> Iterator for GroupByRooms<'a, Row> {
    type Item = (Axial, &'a [(WorldPosition, Row)]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.items.len() <= self.group_begin {
            return None;
        }
        let mut end = self.group_begin;
        let begin = &self.items[self.group_begin].0.room;
        for (i, (WorldPosition { room, .. }, _)) in
            self.items[self.group_begin..].iter().enumerate()
        {
            if room != begin {
                break;
            }
            end = i;
        }
        end += self.group_begin;
        let group_begin = self.group_begin;
        self.group_begin = end + 1;
        if group_begin <= end {
            Some((*begin, &self.items[group_begin..=end]))
        } else {
            None
        }
    }
}

impl<'a, Row> GroupByRooms<'a, Row> {
    pub fn new(items: &'a [(WorldPosition, Row)]) -> Self {
        #[cfg(debug_assertions)]
        {
            // assert that items is sorted.
            // at the time of writing `is_sorted` is still unstable
            if items.len() > 2 {
                let mut it = items.iter();
                let mut current = it.next().unwrap();
                for item in it {
                    assert!(
                        MortonKey::new(
                            u16::try_from(current.0.room.q).unwrap(),
                            u16::try_from(current.0.room.r).unwrap(),
                        ) <= MortonKey::new(
                            u16::try_from(item.0.room.q).unwrap(),
                            u16::try_from(item.0.room.r).unwrap(),
                        )
                    );
                    current = item;
                }
            }
        }
        Self {
            items,
            group_begin: 0,
        }
    }
}

impl<InnerTable, Row> Table for RoomMortonTable<InnerTable, Row>
where
    InnerTable: SpacialStorage<Row>,
    Row: TableRow,
{
    type Id = WorldPosition;
    type Row = Row;

    /// delete all values at id and return the first one, if any
    fn delete(&mut self, id: Self::Id) -> Option<Row> {
        let WorldPosition { room, pos } = id;
        let room = self.table.at_mut(room)?;
        room.delete(pos)
    }

    fn get_by_id(&self, id: Self::Id) -> Option<&Row> {
        RoomMortonTable::get_by_id(self, id)
    }
}

impl<Row> MortonMortonTable<Row>
where
    Row: TableRow,
{
    pub fn iter(&self) -> impl Iterator<Item = (WorldPosition, &Row)> {
        self.iter_rooms().flat_map(|(room_id, room)| {
            room.iter().map(move |(pos, item)| {
                (
                    WorldPosition {
                        room: room_id.0,
                        pos,
                    },
                    item,
                )
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extends_multiple_rooms_correctly() {
        let mut pts = [
            (
                WorldPosition {
                    room: Axial::new(42, 69),
                    pos: Axial::new(1, 2),
                },
                1,
            ),
            (
                WorldPosition {
                    room: Axial::new(42, 69),
                    pos: Axial::new(2, 4),
                },
                2,
            ),
            (
                WorldPosition {
                    room: Axial::new(42, 69),
                    pos: Axial::new(1, 2),
                },
                3,
            ),
            (
                WorldPosition {
                    room: Axial::new(42, 69),
                    pos: Axial::new(2, 4),
                },
                4,
            ),
            (
                WorldPosition {
                    room: Axial::new(69, 69),
                    pos: Axial::new(8, 9),
                },
                5,
            ),
            (
                WorldPosition {
                    room: Axial::new(69, 69),
                    pos: Axial::new(8, 8),
                },
                6,
            ),
        ];

        let mut table = MortonMortonTable::new();
        table
            .extend_rooms(
                [Axial::new(69, 69), Axial::new(42, 69)]
                    .iter()
                    .cloned()
                    .map(|p| Room(p)),
            )
            .unwrap();
        table.extend_from_slice(&mut pts).unwrap();

        assert_eq!(table.table.get_by_id(Axial::new(69, 69)).unwrap().len(), 2);
        assert_eq!(table.table.get_by_id(Axial::new(42, 69)).unwrap().len(), 4);
    }
}
