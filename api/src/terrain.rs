#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[repr(u8)]
pub enum TileTerrainType {
    Empty = 0,
    Wall,
}

impl Default for TileTerrainType {
    fn default() -> Self {
        TileTerrainType::Empty
    }
}
