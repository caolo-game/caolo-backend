from typing import List, Any
from pydantic import BaseModel


class RoomObjectsPayload(BaseModel):
    bots: List[Any] = []
    structures: List[Any] = []
    resources: List[Any] = []


class RoomObjects(BaseModel):
    payload: RoomObjectsPayload = RoomObjectsPayload()
    time: int = -1


class Axial(BaseModel):
    q: int
    r: int


def parse_room_id(room_id: str):
    return room_id.split(";")


def make_room_id(q: int, r: int):
    return f"{q};{r}"
