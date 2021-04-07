from typing import Dict, List, Tuple

from fastapi import APIRouter, Query

from ..api_schema import RoomObjects, make_room_id, parse_room_id
from ..model.game_state import get_room_objects, manager


router = APIRouter(prefix="/world", tags=["world"])


@router.get("/room-terrain-layout", response_model=List[Tuple[int, int]])
async def room_terrain_layout():
    """
    return the coordinates of the room grid points in a list.

    If you query the terrain the i-th terrain enum value
    will correspond to the i-th coordinates returned by this endpoint
    """
    return manager.game_state.properties["terrain"]["roomLayout"]


@router.get("/terrain", response_model=List[str])
async def terrain(q: int = Query(None), r: int = Query(None)):
    room_id = make_room_id(q, r)
    return manager.game_state.properties["terrain"]["roomTerrain"].get(room_id)


@router.get("/rooms", response_model=List[Dict])
async def rooms():
    # keys are 'q;r', so split them and insert them into a 'pos' object,
    # then put the rest of the values next to it
    return (
        {"pos": [room_id.q, room_id.r], **v}
        for room_id, v in (
            (parse_room_id(k), v)
            for k, v in manager.game_state.properties["rooms"].items()
        )
    )


@router.get("/room-objects", response_model=RoomObjects)
async def room_objects(q: int = Query(None), r: int = Query(None)):
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
