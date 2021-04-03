/// # Mutation Queries
///
/// Designed to reduce boilerplate when updating tables.
/// The query does nothing special, just calls the provided methods on the tables
/// obtained by `unsafe_view::<key, value>()`.
///
/// ## Safety
///
/// Updates on Storage are an unsafe operation. Be sure that no other threads have write access to
/// the tables you're mutating!
///
/// ```
/// use caolo_sim::query;
/// use caolo_sim::prelude::*;
///
/// let mut store = SimpleExecutor.initialize(None, caolo_sim::executor::GameConfig {
///     world_radius: 1,
///     room_radius: 10,
///     ..Default::default()
/// },
/// ).unwrap();
///
/// let entity_1 = store.insert_entity();
/// let entity_2 = store.insert_entity();
///
/// query!(
///     mutate
///     store
///     {
///         EntityId, Bot, .insert(entity_1);
///         EntityId, Bot, .insert(entity_2);
///         EntityId, CarryComponent,
///                  .insert_or_update(entity_1, CarryComponent{carry: 12, carry_max: 69});
///         EntityId, CarryComponent,
///                  .insert_or_update(entity_2, CarryComponent{carry: 0, carry_max: 69});
///     }
/// );
/// ```
#[macro_export]
macro_rules! query {
    (
        mutate
        $store: ident
        {
        $(
            $id: ty, $row: ty, $(.$op: ident ( $($args: expr),* ))*
        );*;
        }
    ) => {
        {
            $(
                $store.unsafe_view::<$id, $row>()
                    . $(
                        $op($($args),*)
                    ).*
            );*
        }
    };
}

///
///# Examples
///
///## Join iterators
///
///```
/// use caolo_sim::query;
/// use caolo_sim::prelude::*;
/// use caolo_sim::join;
/// use caolo_sim::tables::JoinIterator;
///
/// let mut store = SimpleExecutor.initialize(None, caolo_sim::executor::GameConfig {
///     world_radius: 1,
///     room_radius: 10,
///     ..Default::default()
/// },
/// ).unwrap();
///
/// let entity_1 = store.insert_entity();
/// let entity_2 = store.insert_entity();
/// let entity_3 = store.insert_entity();
///
/// // Initialize entities ...
///
/// query!(
///    mutate
///    store
///    {
///        EntityId, Bot, .insert(entity_1);
///        EntityId, Bot, .insert(entity_2);
///
///        EntityId, PositionComponent, .insert_or_update(entity_1, PositionComponent::default());
///        EntityId, PositionComponent, .insert_or_update(entity_2, PositionComponent::default());
///        EntityId, PositionComponent, .insert_or_update(entity_3, PositionComponent::default());
///
///        // notice how entity_3 is not a bot, but has carry
///
///        EntityId, CarryComponent,
///                 .insert_or_update(entity_1, CarryComponent{carry: 12, carry_max: 69});
///        EntityId, CarryComponent,
///                 .insert_or_update(entity_2, CarryComponent{carry: 30, carry_max: 69});
///        EntityId, CarryComponent,
///                 .insert_or_update(entity_3, CarryComponent{carry: 40, carry_max: 69});
///    }
/// );
///
/// let bot_table = store.view::<EntityId, Bot>();
/// let bot = bot_table.iter();
/// let pos_table = store.view::<EntityId, PositionComponent>();
/// let pos = pos_table.iter();
/// let carry_table = store.view::<EntityId, CarryComponent>();
/// let car = carry_table.iter();
///
/// let res: i32 = join!( [ bot , pos , car ])
///     // data has fields `carry` and `bot`, specified in the macro invocation
///     // these are references to their respective components...
///     // we'll extract the carry amount
///     //
///     // pos_components are default (0,0), we access them for demo purposes...
///     .map(|(id, (bot, pos, car))|{ car.carry as i32 + pos.0.pos.q })
///     .sum();
///
/// assert_eq!(res, 42); // entity_1 carry + entity_2 carry
///```
///
///## Join on storage
///
///```
/// use caolo_sim::query;
/// use caolo_sim::prelude::*;
///
/// // these rows are mandatory
/// use caolo_sim::join;
/// use caolo_sim::tables::JoinIterator;
///
/// let mut store = SimpleExecutor.initialize(None, caolo_sim::executor::GameConfig {
///     world_radius: 1,
///     room_radius: 10,
///     ..Default::default()
/// },
/// ).unwrap();
///
/// let entity_1 = store.insert_entity();
/// let entity_2 = store.insert_entity();
/// let entity_3 = store.insert_entity();
///
/// query!(
///     mutate
///     store
///     {
///         EntityId, Bot, .insert(entity_1);
///         EntityId, Bot, .insert(entity_2);
///
///         EntityId, PositionComponent, .insert_or_update(entity_1, PositionComponent::default());
///         EntityId, PositionComponent, .insert_or_update(entity_2, PositionComponent::default());
///         EntityId, PositionComponent, .insert_or_update(entity_3, PositionComponent::default());
///
///         // notice how entity_3 is not a bot, but has carry
///
///         EntityId, CarryComponent,
///                  .insert_or_update(entity_1, CarryComponent{carry: 12, carry_max: 69});
///         EntityId, CarryComponent,
///                  .insert_or_update(entity_2, CarryComponent{carry: 30, carry_max: 69});
///         EntityId, CarryComponent,
///                  .insert_or_update(entity_3, CarryComponent{carry: 40, carry_max: 69});
///     }
/// );
///
/// let res: i32 = join!(
///       store
///       EntityId
///       [ bot : Bot,
///         pos_component : PositionComponent,
///         carry_component : CarryComponent ]
///     )
///     // data has fields `carry` and `bot`, specified in the macro invocation
///     // these are references to their respective components...
///     // we'll extract the carry amount
///     //
///     // pos_components are default (0,0), we access them for demo purposes...
///     .map(|(id, (_bot_component, pos_component, carry_component))| {
///         carry_component.carry as i32 + pos_component.0.pos.q
///     })
///     .sum();
///
/// assert_eq!(res, 42); // entity_1 carry + entity_2 carry
///```
#[macro_export]
macro_rules! join {
    (
        [
            $it0: ident,
            $(
                $its: ident
            ),+
        ]
    ) => {{
        join!(@iter $it0, $($its),*)
            .map(
                // closure taking id and a nested tuple of pairs
                |(
                    id,
                    join!(@args $it0, $($its),*)
                 )| {
                    (id,
                     // flatten the tuple
                     ($it0, $($its),*)
                    )
                }
            )
    }};

    (
        $storage: ident
        $id: ty
        [
            $name0: ident : $row0: ty,
            $(
                $names: ident : $rows: ty
            ),+
        ]
    ) => {{
        join!(@join $storage $id, $row0, $($rows),*)
            .map(
                // closure taking id and a nested tuple of pairs
                |(
                    id,
                    join!(@args $name0, $($names),*)
                 )| {
                    (id,
                     // flatten the tuple
                     ($name0, $($names),*)
                    )
                }
            )
    }};

    (@iter $it: ident) => {
        $it
    };

    (@iter $head: ident,
            $(
                $tail: ident
            ),+
    ) => {
        JoinIterator::new(
            $head,
            join!(@iter $($tail),*)
        )
    };

    (@join $storage: ident $id: ty, $row: ty) => {
        // stop the recursion
        $storage.view::<$id, $row>().iter()
    };

    (@join $storage: ident $id: ty, $row0: ty,
            $(
                $rows: ty
            ),+
    ) => {
        JoinIterator::new(
            $storage.view::<$id, $row0>().iter(),
            join!(@join $storage $id, $($rows),*)
        )
    };

    (@args $name: ident) => {
        // stop the recursion
        $name
    };

    (@args $name0: ident, $( $names: ident),+) => {
        // nested tuple arguments
        (
         $name0,
         join!(@args $( $names),*)
        )
    };
}

