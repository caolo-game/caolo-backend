from typing import Dict, List, Tuple
from fastapi import FastAPI, Response, Query, Request
from fastapi.middleware.cors import CORSMiddleware
import asyncpg
import json
import os


from .api_schema import RoomObjects, Axial, make_room_id

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

DB_STR = os.getenv("DATABASE_URL", "postgres://postgres:admin@localhost:5432/caolo")


@app.middleware("http")
async def db_session(req, call_next):
    resp = Response(status_code=500)
    dbconn = await asyncpg.connect(DB_STR)
    try:
        req.state.db = dbconn
        resp = await call_next(req)
    finally:
        await req.state.db.close()
    return resp


@app.get("/health")
async def health():
    return Response(status_code=204)


@app.get("/terrain", response_model=List[Tuple[Axial, str]])
async def terrain(
    req: Request, q: str = Query(None, max_length=5), r: str = Query(None, max_length=5)
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


@app.get("/rooms", response_model=List[Dict])
async def rooms(req: Request):
    res = await req.state.db.fetch(
        """
SELECT t.payload->'rooms' AS rooms
FROM public.world_output t
ORDER BY t.created DESC
LIMIT 1
        """
    )

    assert res, "No results returned"

    res_encoded = res[0]["rooms"]
    data: Dict[str, Dict] = json.loads(res_encoded)
    # keys are 'q;r', so split them and insert them into a 'pos' object, then put the rest of the values next to it
    return (
        {"pos": {"q": q, "r": r}, **v}
        for q, r, v in ((*k.split(";"), v) for k, v in data.items())
    )


@app.get("/room-objects", response_model=RoomObjects)
async def room_objects(
    req: Request, q: str = Query(None, max_length=5), r: str = Query(None, max_length=5)
):
    """
    return a list of each type of entity in the given room
    """

    room_id = make_room_id(q, r)

    # Turns out that loading the entire game state into memory,
    # parsing the json and filtering it is about 3x faster than doing this filtering in Postgres
    res = await req.state.db.fetchrow(
        """
SELECT
    t.payload as payload
    , t.world_time as time
FROM public.world_output t
ORDER BY t.created DESC
LIMIT 1
        """,
    )
    payload = RoomObjects()

    try:
        pl = json.loads(res["payload"])
        payload.time = res["time"]
        payload.payload = {
            "bots": pl["bots"][room_id],
            "structures": pl["structures"][room_id],
            "resources": pl["resources"][room_id],
        }
    except KeyError:
        pass
    return payload
