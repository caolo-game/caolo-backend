use super::*;
use crate::model::{
    components::LogEntry,
    indices::{EntityId, EntityTime},
};

#[derive(Debug, Clone)]
pub struct LogIntent {
    pub entity: EntityId,
    pub payload: String,
    pub time: u64,
}

impl LogIntent {
    pub fn execute(&self, storage: &mut Storage) -> IntentResult {
        let id = EntityTime(self.entity, self.time);
        let table = storage.log_table_mut::<LogEntry>();
        let entry = match table.get_by_id(&id).cloned() {
            Some(mut entry) => {
                entry.payload.push(self.payload.clone());
                entry
            }
            None => LogEntry {
                payload: vec![self.payload.clone()],
            },
        };
        table.insert_or_update(id, entry);

        Ok(())
    }
}
