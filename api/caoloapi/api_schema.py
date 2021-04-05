from typing import List, Any
from pydantic import BaseModel, Field


class RoomObjectsPayload(BaseModel):
    bots: List[Any] = []
    structures: List[Any] = []
    resources: List[Any] = []


class RoomObjects(BaseModel):
    payload: RoomObjectsPayload = RoomObjectsPayload()
    time: int = -1


class Axial(BaseModel):
    q: int = 0
    r: int = 0

    def __init__(self, q=0, r=0):
        super().__init__()
        self.q = q
        self.r = r


class WorldPosition(BaseModel):
    room: Axial
    pos: Axial


class StructureType(BaseModel):
    value: int = Field(ge=0, lt=1)


def parse_room_id(room_id: str) -> Axial:
    q, r = room_id.split(";")
    return Axial(q, r)


def make_room_id(q: int, r: int):
    return f"{q};{r}"
