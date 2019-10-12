use super::*;
use crate::model;
use crate::profile;
use crate::systems::pathfinding::find_path;
use caolo_api::point::Point;

const MAX_STEPS: u32 = 64;
