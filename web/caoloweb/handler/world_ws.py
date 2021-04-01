import json
import logging
import asyncio
from dataclasses import dataclass

from fastapi import (
    APIRouter,
    WebSocket,
    Depends,
    WebSocketDisconnect,
)

from ..model.game_state import (
    manager as game_state_manager,
    get_room_objects,
    get_terrain,
)

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

    async def send_terrain(self, client):
        state = self.game_state or game_state_manager.game_state
        if not state:
            logging.error("No GameState is available")
            return
        terrain = get_terrain(state, client.room_id)
        pl = {"terrain": terrain, "ty": "terrain"}
        pl = json.dumps(pl, default=lambda o: o.__dict__)
        await client.ws.send_text(pl)

    async def send_entities(self, client):
        state = self.game_state or game_state_manager.game_state
        if not state:
            logging.error("No GameState is available")
            return
        client.last_seen = state.created
        entities = get_room_objects(state, client.room_id)
        pl = {"entities": entities, "ty": "entities"}
        pl = json.dumps(pl, default=lambda o: o.__dict__)
        await client.ws.send_text(pl)

    def on_new_state(self, state):
        self.game_state = state
        asyncio.create_task(self.broadcast())

    async def broadcast(self):
        dc = []
        for client in self.connections:
            try:
                await self.send_entities(client)
            except WebSocketDisconnect:
                dc.append(client)
            except:
                logging.exception("Sending game state failed")
                dc.append(client)
        # disconnected clients
        for c in dc:
            await self.disconnect(c)


world_messanger = WorldMessenger()


game_state_manager.on_new_state(world_messanger.on_new_state)


# NOTE:
# the router.websocket ignores the router's path prefix
@router.websocket("/world/object-stream")
async def object_stream(
    ws: WebSocket, manager: WorldMessenger = Depends(lambda: world_messanger)
):
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
            await manager.send_terrain(client)
            await manager.send_entities(client)
    except WebSocketDisconnect:
        pass
    except:
        logging.exception("Error in object streaming to client")
    finally:
        await manager.disconnect(client)
