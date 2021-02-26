from typing import Dict, List, Tuple
from fastapi import APIRouter, Query, Request, Response
import json

from ..api_schema import RoomObjects, Axial, make_room_id, parse_room_id
from ..model.game_state import get_room_objects, manager


router = APIRouter(prefix="/world")


@router.get("/terrain", response_model=List[Tuple[Axial, str]])
async def terrain(
    req: Request,
    q: str = Query(None, max_length=5),
    r: str = Query(None, max_length=5),
):
    room_id = make_room_id(q, r)
    return manager.game_state.payload["terrain"].get(room_id)


@router.get("/rooms", response_model=List[Dict])
async def rooms(req: Request):
    # keys are 'q;r', so split them and insert them into a 'pos' object, then put the rest of the values next to it
    return (
        {"pos": room_id, **v}
        for room_id, v in (
            (parse_room_id(k), v)
            for k, v in manager.game_state.payload["rooms"].items()
        )
    )


@router.get("/room-objects", response_model=RoomObjects)
async def room_objects(
    req: Request,
    q: str = Query(None, max_length=5),
    r: str = Query(None, max_length=5),
):
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
    return manager.game_state.payload["diagnostics"]
