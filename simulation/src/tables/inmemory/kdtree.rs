use super::*;
use crate::storage::TableId;
use rayon::prelude::*;

pub trait KdKey: TableId {
    const AXIS: u32;
    fn get_axis(&self, axis: u32) -> i32;
}

pub struct KdTreeTable<Id, Row>
where
    Id: KdKey,
    Row: TableRow,
{
    indices: Vec<(Id, usize)>,
    values: Vec<Row>,
    // up to which index is the table sorted
    // after that linear search may be utilised
    sort_limit: usize,
}

impl<Id, Row> KdTreeTable<Id, Row>
where
    Id: KdKey,
    Row: TableRow,
{
    pub fn new() -> Self {
        Self {
            indices: Vec::with_capacity(512),
            values: Vec::with_capacity(512),
            sort_limit: 0,
        }
    }

    pub fn find_by_pos<'a>(&'a self, id: &Id) -> Option<&'a Row> {
        Self::find_by_pos_depth(&self.indices[..self.sort_limit], id, 0)
            .or_else(|| {
                // if not found use linear search on the unsorted range
                self.indices[self.sort_limit..]
                    .iter()
                    .find(|(k, _)| k == id)
                    .map(|(_, i)| *i)
            })
            .map(|index| &self.values[index])
    }

    fn find_by_pos_depth(indices: &[(Id, usize)], id: &Id, depth: u32) -> Option<usize> {
        if indices.len() < 2 {
            return None;
        }
        let median = indices.len() / 2;
        let node = &indices[median];
        if node.0 == *id {
            return Some(node.1);
        }
        let bx = node.0.get_axis(depth);
        let ax = id.get_axis(depth);
        if ax < bx {
            Self::find_by_pos_depth(&indices[..median], id, (depth + 1) % Id::AXIS)
        } else {
            Self::find_by_pos_depth(&indices[median + 1..], id, (depth + 1) % Id::AXIS)
        }
    }

    pub fn sort_tree(&mut self) {
        Self::sort_tree_depth(&mut self.indices, 0);
        self.sort_limit = self.indices.len();
    }

    fn sort_tree_depth(indices: &mut [(Id, usize)], depth: u32) {
        if indices.len() < 2 {
            return;
        }
        let median = indices.len() / 2;
        indices.sort_unstable_by_key(|k| k.0.get_axis(depth));
        let (mut hi, lo) = indices.split_at_mut(median);
        let depth = (depth + 1) % Id::AXIS;
        rayon::join(
            || Self::sort_tree_depth(&mut hi, depth),
            || Self::sort_tree_depth(&mut lo[1..], depth),
        );
    }
}

impl<Id, Row> Table for KdTreeTable<Id, Row>
where
    Id: KdKey,
    Row: TableRow,
{
    type Id = Id;
    type Row = Row;

    fn get_by_id<'a>(&'a self, id: &Id) -> Option<&'a Row> {
        self.find_by_pos(id)
    }

    fn get_by_ids<'a>(&'a self, ids: &[Id]) -> Vec<(Id, &'a Row)> {
        ids.iter()
            .filter_map(|id| self.find_by_pos(id).map(|val| (*id, val)))
            .collect()
    }

    fn insert(&mut self, id: Id, row: Row) {
        self.indices.push((id, self.values.len()));
        self.values.push(row);
    }

    fn delete(&mut self, _id: &Id) -> Option<Row> {
        unimplemented!()
    }
}
