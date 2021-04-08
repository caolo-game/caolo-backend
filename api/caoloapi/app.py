from typing import Dict, List, Tuple
from fastapi import FastAPI, Response, Query, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
import asyncpg

import logging
import json
import os
import asyncio
import sys

from .config import QUEEN_TAG, DB_URL, QUEEN_URL

from .api_schema import RoomObjects, Axial, make_room_id, parse_room_id
from .model import game_state

from . import handler


logging.basicConfig(
    level=logging.DEBUG,
    #  level=logging.WARNING,
    format="%(asctime)s [%(levelname)s] %(pathname)s:%(lineno)d: %(message)s",
)


tags_metadata = [
    {"name": "world", "description": "game world related stuff"},
    {"name": "scripting", "description": "Cao-Lang related stuff"},
    {"name": "commands", "description": "Simulation interaction"},
    {"name": "users", "description": "User management"},
]

app = FastAPI(title="Cao-Lo API", version="0.1.0", openapi_tags=tags_metadata)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
app.add_middleware(GZipMiddleware, minimum_size=1000)

_DB_POOL = None


async def db_pool():
    global _DB_POOL
    if not _DB_POOL:
        _DB_POOL = await asyncpg.create_pool(DB_URL)
    return _DB_POOL


@app.middleware("http")
async def db_session(req, call_next):
    resp = Response(status_code=500)
    pool = await db_pool()
    async with pool.acquire() as con:
        req.state.db = con
        resp = await call_next(req)
    return resp


@app.middleware("http")
async def rate_limit(req, call_next):
    # TODO
    return await call_next(req)


@app.get("/health")
async def health():
    return Response(status_code=204)


app.include_router(handler.world.router)
app.include_router(handler.scripting.router)
app.include_router(handler.admin.router)
app.include_router(handler.world_ws.router)
app.include_router(handler.commands.router)
app.include_router(handler.users.router)


async def _broadcast_gamestate():
    pool = await db_pool()
    async with pool.acquire() as con:
        await game_state.manager.start(QUEEN_TAG, QUEEN_URL, con)


@app.on_event("startup")
async def on_start():
    # force connections on startup instead of at the first request
    await db_pool()
    await _broadcast_gamestate()
