from typing import Dict, List, Tuple
from fastapi import APIRouter, Query, Request
import json

from ..api_schema import RoomObjects, Axial, make_room_id, parse_room_id
from ..model.game_state import load_latest_game_state, get_room_objects


router = APIRouter(prefix="/world")


@router.get("/terrain", response_model=List[Tuple[Axial, str]])
async def terrain(
    req: Request,
    q: str = Query(None, max_length=5),
    r: str = Query(None, max_length=5),
):
    room_id = make_room_id(q, r)

    res_encoded = await req.state.db.fetchval(
        """
        SELECT t.payload->'terrain'->$1 AS room
        FROM public.world_output t
        ORDER BY t.created DESC
        """,
        room_id,
    )

    # returned data is already json encoded string
    # TODO: just write the already encoded response...
    if not res_encoded:
        return []
    return json.loads(res_encoded)


@router.get("/rooms", response_model=List[Dict])
async def rooms(req: Request):
    res = await req.state.db.fetchrow(
        """
        SELECT t.payload->'rooms' AS rooms
        FROM public.world_output t
        ORDER BY t.created DESC
        """
    )

    assert res, "No results returned"

    res_encoded = res["rooms"]
    data: Dict[str, Dict] = json.loads(res_encoded)
    # keys are 'q;r', so split them and insert them into a 'pos' object, then put the rest of the values next to it
    return (
        {"pos": room_id, **v}
        for room_id, v in ((parse_room_id(k), v) for k, v in data.items())
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

    # Turns out that loading the entire game state into memory,
    # parsing the json and filtering it is about 3x faster than doing this filtering in Postgres
    pl = await load_latest_game_state(req.state.db)
    return get_room_objects(pl, room_id)


@router.get("/diagnostics", response_model=Dict)
async def diagnostics(req: Request):
    """
    returns internal engine diagnostics
    """

    row = await req.state.db.fetchrow(
        """
        SELECT
            t.payload->'diagnostics' as pl
            , t.world_time as time
        FROM public.world_output t
        ORDER BY t.created DESC
        """,
    )

    if not row:
        return None
    pl = None
    try:
        pl = json.loads(row["pl"])
        pl["tick"] = row.get("time", -1)
    except KeyError:
        pass
    return pl
