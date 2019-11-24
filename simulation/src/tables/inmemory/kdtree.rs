//! KD-tree based In Memory table optimal for spacial lookup of higher dimensional keys
//!
use super::{TableBackend, TableIterator};
use crate::storage::TableId;
use rayon::prelude::*;
use std::cmp::Ordering;

pub trait KdTreeKey: TableId + Sync {
    /// Number of dimensions to split the data by
    const DIMS: u32;
    /// Compare two keys among the given axis
    fn compare(&self, other: &Self, axis: u32) -> Ordering;
    fn distance(&self, other: &Self) -> f32;
}

/// Return the found node as well as it's distance from the query
pub struct QueryResult<Id, Row> {
    pub dist: f32,
    pub key: Id,
    pub value: Row,
}

#[derive(Default)]
pub struct KdTreeTable<Id, Row>
where
    Id: KdTreeKey,
    Row: Clone,
{
    key: Id,
    value: Row,
    size: u32,
    /// How deep is this subtree
    depth: u32,

    left: Option<Box<KdTreeTable<Id, Row>>>,
    right: Option<Box<KdTreeTable<Id, Row>>>,
}

impl<Id, Row> KdTreeTable<Id, Row>
where
    Id: KdTreeKey,
    Row: Clone + Send,
{
    /// Insertions and deletions may leave the tree imbalanced
    /// To regain its performance it might be necessary to rebalance
    /// currently the only way to do that is to rebuild the tree
    pub fn rebuild(&mut self) {
        let mut buffer = Vec::with_capacity(self.size as usize);
        self.collect_nodes(&mut buffer);
        *self = Self::from_nodes(&mut buffer).unwrap();
    }

    fn collect_nodes(&mut self, buffer: &mut Vec<(Id, Row)>) {
        buffer.push((self.key, self.value.clone()));
        if let Some(left) = self.left.as_mut() {
            left.collect_nodes(buffer);
        }
        if let Some(right) = self.right.as_mut() {
            right.collect_nodes(buffer);
        }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub fn from_nodes(nodes: &mut [(Id, Row)]) -> Option<Self> {
        Self::from_nodes_depth(nodes, 0)
    }

    fn from_nodes_depth(nodes: &mut [(Id, Row)], depth: u32) -> Option<Self> {
        if nodes.len() == 0 {
            return None;
        }
        let axis = depth % Id::DIMS;
        nodes.sort_unstable_by(|x, y| x.0.compare(&y.0, axis));

        let median = nodes.len() / 2;

        let (lo, hi) = nodes.split_at_mut(median);
        let (left, right) = rayon::join(
            || Self::from_nodes_depth(lo, depth + 1).map(Box::new),
            || Self::from_nodes_depth(&mut hi[1..], depth + 1).map(Box::new),
        );

        let children_depth = left
            .as_ref()
            .map(|x| x.depth)
            .or_else(|| right.as_ref().map(|x| x.depth))
            .unwrap_or(0)
            + 1;

        let (key, value) = nodes[median].clone();
        let node = Self {
            key,
            value,
            left,
            right,
            size: nodes.len() as u32,
            depth: children_depth,
        };
        Some(node)
    }

    pub fn find_k_nearest(&self, point: &Id, k: usize) -> Vec<QueryResult<Id, Row>> {
        let mut result = Vec::with_capacity(512); // TODO
        self.find_k_nearest_by_depth(point, k, 0, &mut result);
        result
    }

    fn find_k_nearest_by_depth(
        &self,
        point: &Id,
        k: usize,
        depth: u32,
        out: &mut Vec<QueryResult<Id, Row>>,
    ) {
        let dist = self.key.distance(point);
        if self.left.is_none() && self.right.is_none() {
            out.push(QueryResult {
                dist,
                key: self.key,
                value: self.value.clone(),
            });
            return;
        }
        let axis = depth % Id::DIMS;
        let (nearer, further) = if self.right.is_none()
            || (self.left.is_some() && point.compare(&self.key, axis) != Ordering::Less)
        {
            (self.left.as_ref().unwrap(), &self.right)
        } else {
            (self.right.as_ref().unwrap(), &self.left)
        };
        nearer.find_k_nearest_by_depth(point, k, depth + 1, out);
        if let Some(ref further) = further {
            if out.len() < k || further.key.distance(point) < out.last().unwrap().dist {
                further.find_k_nearest_by_depth(point, k, depth + 1, out);
            }
        }

        out.push(QueryResult {
            dist,
            key: self.key,
            value: self.value.clone(),
        });

        out.sort_unstable_by(|x, y| x.dist.partial_cmp(&y.dist).unwrap_or(Ordering::Equal));
        out.truncate(k);
    }

    pub fn find_by_depth(&self, point: &Id, depth: u32) -> Option<&Row> {
        if *point == self.key {
            return Some(&self.value);
        }
        if self.left.is_none() && self.right.is_none() {
            return None;
        }
        let axis = depth % Id::DIMS;
        let nearer = if self.right.is_none()
            || (self.left.is_some() && point.compare(&self.key, axis) != Ordering::Less)
        {
            self.left.as_ref().unwrap()
        } else {
            self.right.as_ref().unwrap()
        };
        nearer.find_by_depth(point, depth + 1)
    }
}

impl<Id, Row> TableBackend for KdTreeTable<Id, Row>
where
    Id: KdTreeKey,
    Row: Clone + Send + Sync,
{
    type Id = Id;
    type Row = Row;

    fn get_by_id(&self, id: &Id) -> Option<Row> {
        self.find_by_depth(id, 0).cloned()
    }

    fn get_by_ids(&self, ids: &[Id]) -> Vec<(Id, Row)> {
        ids.par_iter()
            .filter_map(|id| self.find_by_depth(id, 0).map(|v| (*id, v.clone())))
            .collect()
    }

    fn insert(&mut self, id: Id, row: Row) {
        unimplemented!()
    }

    fn delete(&mut self, id: &Id) -> Option<Row> {
        unimplemented!()
    }

    fn iter<'a>(&'a self) -> Box<dyn TableIterator<Id, Row> + 'a> {
        unimplemented!()
    }
}
