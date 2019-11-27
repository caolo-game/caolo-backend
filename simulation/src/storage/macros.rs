#[macro_export(local_inner_macros)]
macro_rules! implement_table_type {
    ($field_name: ident, $getter: ident, $mutter: ident, $setter: ident,$deleter: ident, $id: tt) =>{
        pub fn $getter<'a, Row: Component<$id>>(&'a self) -> &'a Row::Table {
            let rowtype = TypeId::of::<Row>();
            self.$field_name
                .get(&rowtype)
                .and_then(|a| a.downcast_ref::<Row>())
                .ok_or_else(|| {
                    log::error!(
                        "Table {:?} was not registered",
                        type_name::<($id, Row)>()
                    );
                    std::format!("{:?}", type_name::<($id, Row)>())
                })
            .expect("Table was not registered!")
        }

        pub fn $mutter<'a, Row: Component<$id>>(&'a mut self) -> &'a mut Row::Table {
            let rowtype = TypeId::of::<Row>();
            self.$field_name
                .get_mut(&rowtype)
                .and_then(|a| a.downcast_mut::<Row>())
                .ok_or_else(|| {
                    log::error!(
                        "Table {:?} was not registered",
                        type_name::<($id, Row)>()
                    );
                    std::format!("{:?}", type_name::<($id, Row)>())
                })
            .expect("Table was not registered!")
        }

        pub fn $setter<Row: Component<$id> + Sync>( &mut self, table: <Row as Component<$id>>::Table )
            where <Row as Component<$id>>::Table: crate::storage::homogenoustable::DynTable::<$id>
        {
            let id = TypeId::of::<Row>();
            self.$field_name.insert(id, HomogenousTable::new::<Row>(table));
        }

        pub fn $deleter(&mut self, id: $id) {
            for (_key, table) in self.$field_name.iter_mut() {
                table.delete_entity(&id);
            }
        }

    }
}
