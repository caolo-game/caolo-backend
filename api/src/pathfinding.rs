use crate::point::Point;

/// Holds a path find result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub path: Vec<Point>,
}
