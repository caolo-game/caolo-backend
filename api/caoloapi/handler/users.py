from typing import Optional
import logging
import string
import random
from uuid import UUID

from fastapi import (
    APIRouter,
    Request,
    Depends,
    HTTPException,
    Body,
    status,
)
from fastapi.security import OAuth2PasswordBearer, OAuth2PasswordRequestForm
from pydantic import BaseModel, Field, EmailStr
from jose import JWTError


from asyncpg.exceptions import UniqueViolationError

from ..model.auth import (
    hashpw,
    verifypw,
    PEPPER_RANGE,
    create_access_token,
    decode_access_token,
)


router = APIRouter(tags=["users"])


class User(BaseModel):
    user_id: UUID
    username: str
    displayname: str
    email: Optional[str] = None


oauth2_scheme = OAuth2PasswordBearer(tokenUrl="token")


async def get_current_user_id(token: str = Depends(oauth2_scheme)):
    def credentials_exception():
        return HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Could not validate credentials",
            headers={"WWW-Authenticate": "Bearer"},
        )

    try:
        payload = decode_access_token(token)
    except (AssertionError, JWTError) as err:
        logging.exception("Failed to validate JWT")
        raise credentials_exception() from err
    return payload.get("sub")


@router.get("/myself")
async def get_myself(req: Request, current_user=Depends(get_current_user_id)):
    current_user = await req.state.db.fetchrow(
        """
        SELECT id, username, email, display_name
        FROM user_account
        WHERE id=$1
        """,
        current_user,
    )
    return User(
        user_id=current_user["id"],
        username=current_user["username"],
        email=current_user["email"],
        displayname=current_user["display_name"],
    )


def __verify_pw(pw, salt, hashed):
    for pep in range(*PEPPER_RANGE):
        if verifypw(pw, salt, pep, hashed):
            return True
    return False


class RegisterForm(BaseModel):
    username: str = Field(..., min_length=3, max_length=125)
    email: EmailStr
    pw: str = Field(
        ...,
        min_length=8,
        max_length=125,
        description="Passwords must contain at least 8 characters",
    )


@router.post("/register")
async def register(req: Request, form_data: RegisterForm = Body(...)):
    db = req.state.db

    raw_pw = form_data.pw
    salt = "".join(random.choice(string.ascii_letters) for i in range(10))
    pepper = random.choice(range(*PEPPER_RANGE))

    pw = hashpw(raw_pw, salt, pepper)

    try:
        res = await db.fetchrow(
            """
            INSERT INTO user_account (username, display_name, email, pw, salt)
            VALUES ($1, $1, $2, $3, $4)
            RETURNING id
            """,
            form_data.username,
            form_data.email,
            pw,
            salt,
        )

    except UniqueViolationError as err:
        status_code = status.HTTP_500_INTERNAL_SERVER_ERROR
        detail = ""
        if err.constraint_name == "username_is_unique":
            status_code = status.HTTP_400_BAD_REQUEST
            detail = "Username is already in use"
        elif err.constraint_name == "email_is_unique":
            status_code = status.HTTP_400_BAD_REQUEST
            detail = "Email is already in use"
        else:
            logging.exception("Failed to register new user, constraint not handled")

        raise HTTPException(status_code=status_code, detail=detail) from err

    token = await _update_access_token(res["id"], db)
    return {"access_token": token, "token_type": "bearer"}


async def _update_access_token(userid, db):
    """
    generate a new access token for the given user and store it in the database
    """
    token = create_access_token({"sub": str(userid)})

    await db.execute(
        """
        UPDATE user_account
        SET token=$2
        WHERE id=$1
        """,
        userid,
        token,
    )

    return token


@router.post("/token")
async def login4token(req: Request, form_data: OAuth2PasswordRequestForm = Depends()):
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

    if not __verify_pw(form_data.password, user_in_db["salt"], user_in_db["pw"]):
        raise HTTPException(status_code=400, detail="Incorrect username or password")

    token = await _update_access_token(user_in_db["id"], db)
    return {"access_token": token, "token_type": "bearer"}
