use crate::input::script_update;
use crate::protos::cao_common;
use crate::protos::cao_script;
use std::convert::TryInto;
use tonic::{Response, Status};

#[derive(Clone)]
pub struct ScriptingService {
    world: std::sync::Arc<tokio::sync::Mutex<crate::World>>,
}

impl std::fmt::Debug for ScriptingService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptingService").finish()
    }
}

impl ScriptingService {
    pub fn new(world: std::sync::Arc<tokio::sync::Mutex<crate::World>>) -> Self {
        Self { world }
    }
}

#[tonic::async_trait]
impl cao_script::scripting_server::Scripting for ScriptingService {
    async fn update_entity_script(
        &self,
        request: tonic::Request<cao_script::UpdateEntityScriptCommand>,
    ) -> Result<tonic::Response<cao_script::CommandResult>, tonic::Status> {
        let mut w = self.world.lock().await;
        script_update::update_entity_script(&mut *w, request.get_ref())
            .map(|_: ()| Response::new(cao_script::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }

    async fn update_script(
        &self,
        request: tonic::Request<cao_script::UpdateScriptCommand>,
    ) -> Result<tonic::Response<cao_script::CommandResult>, tonic::Status> {
        let mut w = self.world.lock().await;
        script_update::update_program(&mut *w, request.get_ref())
            .map(|_: ()| Response::new(cao_script::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }

    async fn set_default_script(
        &self,
        request: tonic::Request<cao_script::SetDefaultScriptCommand>,
    ) -> Result<tonic::Response<cao_script::CommandResult>, tonic::Status> {
        let mut w = self.world.lock().await;
        script_update::set_default_script(&mut *w, request.get_ref())
            .map(|_: ()| Response::new(cao_script::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }

    async fn get_bot_script_id(
        &self,
        request: tonic::Request<cao_script::EntityId>,
    ) -> Result<tonic::Response<cao_common::Uuid>, tonic::Status> {
        let w = self.world.lock().await;
        match w
            .view::<caolo_sim::prelude::EntityId, caolo_sim::prelude::EntityScript>()
            .get_by_id(caolo_sim::prelude::EntityId(
                request
                    .get_ref()
                    .id
                    .try_into()
                    .map_err(|_| tonic::Status::invalid_argument("invalid entity id"))?,
            )) {
            Some(id) => Ok(tonic::Response::new(cao_common::Uuid {
                data: (id.0).0.as_bytes().to_vec(),
            })),
            None => Err(tonic::Status::not_found(
                "Entity not found or has no script",
            )),
        }
    }
}
