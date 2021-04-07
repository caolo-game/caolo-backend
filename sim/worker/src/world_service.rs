mod ser_bots;
mod ser_resources;
mod ser_structures;

use caolo_sim::prelude::World;
use std::sync::Arc;
use tokio::sync::{broadcast::Sender, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::info;

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

impl Payload {
    /// Transform the usual json serialized world into Payload
    pub fn update(&mut self, time: u64, world: &World) {
        self.payload.world_time = time as i64;
        self.payload.bots.clear();
        self.payload.structures.clear();
        self.payload.resources.clear();

        ser_bots::bot_payload(
            &mut self.payload.bots,
            caolo_sim::prelude::FromWorld::new(world),
        );
        ser_structures::structure_payload(
            &mut self.payload.structures,
            caolo_sim::prelude::FromWorld::new(world),
        );
        ser_resources::resource_payload(
            &mut self.payload.resources,
            caolo_sim::prelude::FromWorld::new(world),
        );

        if let Some(diag) = world
            .view::<caolo_sim::prelude::EmptyKey, caolo_sim::diagnostics::Diagnostics>()
            .value
            .as_ref()
        {
            self.payload.diagnostics = Some(cao_world::Diagnostics {
                current: Some(cao_world::diagnostics::Current {
                    number_of_intents: diag.number_of_intents,
                    number_of_scripts_ran: diag.number_of_scripts_ran,
                    number_of_scripts_errored: diag.number_of_scripts_errored,
                    systems_update_ms: diag.systems_update_ms,
                    scripts_execution_ms: diag.scripts_execution_ms,
                    tick: diag.tick,
                    tick_latency_ms: diag.tick_latency_ms,
                }),
                accumulated: Some(cao_world::diagnostics::Accumulated {
                    tick_latency_count: diag.tick_latency_count.into(),
                    tick_latency_max: diag.tick_latency_max.into(),
                    tick_latency_min: diag.tick_latency_min.into(),
                    tick_latency_mean: diag.tick_latency_mean,
                    tick_latency_std: diag.tick_latency_std,
                }),
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
