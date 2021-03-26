from typing import Dict, List, Tuple
from fastapi import APIRouter, Response, Query, Request, HTTPException
import json

import cao_lang

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


@router.post("/compile")
async def compile(req: Request):
    payload: bytes = await req.body()
    payload = payload.decode(encoding="UTF-8")
    try:
        compilation_unit = cao_lang.CompilationUnit.from_json(payload)
    except ValueError as err:
        raise HTTPException(
            status_code=400, detail=f"Failed to parse compilation unit: {err}"
        )

    try:
        _program = cao_lang.compile(compilation_unit)
    except ValueError as err:
        raise HTTPException(status_code=400, detail=f"Failed to compile program: {err}")

    return {"status": "Ok"}
