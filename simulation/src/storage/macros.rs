#[macro_export(local_inner_macros)]
macro_rules! implement_table_type {
    ($field_name: ident, $getter: ident, $mutter: ident, $setter: ident,$deleter: ident, $id: tt) =>{
        pub fn $getter<'a, Row: TableRow>(&'a self) -> &'a Table<$id, Row> {
            let rowtype = TypeId::of::<Row>();
            self.$field_name
                .get(&rowtype)
                .and_then(|a| a.downcast_ref())
                .ok_or_else(|| {
                    log::error!(
                        "Table {:?} was not registered",
                        type_name::<($id, Row)>()
                    )
                })
            .expect("Table was not registered!")
        }

        pub fn $mutter<'a, Row: TableRow>(
            &'a mut self,
            ) -> &'a mut Table<$id, Row> {
            let rowtype = TypeId::of::<Row>();
            self.$field_name
                .get_mut(&rowtype)
                .and_then(|a| a.downcast_mut())
                .ok_or_else(|| {
                    log::error!(
                        "Table {:?} was not registered",
                        type_name::<($id, Row)>()
                    )
                })
            .expect("Table was not registered!")
        }

        pub fn $setter<Row: TableRow + Sync>(
            &mut self,
            table: Table<$id, Row>,
            ) {
            let id = TypeId::of::<Row>();
            self.$field_name.insert(id, HomogenousTable::new(table));
        }

        pub fn $deleter(&mut self, id: $id) {
            for (_key, table) in self.$field_name.iter_mut() {
                table.delete_entity(&id);
            }
        }

    }
}
