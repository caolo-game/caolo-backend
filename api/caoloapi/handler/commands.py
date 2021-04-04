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

from aioredis import Redis
import asyncio
import grpc

import cao_commands_pb2 as cao_commands
import cao_commands_pb2_grpc

from ..config import QUEEN_TAG, CAO_URL
from .users import get_current_user_id


router = APIRouter(prefix="/commands", tags=["commands"])


class BotScriptPayload(BaseModel):
    bot_id: int
    script_id: UUID


def _write_uuid(field, identifier: UUID):
    field.data = identifier.bytes


@router.post("/bot-script")
async def set_bot_script(
    req: Request,
    req_payload: BotScriptPayload = Body(...),
    current_user_id=Depends(get_current_user_id),
):

    msg = cao_commands.UpdateEntityScriptCommand()
    current_user_id = UUID(current_user_id)
    _write_uuid(msg.userId, current_user_id)
    _write_uuid(msg.scriptId, req_payload.script_id)
    msg.entityId = req_payload.bot_id

    channel = grpc.aio.insecure_channel(CAO_URL)
    stub = cao_commands_pb2_grpc.CommandStub(channel)

    try:
        _result = await stub.UpdateEntityScript(msg)
    except grpc.aio.AioRpcError as err:
        if err.code() == grpc.StatusCode.INVALID_ARGUMENT:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST, detail=err.details()
            )
        logging.exception("Unhandled rpc error")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        )

    return {"status": "ok"}
