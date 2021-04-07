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
import cao_script_pb2_grpc
import cao_script_pb2 as cao_script

from .scripting import CaoLangProgram, _compile_caolang_program
from ..config import QUEEN_TAG, QUEEN_URL
from .users import get_current_user_id
from ..api_schema import WorldPosition, StructureType
from ..queen import queen_channel


router = APIRouter(prefix="/commands", tags=["commands"])


def commands_stub():
    channel = queen_channel()
    return cao_commands_pb2_grpc.CommandStub(channel)

def scripting_stub():
    channel = queen_channel()
    return cao_script_pb2_grpc.ScriptingStub(channel)


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

    stub = scripting_stub()

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
    stub = commands_stub()
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


class UpdateScriptPayload(BaseModel):
    script_id: UUID
    program: CaoLangProgram


@router.post("/update-script")
async def update_script(
    req: Request,
    req_payload: UpdateScriptPayload = Body(...),
    current_user_id=Depends(get_current_user_id),
):

    script_payload = json.dumps(req_payload.program, default=dict)

    # return error if the script does not compile..
    _compile_caolang_program(script_payload)

    msg = cao_commands.UpdateScriptCommand()
    msg.compilationUnit.compilationUnit.value = script_payload.encode("utf-8")

    msg.scriptId.data = req_payload.script_id.bytes
    msg.userId.data = UUID(current_user_id).bytes

    stub = scripting_stub()
    try:
        _result = await stub.UpdateScript(msg)
    except grpc.aio.AioRpcError as err:
        if err.code() == grpc.StatusCode.INVALID_ARGUMENT:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST, detail=err.details()
            ) from err
        logging.exception("Unhandled rpc error")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        ) from err
