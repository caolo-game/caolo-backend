from typing import Dict, List, Tuple
from fastapi import FastAPI, Response, Query, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
import asyncpg

import json
import os
import asyncio
import aioredis


from .api_schema import RoomObjects, Axial, make_room_id, parse_room_id
from .model import game_state

from .handler import world, scripting, admin, world_ws, commands

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
app.add_middleware(GZipMiddleware, minimum_size=1000)

DB_STR = os.getenv("DATABASE_URL", "postgres://postgres:admin@localhost:5432/caolo")
_DB_POOL = None


async def db_pool():
    global _DB_POOL
    if not _DB_POOL:
        _DB_POOL = await asyncpg.create_pool(DB_STR)
    return _DB_POOL


REDIS_STR = os.getenv("REDIS_URL", "redis://localhost:6379/0")
_REDIS_POOL = None


async def redis_pool():
    global _REDIS_POOL
    if not _REDIS_POOL:
        _REDIS_POOL = await aioredis.create_pool(REDIS_STR)
    return _REDIS_POOL


@app.middleware("http")
async def db_session(req, call_next):
    resp = Response(status_code=500)
    pool = await db_pool()
    async with pool.acquire() as con:
        req.state.db = con
        resp = await call_next(req)
    return resp


@app.middleware("http")
async def redis_session(req, call_next):
    resp = Response(status_code=500)
    pool = await redis_pool()
    cache = await pool.acquire()
    req.state.cache = aioredis.Redis(cache)
    try:
        resp = await call_next(req)
    finally:
        pool.release(cache)
    return resp


@app.middleware("http")
async def rate_limit(req, call_next):
    pool = await redis_pool()
    cache = await pool.acquire()
    redis = aioredis.Redis(cache)
    try:
        tr = redis.multi_exec()

        host = req.client.host
        key = f"cao-access-{host}"
        tr.setnx(key, 0)
        tr.incr(key)
        tr.expire(key, 1)

        res = await tr.execute()
        res = res[1]

        if res > 50:
            return Response(status_code=429)
    finally:
        pool.release(cache)

    return await call_next(req)


@app.get("/health")
async def health():
    return Response(status_code=204)


app.include_router(world.router)
app.include_router(scripting.router)
app.include_router(admin.router)
app.include_router(world_ws.router)
app.include_router(commands.router)


async def _broadcast_gamestate():
    pool = await db_pool()

    r = game_state.manager.run(pool)
    while 1:
        await r.__anext__()


@app.on_event("startup")
async def on_start():
    asyncio.create_task(_broadcast_gamestate())
    # force connections on startup instead of at the first request
    await redis_pool()
    await db_pool()
