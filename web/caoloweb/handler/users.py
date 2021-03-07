import json
from typing import Dict, List, Tuple, Optional

from fastapi import APIRouter, Response, Query, Request, Depends, HTTPException
from fastapi.security import OAuth2PasswordBearer, OAuth2PasswordRequestForm
from pydantic import BaseModel

from .. import app
from ..model.auth import hashpw, verifypw, PEPPER_RANGE


router = APIRouter()


class User(BaseModel):
    username: str
    email: Optional[str] = None


oauth2_scheme = OAuth2PasswordBearer(tokenUrl="token")


async def get_current_user(db, token: str = Depends(oauth2_scheme)):
    return User(username="boi")


@router.get("/myself")
async def get_myself(current_user=Depends(get_current_user)):
    return current_user


def __verify(pw, salt, hashed):
    for i in range(*PEPPER_RANGE):
        if verifypw(pw, salt, i, hashed):
            return True
    return False


@router.post("/token")
async def login(
    req: Request,
    form_data: OAuth2PasswordRequestForm = Depends(),
):
    db = req.state.db

    user_in_db = await db.fetchrow(
        """
        SELECT id, pw, salt
        FROM user_account
        WHERE username=$1
        """,
        form_data.username,
    )

    if not user_in_db:
        raise HTTPException(status_code=400, detail="Incorrect username or password")

    if not __verify(form_data.password, user_in_db["salt"], user_in_db["pw"]):
        raise HTTPException(status_code=400, detail="Incorrect username or password")

    # TODO:
    # produce token
    # save token in db
    return {"access_token": user_in_db["id"], "token_type": "bearer"}
