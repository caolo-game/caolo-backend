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
from ..model.game_state import manager as game_state_manager, get_room_objects

router = APIRouter()


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
        try:
            self.connections.remove(ws)
        except ValueError:
            pass

    async def send_to(self, client):
        state = self.game_state
        client.last_seen = state.created
        pl = get_room_objects(state, client.room_id)
        pl = json.dumps(pl, default=lambda o: o.__dict__)
        await client.ws.send_text(pl)

    def on_new_state(self, state):
        self.game_state = state
        asyncio.create_task(_broadcast_gamestate(self))

    async def run(self):
        if self.running:
            return
        assert not self.running
        self.running = True
        self.last_sent = -1
        try:
            while 1:
                if self.game_state.created == self.last_sent:
                    continue

                dc = []
                for client in self.connections:
                    try:
                        if self.game_state.created != client.last_seen:
                            await self.send_to(client)
                    except WebSocketDisconnect:
                        dc.append(client)
                    except Exception as exc:
                        logging.exception("Sending game state failed")
                        dc.append(client)
                # disconnected clients
                for c in dc:
                    await self.disconnect(c)
                yield
        finally:
            self.running = False


manager = WorldMessenger()


game_state_manager.on_new_state(manager.on_new_state)


async def _broadcast_gamestate(manager):
    r = manager.run()
    await r.__anext__()


# NOTE:
# the router.websocket ignores the router's path prefix
@router.websocket("/world/object-stream")
async def object_stream(ws: WebSocket, manager=Depends(lambda: manager)):
    """
    Streams game objects of a room.

    Send in the room_id in the form 'q;r' to receive objects of the given room
    """
    logging.info("Client is attempting to connect to object stream")
    await ws.accept()
    client = WorldClient(ws=ws, room_id=None)
    await manager.connect(client)
    logging.info("Client connected to object stream")
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
