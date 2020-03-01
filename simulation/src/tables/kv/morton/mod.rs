//! Linear Quadtree.
//! # Contracts:
//! - Key axis must be in the interval [0, 2^16)
//! This is a severe restriction on the keys that can be used, however dense queries and
//! constructing from iterators is much faster than quadtrees.
//!
#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::mem;

mod morton_key;
#[cfg(test)]
mod tests;

use super::*;
use crate::model::{components::EntityComponent, geometry::Point};
use morton_key::*;
use rayon::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

use crate::profile;

#[derive(Debug, Clone)]
pub enum ExtendFailure<Id: SpatialKey2d> {
    InvalidPosition(Id),
}

const SKIP_LEN: usize = 8;
type SkipList = [u32; SKIP_LEN];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortonTable<Pos, Row>
where
    Pos: SpatialKey2d,
    Row: TableRow,
{
    skiplist: SkipList,
    skipstep: u32,
    // ---- 9 * 4 bytes so far
    // assuming 64 byte long L1 cache lines we can fit 10 keys
    // keys is 24 bytes in memory
    keys: Vec<MortonKey>,
    poss: Vec<Pos>,
    values: Vec<Row>,
}

impl<Pos, Row> Default for MortonTable<Pos, Row>
where
    Pos: SpatialKey2d + Send,
    Row: TableRow + Send,
{
    fn default() -> Self {
        Self {
            skiplist: [0; SKIP_LEN],
            skipstep: 0,
            keys: Default::default(),
            values: Default::default(),
            poss: Default::default(),
        }
    }
}

unsafe impl<Pos, Row> Send for MortonTable<Pos, Row>
where
    Pos: SpatialKey2d + Send,
    Row: TableRow + Send,
{
}

