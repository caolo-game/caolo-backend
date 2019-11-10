use caolo_api::{
    bots::Bots,
    point::{Circle, Point},
    resources::Resources,
    structures::Structures,
    terrain::TileTerrainType,
};
use caolo_engine::api::{build_bot, build_resource, build_structure};
use caolo_engine::model;
use caolo_engine::storage::Storage;
use caolo_engine::tables::PositionTable;

/// terrain is a list of non-plain terrain types
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    bots: Bots,
    structures: Structures,
    resources: Resources,

    terrain: Vec<(Point, TileTerrainType)>,

    delta_time_ms: i64,
    time: u64,
}

impl Payload {
    // TODO: pass circle as vision
    pub fn new(storage: &Storage, vision: Circle) -> Self {
        let ids = storage
            .entity_table::<model::PositionComponent>()
            .get_entities_in_range(&vision)
            .into_iter()
            .map(|(e, _)| e)
            .collect::<Vec<_>>();

        let bots = {
            let bots = ids
                .iter()
                .filter_map(|e| build_bot(*e, storage))
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
            storage
                .point_table::<model::TileTerrainType>()
                .iter()
                .filter(|(p, t)| vision.is_inside(*p) && *t != TileTerrainType::Empty)
                .collect()
        };

        let resources = {
            let resources = storage
                .entity_table::<model::Resource>()
                .iter()
                .filter_map(|(id, r)| build_resource(id, r, storage))
                .collect();
            Resources::new(resources)
        };

        let dt = storage.delta_time();
        let delta_time_ms = dt.num_milliseconds();
        let time = storage.time();

        Self {
            bots,
            structures,
            terrain,
            resources,
            delta_time_ms,
            time,
        }
    }
}
