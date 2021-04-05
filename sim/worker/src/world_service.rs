use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

use crate::protos::cao_common;
use crate::protos::cao_world;

#[derive(Clone)]
pub struct WorldService {
    logger: slog::Logger,
    entities: Arc<RwLock<Payload>>,
}

#[derive(Default, Debug)]
pub struct Payload {
    pub world_time: i64,
    pub payload: cao_world::WorldEntities,
}

impl WorldService {
    pub fn new(logger: slog::Logger, entities: Arc<RwLock<Payload>>) -> Self {
        Self { logger, entities }
    }
}

fn write_world_payload(
    world_payload: &serde_json::Value,
    key: &str,
    out: &mut ::prost::alloc::vec::Vec<cao_world::RoomObjects>,
) {
    use cao_common::{Axial, Json};
    use cao_world::RoomObjects;

    for (roomid, pl) in world_payload[key]
        .as_object()
        .expect("key was not a map")
        .iter()
    {
        // TODO:
        // I'd really prefer if we could just use the original roomids instead of parsing back the
        // serialized roomids
        let mut split = roomid.split(';');
        let q = split.next().unwrap();
        let r = split.next().unwrap();

        let q = q.parse().unwrap();
        let r = r.parse().unwrap();

        out.push(RoomObjects {
            room_id: Some(Axial { q, r }),
            payload: Some(Json {
                value: serde_json::to_vec(pl).unwrap(),
            }),
        });
    }
}

impl Payload {
    /// Transform the usual json serialized world into Payload
    pub fn update(&mut self, time: u64, world_payload: &serde_json::Value) {
        self.world_time = time as i64;
        self.payload.bots.clear();
        self.payload.structures.clear();
        self.payload.resources.clear();

        write_world_payload(world_payload, "bots", &mut self.payload.bots);
        write_world_payload(world_payload, "structures", &mut self.payload.structures);
        write_world_payload(world_payload, "resources", &mut self.payload.resources);
    }
}

#[tonic::async_trait]
impl cao_world::world_server::World for WorldService {
    type EntitiesStream = ReceiverStream<Result<cao_world::WorldEntities, Status>>;

    async fn entities(
        &self,
        _r: tonic::Request<cao_world::Empty>,
    ) -> Result<tonic::Response<Self::EntitiesStream>, tonic::Status> {
        let (tx, rx) = mpsc::channel(4);

        let logger = self.logger.clone();
        let world = Arc::clone(&self.entities);
        let mut last_sent = -1;
        tokio::spawn(async move {
            let _logger = logger;
            loop {
                // TODO: maybe use the broadcast crate and broadcast new world states?
                let w = world.read().await;
                if w.world_time != last_sent {
                    if let Err(_) = tx.send(Ok(w.payload.clone())).await {
                        break;
                    }
                    last_sent = w.world_time;
                }
            }
        });

        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// test the bare minimum
    #[test]
    fn can_update_payload() {
        use caolo_sim::prelude::Executor;

        let mut pl = Payload::default();

        let mut exc = caolo_sim::prelude::SimpleExecutor;
        let mut w = exc
            .initialize(
                None,
                caolo_sim::executor::GameConfig {
                    world_radius: 2,
                    room_radius: 10,
                    ..Default::default()
                },
            )
            .unwrap();
        let logger = w.logger.clone();
        caolo_sim::init::init_world_entities(logger, &mut *w, 12);

        pl.update(w.time(), &w.hot_as_json());

        assert!(!pl.payload.bots.is_empty());
        assert!(!pl.payload.structures.is_empty());
        assert!(!pl.payload.resources.is_empty());
    }
}
