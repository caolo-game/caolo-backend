mod ser_bots;
mod ser_resources;
mod ser_structures;

use caolo_sim::prelude::{Axial, Hexagon, TerrainComponent, World};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{
    broadcast::{error::RecvError, Sender},
    mpsc,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::{info, log::warn, Instrument};

use crate::protos::cao_common;
use crate::protos::cao_world;

#[derive(Clone)]
pub struct WorldService {
    entities: WorldPayloadSender,
    room_bounds: Hexagon,
    terrain: Arc<HashMap<Axial, Vec<TerrainComponent>>>,
    tracing_span: tracing::Span,
}

impl std::fmt::Debug for WorldService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldService").finish()
    }
}

type WorldPayloadSender = Arc<Sender<Arc<Payload>>>;

#[derive(Default, Debug)]
pub struct Payload {
    pub payload: Vec<cao_world::WorldEntities>,
}

impl WorldService {
    pub fn new(
        entities: WorldPayloadSender,
        room_bounds: Hexagon,
        terrain: Arc<HashMap<Axial, Vec<TerrainComponent>>>,
        span: tracing::Span,
    ) -> Self {
        Self {
            entities,
            room_bounds,
            terrain,
            tracing_span: span,
        }
    }
}

impl Payload {
    /// Transform the usual json serialized world into Payload
    pub fn update(&mut self, world: &World) {
        self.payload.clear();
        ser_bots::bot_payload(&mut self.payload, caolo_sim::prelude::FromWorld::new(world));
        ser_structures::structure_payload(
            &mut self.payload,
            caolo_sim::prelude::FromWorld::new(world),
        );
        ser_resources::resource_payload(
            &mut self.payload,
            caolo_sim::prelude::FromWorld::new(world),
        );
    }
}

#[tonic::async_trait]
impl cao_world::world_server::World for WorldService {
    type EntitiesStream = ReceiverStream<Result<cao_world::WorldEntities, Status>>;

    async fn entities(
        &self,
        _r: tonic::Request<cao_world::Empty>,
    ) -> Result<tonic::Response<Self::EntitiesStream>, tonic::Status> {
        let addr = _r.remote_addr();

        info!("Subscribing new client to world entities. Addr: {:?}", addr);

        let (tx, rx) = mpsc::channel(4);

        let mut entities_rx = self.entities.subscribe();
        tokio::spawn(
            async move {
                'main_send: loop {
                    let w = match entities_rx.recv().await {
                        Ok(w) => w,
                        Err(RecvError::Lagged(l)) => {
                            warn!("Entities stream is lagging behind by {} messages", l);
                            continue 'main_send;
                        }
                        Err(RecvError::Closed) => {
                            warn!("Entities channel was closed");
                            break 'main_send;
                        }
                    };
                    for pl in w.payload.iter() {
                        if tx.send(Ok(pl.clone())).await.is_err() {
                            info!("World entities client lost {:?}", addr);
                            break 'main_send;
                        }
                    }
                }
            }
            .instrument(self.tracing_span.clone()),
        );

        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }

    async fn get_room_layout(
        &self,
        _: tonic::Request<cao_world::Empty>,
    ) -> Result<tonic::Response<cao_world::RoomLayout>, tonic::Status> {
        let positions = self
            .room_bounds
            .iter_points()
            .map(|point| cao_common::Axial {
                q: point.q,
                r: point.r,
            })
            .collect();
        Ok(tonic::Response::new(cao_world::RoomLayout { positions }))
    }

    async fn get_room_terrain(
        &self,
        request: tonic::Request<cao_common::Axial>,
    ) -> Result<tonic::Response<cao_world::RoomTerrain>, tonic::Status> {
        let q = request.get_ref().q;
        let r = request.get_ref().r;
        let p = Axial::new(q, r);
        let room = self
            .terrain
            .get(&p)
            .ok_or_else(|| tonic::Status::not_found("Room does not exist"))?;

        Ok(tonic::Response::new(cao_world::RoomTerrain {
            room_id: Some(cao_common::Axial { q, r }),
            tiles: room
                .iter()
                .map(|TerrainComponent(t)| match t {
                    caolo_sim::terrain::TileTerrainType::Empty => cao_world::Terrain::Empty,
                    caolo_sim::terrain::TileTerrainType::Plain => cao_world::Terrain::Plain,
                    caolo_sim::terrain::TileTerrainType::Bridge => cao_world::Terrain::Bridge,
                    caolo_sim::terrain::TileTerrainType::Wall => cao_world::Terrain::Wall,
                })
                .map(|t| t.into())
                .collect(),
        }))
    }

    async fn get_room_list(
        &self,
        _: tonic::Request<cao_world::Empty>,
    ) -> Result<tonic::Response<cao_world::RoomList>, tonic::Status> {
        let room_ids = self
            .terrain
            .keys()
            .map(|point| cao_common::Axial {
                q: point.q,
                r: point.r,
            })
            .collect();
        Ok(tonic::Response::new(cao_world::RoomList { room_ids }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// test the bare minimum
    #[test]
    fn can_update_payload() {
        let mut pl = Payload::default();

        let mut exc = caolo_sim::prelude::SimpleExecutor;
        let mut w = exc.initialize(caolo_sim::executor::GameConfig {
            world_radius: 2,
            room_radius: 10,
            ..Default::default()
        });
        caolo_sim::init::init_world_entities(&mut *w, 12);

        pl.update(&w);

        assert!(!pl.payload.is_empty());
    }
}
