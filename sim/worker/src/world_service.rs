use std::sync::Arc;
use tokio::sync::{broadcast::Sender, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::info;

use crate::protos::cao_common;
use crate::protos::cao_world;

#[derive(Clone)]
pub struct WorldService {
    entities: WorldPayloadSender,
}

impl std::fmt::Debug for WorldService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldService").finish()
    }
}

type WorldPayloadSender = Arc<Sender<Arc<Payload>>>;

#[derive(Default, Debug)]
pub struct Payload {
    pub payload: cao_world::WorldEntities,
}

impl WorldService {
    pub fn new(entities: WorldPayloadSender) -> Self {
        Self { entities }
    }
}

fn write_world_payload(
    world_payload: &serde_json::Value,
    key: &str,
    out: &mut ::prost::alloc::vec::Vec<cao_world::RoomObjects>,
) {
    use cao_common::{Axial, Json};
    use cao_world::RoomObjects;

    let pl = world_payload[key].as_object().expect("key was not a map");

    out.reserve(pl.len());

    for (roomid, pl) in pl.iter() {
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
        self.payload.world_time = time as i64;
        self.payload.bots.clear();
        self.payload.structures.clear();
        self.payload.resources.clear();

        // TODO:
        // we could reuse these buffers?
        write_world_payload(world_payload, "bots", &mut self.payload.bots);
        write_world_payload(world_payload, "structures", &mut self.payload.structures);
        write_world_payload(world_payload, "resources", &mut self.payload.resources);
        if let Some(diag) = world_payload.get("diagnostics") {
            self.payload.diagnostics = Some(cao_common::Json {
                value: serde_json::to_vec(diag).unwrap(),
            });
        }
    }
}

#[tonic::async_trait]
impl cao_world::world_server::World for WorldService {
    type EntitiesStream = ReceiverStream<Result<cao_world::WorldEntities, Status>>;

    #[tracing::instrument]
    async fn entities(
        &self,
        _r: tonic::Request<cao_world::Empty>,
    ) -> Result<tonic::Response<Self::EntitiesStream>, tonic::Status> {
        let addr = _r.remote_addr();

        info!("Subscribing new client to world entities. Addr: {:?}", addr);

        let (tx, rx) = mpsc::channel(4);

        let mut entities_rx = self.entities.subscribe();
        let mut last_sent = -1;
        tokio::spawn(async move {
            loop {
                let w = entities_rx.recv().await.expect("world receive failed");
                if w.payload.world_time != last_sent {
                    if tx.send(Ok(w.payload.clone())).await.is_err() {
                        info!("World entities client lost {:?}", addr);
                        break;
                    }
                    last_sent = w.payload.world_time;
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
            .initialize(caolo_sim::executor::GameConfig {
                world_radius: 2,
                room_radius: 10,
                ..Default::default()
            })
            .unwrap();
        caolo_sim::init::init_world_entities(&mut *w, 12);

        pl.update(w.time(), &w.hot_as_json());

        assert!(!pl.payload.bots.is_empty());
        assert!(!pl.payload.structures.is_empty());
        assert!(!pl.payload.resources.is_empty());
    }
}
