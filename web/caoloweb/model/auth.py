from passlib.context import CryptContext

SECRET_KEY = "fe9fb923daa2a5c34a57b6da5d807a1e9cb48d4afee5c10095bab37bcf860059"
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 30
PEPPER_RANGE = (128, 138, 1)


pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")


def __concatpw(pw: str, salt: str, pepper):
    return f"{pw}-{salt}-{pepper}"


def verifypw(plain, salt, pepper, hashed_pw):
    pw = __concatpw(plain, salt, pepper)
    return pwd_context.verify(pw, hashed_pw)


def hashpw(pw: str, salt: str, pepper):
    return pwd_context.hash(__concatpw(pw, salt, pepper))
