from typing import Dict, List, Tuple
from fastapi import (
    APIRouter,
    Response,
    Query,
    Request,
    WebSocket,
    Depends,
    WebSocketDisconnect,
)
from fastapi.middleware.cors import CORSMiddleware
import asyncpg
import json
import os
import asyncio
import datetime as dt

from dataclasses import dataclass

from ..api_schema import RoomObjects, Axial, make_room_id, parse_room_id

router = APIRouter(prefix="/world")


@dataclass
class WorldClient:
    ws: WebSocket
    room_id: str = None


async def load_game_state(db):

    row = await db.fetchrow(
        """
        SELECT t.payload AS pl, t.world_time as time
        FROM public.world_output t
        ORDER BY t.created DESC
        """,
    )

    state = json.loads(row["pl"])
    state["time"] = row["time"]
    return state


class WorldMessenger:
    def __init__(self):
        self.connections = []
        self.game_state = None
        self.running = False

    async def connect(self, ws: WorldClient):
        self.connections.append(ws)

    async def disconnect(self, ws: WorldClient):
        self.connections.remove(ws)

    async def _onsendfinish(self, state):
        self.last_sent = state["time"]
        # figure out how much we need to sleep
        # formula: expected_next - now
        # expected_next = end of last tick + target latency + actual latency for bias
        end = state["diagnostics"]["tick_end"]
        latency = state["diagnostics"]["tick_latency_ms"]
        target_lat = state["gameConfig"]["target_tick_ms"]
        expected_next = dt.datetime.strptime(
            end[:-4], "%Y-%m-%dT%H:%M:%S.%f"
        ) + dt.timedelta(milliseconds=target_lat + latency)
        now = dt.datetime.utcnow()
        delta = expected_next - now
        print(f"sleeping for {delta.total_seconds()} seconds")
        await asyncio.sleep(max(delta.total_seconds(), 0.01))

    async def send_to(self, client):
        state = self.game_state
        pl = RoomObjects()
        pl.payload.bots = state["bots"].get(client.room_id, [])
        pl.payload.structures = state["structures"].get(client.room_id, [])
        pl.payload.resources = state["resources"].get(client.room_id, [])
        pl.time = state["time"]

        pl = json.dumps(pl, default=lambda o: o.__dict__)
        await client.ws.send_text(pl)

    async def run(self, pool):
        assert not self.running
        self.running = True
        self.last_sent = -1
        try:
            while 1:
                async with pool.acquire() as con:
                    state = await load_game_state(con)
                    self.game_state = state

                if state["time"] == self.last_sent:
                    await self._onsendfinish(state)
                    continue

                dc = []
                for client in self.connections:
                    try:
                        await self.send_to(client)
                    except WebSocketDisconnect:
                        dc.append(client)
                # disconnected clients
                for c in dc:
                    await self.disconnect(c)
                await self._onsendfinish(state)
                yield
        finally:
            self.running = False


manager = WorldMessenger()


async def get_messenger():
    return manager


@router.websocket("/object-stream")
async def object_stream(ws: WebSocket, manager=Depends(get_messenger)):
    """
    send in the room_id in the form 'q;r'
    """
    await ws.accept()
    client = WorldClient(ws=ws, room_id=None)
    await manager.connect(client)
    try:
        while 1:
            room_id = await ws.receive_text()
            client.room_id = room_id
            # on new room_id send a state immediately
            await manager.send_to(client)
    except:
        pass
    finally:
        await manager.disconnect(client)
