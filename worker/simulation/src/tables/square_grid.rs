use std::convert::TryInto;

use crate::{geometry::Axial, prelude::Hexagon};

use super::{Table, TableRow};

/// The grid is always touching the origin
#[derive(Clone)]
pub struct SquareGrid<T> {
    radius: i32,
    values: Vec<T>,
}

impl<T> SquareGrid<T> {
    pub fn new(radius: usize) -> Self
    where
        T: Default + Clone,
    {
        let diameter = (radius * 2) + 1;
        let area = diameter * diameter;
        Self {
            radius: radius.try_into().expect("Radius must fit into 30 bits"),
            values: vec![Default::default(); area],
        }
    }

    fn diamater(radius: i32) -> i32 {
        radius * 2 + 1
    }

    pub fn contains_key(&self, pos: Axial) -> bool {
        let row = pos.r;
        let col = pos.q;

        let diameter = Self::diamater(self.radius);

        col <= diameter && row <= diameter && col >= 0 && row >= 0
    }

    #[inline]
    pub fn get(&self, pos: Axial) -> Option<&T> {
        let ind = self.get_index(pos)?;
        self.values.get(ind)
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, Axial { q, r }: Axial) -> &T {
        let diameter = self.radius * 2 + 1;
        let ind = r as usize * diameter as usize + q as usize;

        self.values.get_unchecked(ind)
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, Axial { q, r }: Axial) -> &T {
        let diameter = self.radius * 2 + 1;
        let ind = r as usize * diameter as usize + q as usize;

        self.values.get_unchecked_mut(ind)
    }

    #[inline]
    pub fn get_mut(&mut self, pos: Axial) -> Option<&mut T> {
        let ind = self.get_index(pos)?;
        self.values.get_mut(ind)
    }

    pub fn resize(&mut self, new_radius: i32)
    where
        T: Default,
    {
        debug_assert!(new_radius >= 0);

        let diameter = Self::diamater(new_radius);
        let area = diameter * diameter;

        self.radius = new_radius;
        self.values.resize_with(area as usize, Default::default);
    }

    /// return the existing value if successful.
    ///
    /// return None if the position is invalid
    pub fn insert(&mut self, pos: Axial, val: T) -> Option<T> {
        let old = self.get_mut(pos)?;
        Some(std::mem::replace(old, val))
    }

    pub fn query_range(&self, query: Hexagon, mut op: impl FnMut(Axial, &T)) {
        for p in query.iter_points() {
            if let Some(t) = self.get(p) {
                op(p, t);
            }
        }
    }

    fn get_index(&self, pos: Axial) -> Option<usize> {
        let row = pos.r;
        let col = pos.q;

        let radius = self.radius;
        let diameter = Self::diamater(radius);

        if col < 0 || row < 0 {
            return None;
        }
        let ind = row as usize * diameter as usize + col as usize;
        Some(ind)
    }
}

impl<T> Table for SquareGrid<T>
where
    T: TableRow + Default,
{
    type Id = Axial;
    type Row = T;

    fn delete(&mut self, id: Self::Id) -> Option<Self::Row> {
        self.get_mut(id).map(|x| std::mem::take(x))
    }

    fn get_by_id(&self, id: Self::Id) -> Option<&Self::Row> {
        self.get(id)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::Hexagon;

    use super::*;

    #[test]
    fn can_access_all_elements_in_hexagon() {
        let grid = SquareGrid::<()>::new(3);

        for p in Hexagon::from_radius(3).iter_points() {
            println!("{:?}", p);
            grid.get(p).unwrap();
        }
    }
}
