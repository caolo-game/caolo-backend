//! Actions, world updates the clients _intented_ to be executed.
//!
mod dropoff_intent;
mod log_intent;
mod mine_intent;
mod move_intent;
mod spawn_intent;

pub use self::dropoff_intent::*;
pub use self::log_intent::*;
pub use self::mine_intent::*;
pub use self::move_intent::*;
pub use self::spawn_intent::*;

impl Intents {
    pub fn new() -> Self {
        Self::default()
    }
}

macro_rules! intents {
    ($($name: ident: $type: ty),*) =>{
        #[derive(Debug, Clone, Default)]
        pub struct Intents {
            $(pub $name: Vec<$type>),*
        }
        impl Intents {
            pub fn merge(&mut self, other: Intents) -> &mut Self {
                $(self.$name.extend_from_slice(&other.$name));* ;
                self
            }
        }
    };
}

intents!(
    move_intents: MoveIntent,
    spawn_intents: SpawnIntent,
    mine_intents: MineIntent,
    dropoff_intents: DropoffIntent,
    log_intents: LogIntent
);
