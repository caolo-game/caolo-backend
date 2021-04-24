use crate::input::script_update;
use crate::protos::cao_common;
use crate::protos::cao_script;
use caolo_sim::{components::CaoIrComponent, indices::ScriptId};
use std::convert::TryInto;
use tonic::{Response, Status};
use tracing::debug;

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
    async fn list_scripts(
        &self,
        _request: tonic::Request<cao_script::Empty>,
    ) -> Result<tonic::Response<cao_script::ScriptList>, tonic::Status> {
        let ids;
        {
            let w = self.world.lock().await;
            let scripts_table = w.view::<ScriptId, CaoIrComponent>();
            ids = scripts_table.iter().map(|(id, _)| id.0).collect::<Vec<_>>();
            // free the world lock asap
        }
        let payload = cao_script::ScriptList {
            script_ids: ids
                .into_iter()
                .map(|id: uuid::Uuid| cao_common::Uuid {
                    data: id.as_bytes().to_vec(),
                })
                .collect(),
        };
        Ok(Response::new(payload))
    }

    async fn get_script(
        &self,
        request: tonic::Request<cao_common::Uuid>,
    ) -> Result<tonic::Response<cao_script::CompilationUnit>, tonic::Status> {
        let msg: &cao_common::Uuid = request.get_ref();
        let id = msg.data.as_slice();
        let id = uuid::Uuid::from_slice(id).map_err(|err| {
            debug!("Failed to parse uuid {:?}", err);
            tonic::Status::invalid_argument("Script id is malformed, expected UUID")
        })?;
        let payload;
        {
            let w = self.world.lock().await;
            let scripts_table = w.view::<ScriptId, CaoIrComponent>();
            let ir = &scripts_table
                .get_by_id(ScriptId(id))
                .ok_or_else(|| tonic::Status::not_found("Script not found"))?
                .0;
            payload = serde_json::to_vec(ir).unwrap();
        }

        let payload = cao_script::CompilationUnit {
            encoded: Some(cao_common::Json { value: payload }),
        };

        Ok(tonic::Response::new(payload))
    }

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

    async fn get_schema(
        &self,
        _: tonic::Request<cao_script::Empty>,
    ) -> Result<tonic::Response<cao_script::Schema>, tonic::Status> {
        use cao_script::SchemaCard;

        let schema = caolo_sim::scripting_api::make_import();
        let imports = schema.imports();
        let basic_descs = cao_lang::compiler::card_description::get_instruction_descriptions();

        // TODO: allocator prime candidate for the allocator interface feature

        let cards = imports
            .iter()
            .map(|card| SchemaCard {
                ty: "Call".to_string(),
                properties: card.desc.properties.iter().map(|x| x.to_string()).collect(),
                outputs: card.desc.output.iter().map(|x| x.to_string()).collect(),
                inputs: card.desc.input.iter().map(|x| x.to_string()).collect(),
                name: card.desc.name.to_string(),
                description: card.desc.description.to_string(),
            })
            .chain(basic_descs.iter().map(|card| SchemaCard {
                properties: card.properties.iter().map(|x| x.to_string()).collect(),
                outputs: card.output.iter().map(|x| x.to_string()).collect(),
                inputs: card.input.iter().map(|x| x.to_string()).collect(),
                name: card.name.to_string(),
                description: card.description.to_string(),
                ty: card.ty.as_str().to_string(),
            }))
            .collect();

        let schema = cao_script::Schema { cards };

        Ok(tonic::Response::new(schema))
    }
}
