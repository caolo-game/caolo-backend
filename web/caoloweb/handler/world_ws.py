from fastapi import (
    APIRouter,
    WebSocket,
    Depends,
    WebSocketDisconnect,
)
import json
import logging
import asyncio
import datetime as dt

from dataclasses import dataclass

from ..api_schema import RoomObjects, Axial, make_room_id, parse_room_id
from ..model.game_state import load_latest_game_state, get_room_objects

router = APIRouter(prefix="/world")


@dataclass
class WorldClient:
    ws: WebSocket
    room_id: str = None
    last_seen: int = -1


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
        self.last_sent = state.created
        # figure out how much we need to sleep
        # formula: expected_next - now
        # expected_next = end of last tick + target latency + actual latency for bias
        try:
            end = state.payload["diagnostics"]["tick_end"]
            end = dt.datetime.strptime(end[:-4], "%Y-%m-%dT%H:%M:%S.%f")
        except Exception as err:
            logging.warn(f"Failed to parse end timestamp {err}")
            end = dt.datetime.now()
        latency = state.payload["diagnostics"]["tick_latency_ms"]
        target_lat = state.payload["gameConfig"]["target_tick_ms"]
        expected_next = end + dt.timedelta(milliseconds=target_lat + latency)
        now = dt.datetime.utcnow()
        delta = expected_next - now
        logging.debug(f"sleeping for {delta.total_seconds()} seconds")
        await asyncio.sleep(max(delta.total_seconds(), 0.01))

    async def send_to(self, client):
        state = self.game_state
        client.last_seen = state.created
        pl = get_room_objects(state, client.room_id)
        pl = json.dumps(pl, default=lambda o: o.__dict__)
        await client.ws.send_text(pl)

    async def run(self, pool):
        assert not self.running
        self.running = True
        self.last_sent = -1
        try:
            while 1:
                async with pool.acquire() as con:
                    state = await load_latest_game_state(con)
                    self.game_state = state

                if state.created == self.last_sent:
                    await self._onsendfinish(state)
                    continue

                dc = []
                for client in self.connections:
                    try:
                        if state.created != client.last_seen:
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


@router.websocket("/object-stream")
async def object_stream(ws: WebSocket, manager=Depends(lambda: manager)):
    """
    Streams game objects of a room.

    Send in the room_id in the form 'q;r' to receive objects of the given room
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
    except WebSocketDisconnect:
        pass
    finally:
        await manager.disconnect(client)
