from typing import List, Any
from pydantic import BaseModel


class RoomObjectsPayload(BaseModel):
    bots: List[Any] = []
    structures: List[Any] = []
    resources: List[Any] = []


class RoomObjects(BaseModel):
    payload: RoomObjectsPayload = RoomObjectsPayload()
    time: int = 0
