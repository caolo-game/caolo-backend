use crate::input::structures;
use crate::input::users;
use crate::{input::rooms, protos::cao_commands};
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct CommandService {
    world: std::sync::Arc<tokio::sync::Mutex<crate::World>>,
}

impl std::fmt::Debug for CommandService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandService").finish()
    }
}

impl CommandService {
    pub fn new(world: std::sync::Arc<tokio::sync::Mutex<crate::World>>) -> Self {
        Self { world }
    }
}

#[tonic::async_trait]
impl cao_commands::command_server::Command for CommandService {
    #[tracing::instrument]
    async fn place_structure(
        &self,
        request: Request<cao_commands::PlaceStructureCommand>,
    ) -> Result<Response<cao_commands::CommandResult>, Status> {
        let mut w = self.world.lock().await;
        structures::place_structure(&mut *w, request.get_ref())
            .map(|_: ()| Response::new(cao_commands::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }


    #[tracing::instrument]
    async fn take_room(
        &self,
        request: tonic::Request<cao_commands::TakeRoomCommand>,
    ) -> Result<tonic::Response<cao_commands::CommandResult>, tonic::Status> {
        let mut w = self.world.lock().await;
        rooms::take_room(&mut *w, request.get_ref())
            .map(|_: ()| Response::new(cao_commands::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }

    #[tracing::instrument]
    async fn register_user(
        &self,
        request: tonic::Request<cao_commands::RegisterUserCommand>,
    ) -> Result<tonic::Response<cao_commands::CommandResult>, tonic::Status> {
        let mut w = self.world.lock().await;
        users::register_user(&mut *w, request.get_ref())
            .map(|_: ()| Response::new(cao_commands::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }
}
