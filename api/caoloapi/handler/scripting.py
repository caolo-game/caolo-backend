from typing import Dict, List, Any, Optional
from uuid import UUID
import json
import logging

import cao_lang

from fastapi import (
    APIRouter,
    Response,
    Query,
    Request,
    HTTPException,
    Body,
    Depends,
    status,
)
from pydantic import BaseModel
from asyncpg.exceptions import UniqueViolationError

from google.protobuf.json_format import MessageToDict

from cao_script_pb2 import Empty
from cao_script_pb2_grpc import ScriptingStub

from .users import get_current_user_id
from ..queen import queen_channel

router = APIRouter(prefix="/scripting", tags=["scripting"])


@router.get("/schema", response_model=List[Dict])
async def get_schema(req: Request):
    stub = ScriptingStub(await queen_channel())
    res = await stub.GetSchema(Empty())
    return MessageToDict(
        res, including_default_value_fields=True, preserving_proto_field_name=False
    ).get("cards", [])


def _compile_caolang_program(prog_json: str):
    try:
        compilation_unit = cao_lang.CompilationUnit.from_json(prog_json)
    except ValueError as err:
        raise HTTPException(
            status_code=400, detail=f"Failed to parse compilation unit: {err}"
        ) from err

    try:
        _program = cao_lang.compile(compilation_unit)
    except ValueError as err:
        raise HTTPException(
            status_code=400, detail=f"Failed to compile program: {err}"
        ) from err


class CaoLangCard(BaseModel):
    """
    A Cao-Lang card
    """

    ty: str
    val: Any


class CaoLangLane(BaseModel):
    """
    See the `schema` endpoint for available cards
    """

    name: Optional[str]
    cards: List[CaoLangCard]


class CaoLangProgram(BaseModel):
    lanes: List[CaoLangLane]


@router.get("/cao-lang-version")
async def cao_lang_version() -> str:
    """
    return the version of Cao-Lang currently in use
    """
    return cao_lang.native_version()


@router.post("/compile")
async def compile_program(req: Request, _body: CaoLangProgram = Body(...)):
    # Body is used for openapi hint
    # we need the program to be json encoded, so just use the raw body
    payload: bytes = await req.body()
    payload = payload.decode(encoding="UTF-8")
    _compile_caolang_program(payload)
    return {"status": "Ok"}


class CommitCaoLangProgram(BaseModel):
    program: CaoLangProgram
    program_id: UUID


@router.post("/commit")
async def commit_script(
    req: Request,
    body: CommitCaoLangProgram = Body(...),
    current_user=Depends(get_current_user_id),
):
    """
    Set a new version for the given program.
    """

    db = req.state.db

    # we need the program to be json encoded
    payload = json.dumps(body.program, default=dict)
    _compile_caolang_program(payload)

    # if the program compiles we can save it in the database
    program_id = body.program_id
    res = await db.fetchrow(
        """
UPDATE user_script
SET program=$1
WHERE id=$2 AND owner_id=$3
RETURNING id
""",
        payload,
        program_id,
        current_user,
    )
    if res is None:
        raise HTTPException(404, "Program not found")

    return {"program_id": program_id}


class NewProgramForm(BaseModel):
    name: str


@router.post("/create-program")
async def init_new_script(
    req: Request,
    body: NewProgramForm = Body(...),
    current_user=Depends(get_current_user_id),
):
    """
    Set a new version for the given program.
    """

    db = req.state.db

    # if the program compiles we can save it in the database
    try:
        res = await db.fetchrow(
            """
            INSERT INTO user_script (program,owner_id,name)
            VALUES($1,$2,$3)
            RETURNING id
            """,
            json.dumps(None),
            current_user,
            body.name,
        )
    except UniqueViolationError as err:
        status_code = status.HTTP_500_INTERNAL_SERVER_ERROR
        detail = ""
        if err.constraint_name == "name_owner_id_unique":
            status_code = status.HTTP_400_BAD_REQUEST
            detail = "Program name is already in use"

        else:
            logging.exception("Failed to insert new program, constraint not handled")
        raise HTTPException(status_code, detail) from err

    return {"program_id": res["id"]}


@router.get("/my-programs")
async def list_my_programs(req: Request, current_user_id=Depends(get_current_user_id)):
    db = req.state.db
    res = await db.fetch(
        """
        SELECT id,name,created,updated
        FROM user_script
        WHERE owner_id=$1
        """,
        current_user_id,
    )
    return res


@router.get("/program")
async def fetch_program(
    req: Request,
    program_id: UUID = Query(...),
    current_user_id=Depends(get_current_user_id),
):
    res = await req.state.db.fetchrow(
        """
        SELECT id,name,program,created,updated
        FROM user_script
        WHERE owner_id=$1 AND id=$2
        """,
        current_user_id,
        program_id,
    )
    if res is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND, detail="Program not found")
    return res


@router.delete("/program")
async def delete_program(
    req: Request,
    program_id: UUID = Query(...),
    current_user_id=Depends(get_current_user_id),
):
    await req.state.db.execute(
        """
        DELETE
        FROM user_script
        WHERE owner_id=$1 AND id=$2
        """,
        current_user_id,
        program_id,
    )
    return {"status": "ok"}


class UpdateProgramForm(BaseModel):
    program_id: UUID
    name: str


@router.put("/program")
async def update_program(
    req: Request,
    form: UpdateProgramForm = Body(...),
    current_user_id=Depends(get_current_user_id),
):
    res = await req.state.db.fetchrow(
        """
        UPDATE user_script
        SET name=$1
        WHERE user_script.id=$2 AND user_script.owner_id=$3
        RETURNING user_script.id
        """,
        form.name,
        form.program_id,
        current_user_id,
    )

    if res is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND)

    assert res["id"] == form.program_id

    return {"status": "ok"}
