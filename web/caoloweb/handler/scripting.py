from typing import Dict, List, Tuple, Optional
from uuid import UUID
import json

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

from .users import get_current_user_id

router = APIRouter(prefix="/scripting")


@router.get("/schema", response_model=List[Dict])
async def get_schema(req: Request):
    res_encoded = await req.state.db.fetchval(
        """
        SELECT t.payload
        FROM public.scripting_schema t
        ORDER BY t.created DESC
        """
    )
    # returned data is already json encoded string
    # just write the already encoded response...
    return Response(res_encoded, media_type="application/json")


def _compile_caolang_program(prog_json: str):
    try:
        compilation_unit = cao_lang.CompilationUnit.from_json(prog_json)
    except ValueError as err:
        raise HTTPException(
            status_code=400, detail=f"Failed to parse compilation unit: {err}"
        )

    try:
        _program = cao_lang.compile(compilation_unit)
    except ValueError as err:
        raise HTTPException(status_code=400, detail=f"Failed to compile program: {err}")


class CaoLangLane(BaseModel):
    """
    See the `schema` endpoint for available cards
    """

    cards: List


class CaoLangProgram(BaseModel):
    lanes: List[CaoLangLane]


@router.post("/compile")
async def compile(req: Request, _body: CaoLangProgram = Body(...)):
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
    payload = json.dumps(body.program, default=lambda o: dict(o))
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
        raise HTTPException(status_code, detail)

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
async def fetch_program(
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
