from typing import Dict, List, Tuple
from uuid import UUID, uuid4
import logging
from fastapi import (
    APIRouter,
    Response,
    Query,
    Request,
    Body,
    Depends,
    HTTPException,
    status,
)
import json
from pydantic import BaseModel

import asyncio
import grpc

import cao_commands_pb2 as cao_commands
import cao_commands_pb2_grpc

from ..config import QUEEN_TAG, QUEEN_URL
from .users import get_current_user_id
from ..api_schema import WorldPosition, StructureType


router = APIRouter(prefix="/commands", tags=["commands"])


def get_cao_commands_stub():
    channel = grpc.aio.insecure_channel(QUEEN_URL)
    return cao_commands_pb2_grpc.CommandStub(channel)


class BotScriptPayload(BaseModel):
    bot_id: int
    script_id: UUID


@router.post("/bot-script")
async def set_bot_script(
    req: Request,
    req_payload: BotScriptPayload = Body(...),
    current_user_id=Depends(get_current_user_id),
):

    current_user_id = UUID(current_user_id)

    msg = cao_commands.UpdateEntityScriptCommand()
    msg.userId.data = current_user_id.bytes
    msg.scriptId.data = req_payload.script_id.bytes
    msg.entityId = req_payload.bot_id

    channel = grpc.aio.insecure_channel(QUEEN_URL)
    stub = get_cao_commands_stub()

    try:
        _result = await stub.UpdateEntityScript(msg)
    except grpc.aio.AioRpcError as err:
        if err.code() == grpc.StatusCode.INVALID_ARGUMENT:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST, detail=err.details()
            ) from err
        logging.exception("Unhandled rpc error")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        ) from err

    return {"status": "ok"}


class PlaceStructurePayload(BaseModel):
    structure_type: StructureType
    position: WorldPosition


@router.post("/place-structure")
async def place_structure(
    req: Request,
    req_payload: PlaceStructurePayload = Body(...),
    current_user_id=Depends(get_current_user_id),
):
    current_user_id = UUID(current_user_id)

    msg = cao_commands.PlaceStructureCommand()
    msg.ownerId.data = current_user_id.bytes
    msg.position.room.q = req_payload.position.room.q
    msg.position.room.r = req_payload.position.room.r
    msg.position.pos.q = req_payload.position.pos.q
    msg.position.pos.r = req_payload.position.pos.r
    if req_payload.structure_type.value == cao_commands.StructureType.SPAWN:
        msg.ty = cao_commands.StructureType.SPAWN
    else:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST, detail="invalid structure type"
        )
    stub = get_cao_commands_stub()
    try:
        _result = await stub.PlaceStructure(msg)
    except grpc.aio.AioRpcError as err:
        if err.code() == grpc.StatusCode.INVALID_ARGUMENT:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST, detail=err.details()
            ) from err
        logging.exception("Unhandled rpc error")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        ) from err

    return {"status": "ok"}
