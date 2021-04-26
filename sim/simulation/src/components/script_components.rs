use crate::indices::EntityId;
use cao_lang::{prelude, program::CaoProgram};
use prelude::CaoIr;
use serde::{Deserialize, Serialize};

/// Currently does nothing as Cao-Lang not yet supports history
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScriptHistoryEntry {
    pub entity_id: EntityId,
    pub time: u64,
}

/// Currently does nothing as Cao-Lang not yet supports history
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScriptHistory(());

/// Entities with Scripts
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompiledScriptComponent(pub CaoProgram);

/// Pre-compiled scripts
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaoIrComponent(pub CaoIr);

impl Default for CaoIrComponent {
    fn default() -> Self {
        Self(CaoIr {
            lanes: Default::default(),
        })
    }
}
