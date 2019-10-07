use crate::rmps::{self, Serializer};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub script: Option<Vec<u8>>,
    pub compiled: Option<Vec<u8>>,
}

impl std::fmt::Debug for UserData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "UserData")
    }
}

impl UserData {
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

    pub fn new(script: Option<Vec<u8>>, compiled: Option<Vec<u8>>) -> Self {
        Self { script, compiled }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization() {
        let userdata = UserData::new(Some(vec![42, 96, 69]), None);

        let data = userdata.serialize();
        let userdata = UserData::deserialize(&data[..]).expect("Failed to deserialize");

        assert_eq!(userdata.script, Some(vec![42, 96, 69]));
    }
}