/// # `iterby`
///
///`iterby` will create `iterby_<name>` methods, that will iterate on the given table and output
/// a tuple (struct) with all optional fields.
/// To use this the given table must have an `iter` method returning a pair of (key, value), only
/// for keys that are in the given table.
/// Will call `get_by_id` for all other tables.
///
/// For this reason I do not recommend using this often, as hand written `join!`-s can be a lot
/// more performant.
///
/// This is mostly here for serialization, when communicating with clients.
#[macro_export(local_inner_macros)]
macro_rules! archetype {
    (
        module $module: ident
        key $id: ty,
        $(
            $(primary)* $(attr $attr: meta )* table $row: ty = $name: ident
        ),*

        $(
            iterby $it: ident
        )*
    ) => {
        pub mod $module {
            use super::*;
            use crate::tables::Table;
            use crate::storage::views::{UnsafeView, View};
            use crate::storage::{HasTable, DeleteById, DeferredDeleteById};
            use serde::{Serialize, Deserialize};
            pub type Key = $id;

            #[derive(Debug, cao_storage_derive::CaoStorage, Default, Serialize, Deserialize)]
            $(
                #[cao_storage_table($id, $name, $row)]
            )*
            $(
                #[cao_storage_iterby($it, $id)]
            )*
            pub struct Archetype {
                $( $(#[ $attr ] )*
                pub(crate) $name: <$row as crate::tables::Component<$id>>::Table ),
                +,
            }

            archetype!(@implement_tables $($name, $id,  $row )*);

            impl Archetype {
                #[allow(unused)]
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    $(
                        $name: <$row as crate::tables::Component<$id>>::Table
                    ),*
                ) -> Self {
                    Self {
                        $( $name ),*
                    }
                }
            }
        }
    };

    (
        @implement_tables
        $($name: ident, $id: ty,  $row: ty )*
    ) => {
        $(
            impl HasTable<$id, $row> for Archetype {
                fn view(&'_ self) -> View<'_, $id, $row>{
                    View::from_table(&self.$name)
                }

                fn unsafe_view(&mut self) -> UnsafeView<$id, $row>{
                    UnsafeView::from_table(&mut self.$name)
                }
            }
        )*
    };
}

/// Create an empty struct that can be used a Storage key
#[macro_export(local_inner_macros)]
macro_rules! empty_key {
    ($name: ident) => {
        #[derive(
            Debug,
            Clone,
            Default,
            Ord,
            PartialOrd,
            Eq,
            PartialEq,
            Copy,
            Hash,
            Serialize,
            Deserialize,
        )]
        pub struct $name;

        unsafe impl Send for $name {}
    };
}