impl<Pos, Row> MortonTable<Pos, Row>
where
    Pos: SpatialKey2d + Sync,
    Row: TableRow + Send + Sync,
{
    pub fn new() -> Self {
        Self {
            skiplist: Default::default(),
            skipstep: 0,
            keys: vec![],
            values: vec![],
            poss: vec![],
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            skiplist: Default::default(),
            skipstep: 0,
            values: Vec::with_capacity(cap),
            keys: Vec::with_capacity(cap),
            poss: Vec::with_capacity(cap),
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (Pos, &'a Row)> + 'a {
        let values = self.values.as_ptr();
        self.poss.iter().enumerate().map(move |(i, id)| {
            let val = unsafe { &*values.offset(i as isize) };
            (*id, val)
        })
    }

    pub fn from_iterator<It>(it: It) -> Result<Self, ExtendFailure<Pos>>
    where
        It: Iterator<Item = (Pos, Row)>,
    {
        let mut res = Self::new();
        res.extend(it)?;
        Ok(res)
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.skiplist = [Default::default(); SKIP_LEN];
        self.values.clear();
        self.poss.clear();
    }

    /// Extend the map by the items provided. Panics on invalid items.
    pub fn extend<It>(&mut self, it: It) -> Result<(), ExtendFailure<Pos>>
    where
        It: Iterator<Item = (Pos, Row)>,
    {
        for (id, value) in it {
            if !self.intersects(&id) {
                return Err(ExtendFailure::InvalidPosition(id));
            }
            let [x, y] = id.as_array();
            let [x, y] = [x as u16, y as u16];
            let key = MortonKey::new(x, y);
            self.keys.push(key);
            self.poss.push(id);
            self.values.push(value);
        }
        sort(
            self.keys.as_mut_slice(),
            self.poss.as_mut_slice(),
            self.values.as_mut_slice(),
        );
        self.rebuild_skip_list();
        Ok(())
    }

    fn rebuild_skip_list(&mut self) {
        #[cfg(debug_assertions)]
        {
            // assert that keys is sorted.
            // at the time of writing is_sorted is still unstable
            if self.keys.len() > 2 {
                let mut it = self.keys.iter();
                let mut current = it.next().unwrap();
                for item in it {
                    assert!(current <= item);
                    current = item;
                }
            }
        }

        let len = self.keys.len();
        let step = len / SKIP_LEN;
        self.skipstep = step as u32;
        if step < 1 {
            if let Some(key) = self.keys.last() {
                self.skiplist[0] = key.0;
            }
            return;
        }
        for (i, k) in (0..len).step_by(step).skip(1).take(SKIP_LEN).enumerate() {
            self.skiplist[i] = self.keys[k].0;
        }
    }

    /// May trigger reordering of items, if applicable prefer `extend` and insert many keys at once.
    pub fn insert(&mut self, id: Pos, row: Row) -> bool {
        if !self.intersects(&id) {
            return false;
        }
        let [x, y] = id.as_array();
        let [x, y] = [x as u16, y as u16];

        let ind = self
            .keys
            .binary_search(&MortonKey::new(x, y))
            .unwrap_or_else(|i| i);
        self.keys.insert(ind, MortonKey::new(x, y));
        self.poss.insert(ind, id);
        self.values.insert(ind, row);
        self.rebuild_skip_list();
        true
    }

    /// Returns the first item with given id, if any
    pub fn get_by_id<'a>(&'a self, id: &Pos) -> Option<&'a Row> {
        profile!("get_by_id");

        if !self.intersects(&id) {
            return None;
        }

        self.find_key(id).map(|ind| &self.values[ind]).ok()
    }

    pub fn contains_key(&self, id: &Pos) -> bool {
        profile!("contains_key");

        if !self.intersects(&id) {
            return false;
        }
        self.find_key(id).is_ok()
    }

    /// Find the position of `id` or the position where it needs to be inserted to keep the
    /// container sorted
    fn find_key(&self, id: &Pos) -> Result<usize, usize> {
        let [x, y] = id.as_array();
        let key = MortonKey::new(x as u16, y as u16);

        let step = self.skipstep as usize;
        if step == 0 {
            return self.keys.binary_search(&key);
        }

        let index = if is_x86_feature_detected!("sse2") {
            unsafe { find_key_partition_sse2(&self.skiplist, &key) }
        } else {
            sse_panic()
        };
        let (begin, end) = {
            if index < 8 {
                let begin = index * step;
                let end = self.keys.len().min(begin + step + 1);
                (begin, end)
            } else {
                debug_assert!(self.keys.len() >= step + 3);
                let end = self.keys.len();
                let begin = end - step - 3;
                (begin, end)
            }
        };
        self.keys[begin..end]
            .binary_search(&key)
            .map(|ind| ind + begin)
            .map_err(|ind| ind + begin)
    }

    /// For each id returns the first item with given id, if any
    pub fn get_by_ids<'a>(&'a self, ids: &[Pos]) -> Vec<(Pos, &'a Row)> {
        profile!("get_by_ids");

        ids.into_par_iter()
            .filter_map(|id| self.get_by_id(id).map(|row| (*id, row)))
            .collect()
    }

    /// Find in AABB
    pub fn find_by_range<'a>(&'a self, center: &Pos, radius: u32, out: &mut Vec<(Pos, &'a Row)>) {
        profile!("find_by_range");

        let r = i32::try_from(radius).expect("radius to fit into 31 bits");
        let r = (r >> 1) + 1;
        let min = *center + Pos::new(-r, -r);
        let max = *center + Pos::new(r, r);

        let [min, max] = self.morton_min_max(&min, &max);
        let it = self.poss[min..max]
            .iter()
            .enumerate()
            .filter_map(|(i, id)| {
                if center.dist(&id) < radius {
                    Some((*id, &self.values[i + min]))
                } else {
                    None
                }
            });
        out.extend(it)
    }

    /// Count in AABB
    pub fn count_in_range<'a>(&'a self, center: &Pos, radius: u32) -> u32 {
        profile!("count_in_range");

        let r = radius as i32 / 2 + 1;
        let min = *center + Pos::new(-r, -r);
        let max = *center + Pos::new(r, r);

        let [min, max] = self.morton_min_max(&min, &max);

        self.poss[min..max]
            .iter()
            .filter(move |id| center.dist(&id) < radius)
            .count()
            .try_into()
            .expect("count to fit into 32 bits")
    }

    /// Turn AABB min-max to from-to indices
    /// Clamps `min` and `max` to intersect `self`
    fn morton_min_max(&self, min: &Pos, max: &Pos) -> [usize; 2] {
        let min: usize = {
            if !self.intersects(&min) {
                0
            } else {
                self.find_key(&min).unwrap_or_else(|i| i)
            }
        };
        let max: usize = {
            let lim = (self.keys.len() as i64 - 1).max(0) as usize;
            if !self.intersects(&max) {
                lim
            } else {
                self.find_key(&max).unwrap_or_else(|i| i)
            }
        };
        [min, max]
    }

    /// Return wether point is within the bounds of this node
    pub fn intersects(&self, point: &Pos) -> bool {
        let [x, y] = point.as_array();
        // at most 15 bits long non-negative integers
        // having the 16th bit set might create problems in find_key
        const MASK: i32 = 0b0111111111111111;
        (x & MASK) == x && (y & MASK) == y
    }
}

