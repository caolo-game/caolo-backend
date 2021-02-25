import json

from ..api_schema import RoomObjects


def get_room_objects(game_state, room_id: str):
    payload = RoomObjects()
    payload.time = game_state["time"]
    payload.payload = {
        "bots": game_state["bots"].get(room_id, []),
        "structures": game_state["structures"].get(room_id, []),
        "resources": game_state["resources"].get(room_id, []),
    }
    return payload


async def load_latest_game_state(db):
    row = await db.fetchrow(
        """
        SELECT
            t.payload as payload
            , t.world_time as time
        FROM public.world_output t
        ORDER BY t.created DESC
        """,
    )
    pl = json.loads(row["payload"])
    pl["time"] = row["time"]
    return pl
