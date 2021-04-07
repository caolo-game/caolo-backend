from typing import Dict, List, Tuple

from fastapi import APIRouter, Query, Depends, HTTPException, status

import grpc
import cao_world_pb2
import cao_world_pb2_grpc
import cao_common_pb2
from google.protobuf.json_format import MessageToDict

from ..api_schema import RoomObjects, make_room_id, parse_room_id
from ..model.game_state import get_room_objects, manager, get_terrain
from ..queen import queen_channel


router = APIRouter(prefix="/world", tags=["world"])


@router.get("/room-terrain-layout", response_model=List[Tuple[int, int]])
async def room_terrain_layout():
    """
    return the coordinates of the room grid points in a list.

    If you query the terrain the i-th terrain enum value
    will correspond to the i-th coordinates returned by this endpoint
    """
    return list(
        map(
            lambda p: (int(p["q"]), int(p["r"])),
            manager.game_state.room_layout,
        )
    )


@router.get("/terrain", response_model=List[str])
async def terrain(q: int = Query(...), r: int = Query(...)):
    return await get_terrain(q, r)


@router.get("/rooms", response_model=List[Dict])
async def rooms():
    return manager.game_state.rooms


@router.get("/room-objects", response_model=RoomObjects)
async def room_objects(q: int = Query(...), r: int = Query(...)):
    """
    return a list of each type of entity in the given room
    """
    room_id = make_room_id(q, r)
    return get_room_objects(manager.game_state, room_id)


@router.get("/diagnostics", response_model=Dict)
async def diagnostics():
    """
    returns internal engine diagnostics
    """
    return manager.game_state.entities.get("diagnostics")