impl<Pos, Row> Table for MortonTable<Pos, Row>
where
    Pos: SpatialKey2d + Send + Sync,
    Row: TableRow + Send + Sync,
{
    type Id = Pos;
    type Row = Row;

    fn delete(&mut self, id: &Pos) -> Option<Row> {
        profile!("delete");
        if !self.contains_key(id) {
            return None;
        }

        self.find_key(&id)
            .map(|ind| {
                self.keys.remove(ind);
                self.poss.remove(ind);
                self.values.remove(ind)
            })
            .ok()
    }
}

impl PositionTable for MortonTable<Point, EntityComponent> {
    fn get_entities_in_range(&self, vision: &Circle) -> Vec<(EntityId, PositionComponent)> {
        profile!("get_entities_in_range");

        let mut res = Vec::new();
        self.find_by_range(&vision.center, vision.radius * 3 / 2, &mut res);
        res.into_iter()
            .filter(|(pos, _)| pos.hex_distance(vision.center) <= u64::from(vision.radius))
            .map(|(pos, id)| (id.0, PositionComponent(pos)))
            .collect()
    }

    fn count_entities_in_range(&self, vision: &Circle) -> usize {
        profile!("count_entities_in_range");

        self.count_in_range(&vision.center, vision.radius * 3 / 2) as usize
    }
}

fn sort<Pos: Send, Row: Send>(keys: &mut [MortonKey], poss: &mut [Pos], values: &mut [Row]) {
    debug_assert!(keys.len() == poss.len(), "{} {}", keys.len(), poss.len());
    debug_assert!(
        keys.len() == values.len(),
        "{} {}",
        keys.len(),
        values.len()
    );
    if keys.len() < 2 {
        return;
    }
    let pivot = sort_partition(keys, poss, values);
    let (klo, khi) = keys.split_at_mut(pivot);
    let (plo, phi) = poss.split_at_mut(pivot);
    let (vlo, vhi) = values.split_at_mut(pivot);
    rayon::join(
        || sort(klo, plo, vlo),
        || sort(&mut khi[1..], &mut phi[1..], &mut vhi[1..]),
    );
}

fn sort_partition<Pos: Send, Row: Send>(
    keys: &mut [MortonKey],
    poss: &mut [Pos],
    values: &mut [Row],
) -> usize {
    debug_assert!(keys.len() > 0);

    let lim = keys.len() - 1;
    let mut i = 0;
    let pivot = keys[lim];
    for j in 0..lim {
        if keys[j] < pivot {
            keys.swap(i, j);
            poss.swap(i, j);
            values.swap(i, j);
            i += 1;
        }
    }
    keys.swap(i, lim);
    poss.swap(i, lim);
    values.swap(i, lim);
    i
}

/// Find the index of the partition where `key` _might_ reside.
/// This is the index of the second to first item in the `skiplist` that is greater than the `key`
#[inline(always)]
unsafe fn find_key_partition_sse2(skiplist: &[u32; SKIP_LEN], key: &MortonKey) -> usize {
    let key = key.0 as i32;
    let keys4 = _mm_set_epi32(key, key, key, key);

    let [s0, s1, s2, s3, s4, s5, s6, s7]: [i32; SKIP_LEN] = mem::transmute(*skiplist);
    let skiplist_a: __m128i = _mm_set_epi32(s0, s1, s2, s3);
    let skiplist_b: __m128i = _mm_set_epi32(s4, s5, s6, s7);

    // set every 32 bits to 0xFFFF if key < skip else sets it to 0x0000
    let results_a = _mm_cmpgt_epi32(keys4, skiplist_a);
    let results_b = _mm_cmpgt_epi32(keys4, skiplist_b);

    // create a mask from the most significant bit of each 8bit element
    let mask_a = _mm_movemask_epi8(results_a);
    let mask_b = _mm_movemask_epi8(results_b);

    // count the number of bits set to 1
    let index = _popcnt32(mask_a) + _popcnt32(mask_b);
    // because the mask was created from 8 bit wide items every key in skip list is counted
    // 4 times.
    // We know that index is unsigned to we can optimize by using bitshifting instead
    //   of division.
    //   This resulted in a 1ns speedup on my Intel Code i7-8700 CPU.
    let index = index >> 2;
    index as usize
}

#[inline(never)]
fn sse_panic() -> usize {
    println!(
        r#"
AVX: {}
SSE: {}
                "#,
        is_x86_feature_detected!("avx"),
        is_x86_feature_detected!("sse"),
    );
    unimplemented!("find_key is not implemented for the current CPU")
}
