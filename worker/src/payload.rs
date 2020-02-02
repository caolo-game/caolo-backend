use caolo_api::{
    bots::Bots, point::Point, resources::Resources, structures::Structures,
    terrain::TileTerrainType,
};
use caolo_sim::api::{build_bot, build_resource, build_structure};
use caolo_sim::model;
use caolo_sim::storage::Storage;
use caolo_sim::tables::LogTable;
use serde_derive::Serialize;
use std::collections::HashMap;

/// terrain is a list of non-plain terrain types
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub bots: Bots,
    pub structures: Structures,
    pub resources: Resources,

    pub terrain: Vec<(Point, TileTerrainType)>,

    pub log: HashMap<caolo_api::EntityId, String>,

    pub delta_time_ms: i64,
    pub time: u64,
}

impl Payload {
    pub fn new(storage: &Storage) -> Self {
        let ids = storage
            .entity_table::<model::PositionComponent>()
            .iter()
            .map(|(e, _)| e)
            .collect::<Vec<_>>();

        let bots = {
            let bots = ids
                .iter()
                .filter_map(|e| build_bot(*e, From::from(storage)))
                .collect::<Vec<_>>();
            Bots::new(bots)
        };

        let structures = {
            let structures = ids
                .iter()
                .filter_map(|e| build_structure(*e, storage))
                .collect::<Vec<_>>();

            Structures::new(structures)
        };

        let terrain = {
            let mut terrain = Vec::with_capacity(512);
            storage
                .point_table::<model::TerrainComponent>()
                .find_by_range(&Point::default(), 256, &mut terrain);
            terrain
                .iter()
                .filter(|(_, t)| t.0 != TileTerrainType::Empty)
                .map(|(x, y)| (x.clone(), y.0.clone()))
                .collect()
        };

        let resources = {
            let resources = storage
                .entity_table::<model::ResourceComponent>()
                .iter()
                .filter_map(|(id, r)| build_resource(id, r.clone(), storage))
                .collect();
            Resources::new(resources)
        };

        let time = storage.time() - 1; // the simulation increases time after the update is done
        let log = storage
            .log_table::<model::LogEntry>()
            .get_logs_by_time(time)
            .into_iter()
            .map(|(t, pl)| (t.0, pl.payload.join("\n")))
            .collect();

        let dt = storage.delta_time();
        let delta_time_ms = dt.num_milliseconds();

        Self {
            bots,
            structures,
            terrain,
            resources,
            delta_time_ms,
            time,
            log,
        }
    }
}
