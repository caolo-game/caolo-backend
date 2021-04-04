use crate::input::rooms;
use crate::input::script_update;
use crate::input::structures;
use crate::input::users;
use tonic::{Request, Response, Status};

pub mod cao_common {
    tonic::include_proto!("cao_common");
}

pub mod cao_script {
    tonic::include_proto!("cao_script");
}

pub mod cao_commands {
    tonic::include_proto!("cao_commands");
}

#[derive(Clone)]
pub struct CommandService {
    logger: slog::Logger,
    world: std::sync::Arc<tokio::sync::Mutex<crate::World>>,
}

impl CommandService {
    pub fn new(
        logger: slog::Logger,
        world: std::sync::Arc<tokio::sync::Mutex<crate::World>>,
    ) -> Self {
        Self { logger, world }
    }
}

#[tonic::async_trait]
impl crate::protos::cao_commands::command_server::Command for CommandService {
    async fn place_structure(
        &self,
        request: Request<crate::protos::cao_commands::PlaceStructureCommand>,
    ) -> Result<Response<crate::protos::cao_commands::CommandResult>, Status> {
        let mut w = self.world.lock().await;
        structures::place_structure(self.logger.clone(), &mut *w, request.get_ref())
            .map(|_: ()| Response::new(crate::protos::cao_commands::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }

    async fn update_entity_script(
        &self,
        request: tonic::Request<crate::protos::cao_commands::UpdateEntityScriptCommand>,
    ) -> Result<tonic::Response<crate::protos::cao_commands::CommandResult>, tonic::Status> {
        let mut w = self.world.lock().await;
        script_update::update_entity_script(&mut *w, request.get_ref())
            .map(|_: ()| Response::new(crate::protos::cao_commands::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }

    async fn update_script(
        &self,
        request: tonic::Request<crate::protos::cao_commands::UpdateScriptCommand>,
    ) -> Result<tonic::Response<crate::protos::cao_commands::CommandResult>, tonic::Status> {
        let mut w = self.world.lock().await;
        script_update::update_program(self.logger.clone(), &mut *w, request.get_ref())
            .map(|_: ()| Response::new(crate::protos::cao_commands::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }

    async fn set_default_script(
        &self,
        request: tonic::Request<crate::protos::cao_commands::SetDefaultScriptCommand>,
    ) -> Result<tonic::Response<crate::protos::cao_commands::CommandResult>, tonic::Status> {
        let mut w = self.world.lock().await;
        script_update::set_default_script(&mut *w, request.get_ref())
            .map(|_: ()| Response::new(crate::protos::cao_commands::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }

    async fn take_room(
        &self,
        request: tonic::Request<crate::protos::cao_commands::TakeRoomCommand>,
    ) -> Result<tonic::Response<crate::protos::cao_commands::CommandResult>, tonic::Status> {
        let mut w = self.world.lock().await;
        rooms::take_room(self.logger.clone(), &mut *w, request.get_ref())
            .map(|_: ()| Response::new(crate::protos::cao_commands::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }

    async fn register_user(
        &self,
        request: tonic::Request<crate::protos::cao_commands::RegisterUserCommand>,
    ) -> Result<tonic::Response<cao_commands::CommandResult>, tonic::Status> {
        let mut w = self.world.lock().await;
        users::register_user(self.logger.clone(), &mut *w, request.get_ref())
            .map(|_: ()| Response::new(crate::protos::cao_commands::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }
}
