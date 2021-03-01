from typing import Dict, List, Tuple
from fastapi import FastAPI, Response, Query, Request
from fastapi.middleware.cors import CORSMiddleware
import asyncpg

import json
import os
import asyncio


from .api_schema import RoomObjects, Axial, make_room_id, parse_room_id
from .model import game_state

from .handler import world, scripting, admin, world_ws

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

DB_STR = os.getenv("DATABASE_URL", "postgres://postgres:admin@localhost:5432/caolo")
_DB_POOL = None


async def db_pool():
    global _DB_POOL
    if not _DB_POOL:
        _DB_POOL = await asyncpg.create_pool(DB_STR)
    return _DB_POOL


@app.middleware("http")
async def db_session(req, call_next):
    resp = Response(status_code=500)
    pool = await db_pool()
    async with pool.acquire() as con:
        req.state.db = con
        resp = await call_next(req)
    return resp


@app.get("/health")
async def health():
    return Response(status_code=204)


app.include_router(world.router)
app.include_router(scripting.router)
app.include_router(admin.router)
app.include_router(world_ws.router)


async def _broadcast_gamestate():
    pool = await db_pool()

    r = game_state.manager.run(pool)
    while 1:
        await r.__anext__()


@app.on_event("startup")
async def on_start():
    asyncio.create_task(_broadcast_gamestate())
