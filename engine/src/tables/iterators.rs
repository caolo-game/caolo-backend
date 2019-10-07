use super::*;
use std::marker::PhantomData;

pub trait TableIterator<Id, Row1>: Iterator<Item = (Id, Row1)> {}

impl<Id, Row, T> TableIterator<Id, Row> for T where T: Iterator<Item = (Id, Row)> {}

/// Joins entities with the same ids in both ranges, yielding entities which are present in both
/// __Contract__: Both input iterators must be sorted by their ids!
pub struct JoinIterator<'a, T1, T2, Id, It1, It2>
where
    Id: TableId,
    It1: TableIterator<Id, T1> + 'a,
    It2: TableIterator<Id, T2> + 'a,
{
    t1: It1,
    t2: It2,

    _m: PhantomData<&'a (T1, T2, Id)>,
}

impl<'a, T1, T2, Id, It1, It2> JoinIterator<'a, T1, T2, Id, It1, It2>
where
    Id: TableId,
    It1: TableIterator<Id, T1> + 'a,
    It2: TableIterator<Id, T2> + 'a,
{
    pub fn new(t1: It1, t2: It2) -> Self {
        Self {
            t1,
            t2,
            _m: Default::default(),
        }
    }
}

impl<'a, T1, T2, Id, It1, It2> Iterator for JoinIterator<'a, T1, T2, Id, It1, It2>
where
    Id: TableId,
    It1: TableIterator<Id, T1> + 'a,
    It2: TableIterator<Id, T2> + 'a,
{
    type Item = (Id, (T1, T2));

    fn next(&mut self) -> Option<Self::Item> {
        let mut row1 = self.t1.next();
        let mut row2 = self.t2.next();

        while row1.is_some() && row2.is_some() {
            let r1 = row1.as_ref().unwrap();
            let r2 = row2.as_ref().unwrap();
            if r1.0 == r2.0 {
                return row1.and_then(|(id, r1)| row2.map(|(_, r2)| (id, (r1, r2))));
            } else if r1.0 < r2.0 {
                let _is_less_than_last = {
                    let id = r1.0;
                    move |row| is_less_than_last(id, row)
                };

                row1 = self.t1.next();

                debug_assert!(
                    _is_less_than_last(row1.as_ref()),
                    "Items of Iterator 1 were not ordered!"
                );
            } else if r2.0 < r1.0 {
                let _is_less_than_last = {
                    let id = r2.0;
                    move |row| is_less_than_last(id, row)
                };

                row2 = self.t2.next();

                debug_assert!(
                    _is_less_than_last(row2.as_ref()),
                    "Items of Iterator 2 were not ordered!"
                );
            }
        }
        None
    }
}

#[allow(unused)]
fn is_less_than_last<Id: TableId, T>(id: Id, val: Option<&(Id, T)>) -> bool {
    val.map(|(i, _)| id < *i).unwrap_or(true)
}
