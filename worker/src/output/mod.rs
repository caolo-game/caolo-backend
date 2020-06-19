use caolo_messages::Bot as BotMsg;
use caolo_messages::LogEntry as LogMsg;
use caolo_messages::{AxialPoint, TerrainType, Tile as TileMsg, WorldPosition as WorldPositionMsg};
use caolo_messages::{Resource as ResourceMsg, ResourceType};
use caolo_messages::{
    Structure as StructureMsg, StructurePayload as StructurePayloadMsg,
    StructureSpawn as StructureSpawnMsg,
};
use caolo_sim::components::{
    Bot, EnergyComponent, LogEntry, OwnedEntity, PositionComponent, Resource, ResourceComponent,
    SpawnComponent, Structure, TerrainComponent,
};
use caolo_sim::model::{indices::EntityTime, terrain::TileTerrainType, EntityId, WorldPosition};
use caolo_sim::storage::views::View;
use caolo_sim::tables::JoinIterator;

type BotInput<'a> = (
    View<'a, EntityId, Bot>,
    View<'a, EntityId, PositionComponent>,
    View<'a, EntityId, OwnedEntity>,
);

pub fn build_bots<'a>(
    (bots, positions, owned_entities): BotInput<'a>,
) -> impl Iterator<Item = BotMsg> + 'a {
    let bots = bots.reborrow().iter();
    let positions = positions.reborrow().iter();
    JoinIterator::new(bots, positions).map(move |(id, (_bot, pos))| {
        let msg = BotMsg {
            id: id.0,
            position: init_world_pos(pos.0),
            owner: owned_entities
                .get_by_id(&id)
                .map(|OwnedEntity { owner_id }| owner_id.0),
        };
        msg
    })
}

pub fn build_logs<'a>(v: View<'a, EntityTime, LogEntry>) -> impl Iterator<Item = LogMsg> + 'a {
    v.reborrow()
        .iter()
        .map(|(EntityTime(EntityId(eid), time), entries)| LogMsg {
            entity_id: eid,
            time,
            payload: entries.payload.iter().cloned().collect(),
        })
}

pub fn build_terrain<'a>(
    v: View<'a, WorldPosition, TerrainComponent>,
) -> impl Iterator<Item = TileMsg> + 'a {
    v.reborrow().iter().map(|(pos, TerrainComponent(tile))| {
        let msg = TileMsg {
            position: init_world_pos(pos),
            ty: match tile {
                TileTerrainType::Plain => TerrainType::Plain,
                TileTerrainType::Wall => TerrainType::Wall,
                TileTerrainType::Bridge => TerrainType::Bridge,
            },
        };
        msg
    })
}

type ResourceInput<'a> = (
    View<'a, EntityId, ResourceComponent>,
    View<'a, EntityId, PositionComponent>,
    View<'a, EntityId, EnergyComponent>,
);

pub fn build_resources<'a>(
    (resource_table, position_table, energy_table): ResourceInput<'a>,
) -> impl Iterator<Item = ResourceMsg> + 'a {
    let join = JoinIterator::new(
        resource_table.reborrow().iter(),
        position_table.reborrow().iter(),
    );

    JoinIterator::new(join, energy_table.reborrow().iter()).map(
        |(id, ((resource, pos), energy))| match resource.0 {
            Resource::Energy => {
                let msg = ResourceMsg {
                    id: id.0,
                    position: init_world_pos(pos.0),
                    ty: ResourceType::Energy {
                        energy: energy.energy as u32,
                        energy_max: energy.energy_max as u32,
                    },
                };
                msg
            }
        },
    )
}

type StructuresInput<'a> = (
    View<'a, EntityId, Structure>,
    View<'a, EntityId, SpawnComponent>,
    View<'a, EntityId, PositionComponent>,
    View<'a, EntityId, OwnedEntity>,
);

pub fn build_structures<'a>(
    (structure_table, spawn_table, position_table, owner_table): StructuresInput<'a>,
) -> impl Iterator<Item = StructureMsg> + 'a {
    let spawns = JoinIterator::new(
        spawn_table.reborrow().iter(),
        structure_table.reborrow().iter(),
    );
    JoinIterator::new(spawns, position_table.reborrow().iter()).map(
        move |(id, ((spawn, _structure), pos))| {
            let msg = StructureMsg {
                id: id.0,
                position: init_world_pos(pos.0),
                owner: owner_table
                    .get_by_id(&id)
                    .map(|OwnedEntity { owner_id }| owner_id.0),
                payload: StructurePayloadMsg::Spawn(StructureSpawnMsg {
                    spawning: spawn.spawning.map(|EntityId(id)| id),
                    time_to_spawn: spawn.time_to_spawn as i32,
                }),
            };
            msg
        },
    )
}

fn init_world_pos(pos: WorldPosition) -> WorldPositionMsg {
    WorldPositionMsg {
        room: AxialPoint {
            q: pos.room.q,
            r: pos.room.r,
        },
        pos: AxialPoint {
            q: pos.pos.q,
            r: pos.pos.r,
        },
    }
}
