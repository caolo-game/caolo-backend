from pydantic import BaseModel, Field


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
