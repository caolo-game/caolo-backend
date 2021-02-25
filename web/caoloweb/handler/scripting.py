from typing import Dict, List, Tuple
from fastapi import APIRouter, Response, Query, Request
import json

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
    # TODO: just write the already encoded response...
    if not res_encoded:
        return []
    return json.loads(res_encoded)
