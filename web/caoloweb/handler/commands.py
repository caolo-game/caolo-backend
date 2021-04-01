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

import cao_commands_pb2 as cao_commands

from ..config import QUEEN_TAG
from .users import get_current_user_id


router = APIRouter(prefix="/commands", tags=["commands"])


class BotScriptPayload(BaseModel):
    bot_id: int
    script_id: UUID


async def _send_command(payload: str, msg_id: UUID, redis: Redis):

    q_name = f"{QUEEN_TAG}-commands"

    await redis.lpush(q_name, payload)

    for i in range(10):
        sleep_dur = 1 << i
        logging.debug("sleeping for ", sleep_dur)
        await asyncio.sleep(sleep_dur)

        res_payload = await redis.get(str(msg_id))
        if res_payload is not None:

            response = cao_commands.CommandResult()
            response.ParseFromString(res_payload)

            logging.debug("response:", response)
            return response

    raise RuntimeError("Command await timed out")


def _write_uuid(field, identifier: UUID):
    field.data = identifier.bytes


@router.post("/bot-script")
async def set_bot_script(
    req: Request,
    req_payload: BotScriptPayload = Body(...),
    current_user_id=Depends(get_current_user_id),
):
    msg_id = uuid4()

    msg = cao_commands.InputMessage()
    _write_uuid(msg.messageId, msg_id)
    current_user_id = UUID(current_user_id)
    _write_uuid(msg.updateEntityScript.userId, current_user_id)
    _write_uuid(msg.updateEntityScript.userId, req_payload.script_id)
    msg.updateEntityScript.entityId = req_payload.bot_id

    redis = req.state.cache

    result = await _send_command(msg.SerializeToString(), msg_id, redis)

    if result.error:
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail=result.error)

    return {"status": "ok"}
