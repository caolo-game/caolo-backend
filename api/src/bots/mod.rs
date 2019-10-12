mod bot;
mod intents;

pub use bot::*;
pub use intents::*;

use crate::rmps::{self, Serializer};
use crate::{point::Point, UserId};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bots {
    pub bots: Vec<Bot>,
}

impl Bots {
    pub fn deserialize(buffer: &[u8]) -> Result<Self, &'static str> {
        rmps::from_slice(buffer).map_err(|e| {
            println!("Failed to decode Bot {:?}", e);
            "Deserialize failed"
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(512);
        <Self as serde::Serialize>::serialize(self, &mut Serializer::new(&mut buffer)).unwrap();
        buffer
    }

    pub fn new(bots: Vec<Bot>) -> Self {
        Self { bots }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization() {
        let pos = Point::new(42, 69);

        let bot = Bot::new(52, pos, 123, None, 42, 69);

        let buffer = bot.serialize();

        let bot = Bot::deserialize(&buffer[..]).expect("Failed to deserialize");

        assert_eq!(bot.position.x, 42);
        assert_eq!(bot.position.y, 69);
        assert_eq!(bot.speed, 123);
    }

    #[test]
    fn intent_serialization() {
        let intent = MoveIntent {
            id: 23,
            position: Point::new(12, 42),
        };

        let data = intent.serialize();
        let intent = MoveIntent::deserialize(&data[..]).expect("Failed to deserialize");

        assert_eq!(intent.id, 23);
    }

    #[test]
    fn nonsense_data_is_an_err() {
        let buffer = vec![123; 16];
        Bot::deserialize(&buffer[..]).expect_err("Should be an error");
    }

    #[test]
    fn bot_list_serialization() {
        let bots = vec![
            Bot::new(1, Point::new(0, 0), 123, None, 12, 123),
            Bot::new(2, Point::new(0, 0), 123, None, 12, 123),
        ];

        let bots = Bots::new(bots);

        let data = bots.serialize();

        let read_bots = Bots::deserialize(&data).expect("Failed to deserialize");

        assert_eq!(read_bots.bots.len(), 2);
    }
}
