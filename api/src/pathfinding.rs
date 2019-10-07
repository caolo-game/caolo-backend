use crate::external;
use crate::point::Point;
use crate::rmps::{self, Serializer};
use crate::OperationResult;

/// Holds a path find result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub path: Vec<Point>,
}

impl Path {
    pub fn deserialize(buffer: &[u8]) -> Result<Self, &'static str> {
        rmps::from_slice(buffer).map_err(|e| {
            println!("Failed to decode Bot {:?}", e);
            "Deserialize failed"
        })
    }

    pub fn serialize(self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(512);
        <Self as serde::Serialize>::serialize(&self, &mut Serializer::new(&mut buffer)).unwrap();
        buffer
    }
}

/// Look for a path between point `from` and `to`
/// The returned path is in the interval (from, to]
pub fn find_path(from: Point, to: Point) -> Result<Path, OperationResult> {
    let max_len = unsafe { external::_get_max_path_length() as usize };

    let mut buff = vec![0; max_len * std::mem::size_of::<Point>() + 4];

    let result = unsafe { external::_find_path(from.x, from.y, to.x, to.y, buff.as_mut_ptr()) };

    if result < 0 {
        let result = OperationResult::from(result);
        return Err(result);
    }
    let len = result;
    if len > 0 {
        let path = Path::deserialize(&buff[0..len as usize]).unwrap();
        Ok(path)
    } else {
        Ok(Path { path: vec![] })
    }
}
