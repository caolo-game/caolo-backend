use crate::protos::world::Bot as BotMsg;
use crate::protos::world::LogEntry as LogMsg;
use crate::protos::world::{Resource as ResourceMsg, Resource_ResourceType};
use crate::protos::world::{Structure as StructureMsg, StructureSpawn as StructureSpawnMsg};
use crate::protos::world::{Tile as TileMsg, Tile_TerrainType};
use caolo_sim::model::{
    components::{
        Bot, EnergyComponent, LogEntry, OwnedEntity, PositionComponent, Resource,
        ResourceComponent, SpawnComponent, Structure, TerrainComponent,
    },
    indices::EntityTime,
    terrain::TileTerrainType,
    EntityId, WorldPosition,
};
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
        let mut msg = BotMsg::default();
        msg.set_id(id.0);
        let msg_pos = msg.mut_position();
        init_world_pos(msg_pos, pos.0);
        msg.mut_owner().clear();
        if let Some(owner) = owned_entities.get_by_id(&id) {
            *msg.mut_owner() = owner.owner_id.0.as_bytes().to_vec();
        }
        msg
    })
}

pub fn build_logs<'a>(v: View<'a, EntityTime, LogEntry>) -> impl Iterator<Item = LogMsg> + 'a {
    v.reborrow()
        .iter()
        .map(|(EntityTime(EntityId(eid), time), entries)| {
            let mut msg = LogMsg::new();
            msg.set_entity_id(eid);
            msg.set_time(time);
            for e in entries.payload.iter() {
                msg.mut_payload().push(e.clone());
            }
            msg
        })
}

pub fn build_terrain<'a>(
    v: View<'a, WorldPosition, TerrainComponent>,
) -> impl Iterator<Item = TileMsg> + 'a {
    v.reborrow().iter().map(|(pos, tile)| {
        let mut msg = TileMsg::new();
        let msg_pos = msg.mut_position();
        init_world_pos(msg_pos, pos);
        match tile.0 {
            TileTerrainType::Bridge => {
                msg.set_ty(Tile_TerrainType::BRIDGE);
            }
            TileTerrainType::Plain => {
                msg.set_ty(Tile_TerrainType::PLAIN);
            }
            TileTerrainType::Wall => {
                msg.set_ty(Tile_TerrainType::WALL);
            }
        }
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
                let mut msg = ResourceMsg::new();
                msg.set_id(id.0);
                let msg_pos = msg.mut_position();
                init_world_pos(msg_pos, pos.0);
                msg.set_ty(Resource_ResourceType::ENERGY);
                msg.set_energy(energy.energy as u32);
                msg.set_energyMax(energy.energy_max as u32);
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
            let mut msg = StructureMsg::new();
            msg.set_id(id.0);
            let msg_pos = msg.mut_position();
            init_world_pos(msg_pos, pos.0);
            msg.mut_owner().clear();
            if let Some(owner) = owner_table.get_by_id(&id) {
                *msg.mut_owner() = owner.owner_id.0.as_bytes().to_vec();
            }
            let mut payload = StructureSpawnMsg::new();
            payload.set_time_to_spawn(spawn.time_to_spawn as i32);
            if let Some(spawning) = spawn.spawning {
                payload.set_spawning(spawning.0);
            }
            msg.set_spawn(payload);
            msg
        },
    )
}

fn init_world_pos(msg_pos: &mut crate::protos::world::WorldPosition, pos: WorldPosition) {
    msg_pos.mut_room().set_q(pos.room.q);
    msg_pos.mut_room().set_r(pos.room.r);
    msg_pos.mut_pos().set_q(pos.pos.q);
    msg_pos.mut_pos().set_r(pos.pos.r);
}
