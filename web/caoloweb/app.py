from typing import Dict, List
from fastapi import FastAPI, Response, Query, Request
from fastapi.middleware.cors import CORSMiddleware
import asyncpg
import json
import os


from .api_schema import RoomObjects

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


@app.get("/terrain", response_model=List[Dict])
async def terrain(
    req: Request, q: str = Query(None, max_length=5), r: str = Query(None, max_length=5)
):
    room_id = f"{q};{r}"

    res_encoded = await req.state.db.fetchval(
        """
SELECT objects.value AS room
FROM public.world_output t, jsonb_each(t.payload->'terrain') objects
WHERE objects.key::text = $1
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

    room_id = f"{q};{r}"
    res = await req.state.db.fetchrow(
        """
SELECT botobj.value AS bots
    , structobj.value as structures
    , resobj.value as resources
    , t.world_time as time
FROM public.world_output t
    , jsonb_each(t.payload->'bots') botobj
    , jsonb_each(t.payload->'structures') structobj
    , jsonb_each(t.payload->'resources') resobj
WHERE botobj.key::text = $1
    AND structobj.key::text = $1
    AND resobj.key::text = $1
ORDER BY t.created DESC
        """,
        room_id,
    )
    payload = RoomObjects()
    if not res:
        return payload

    payload.payload = {
        "bots": json.loads(res["bots"]),
        "structures": json.loads(res["structures"]),
        "resources": json.loads(res["resources"]),
    }
    payload.time = res["time"]

    return payload
