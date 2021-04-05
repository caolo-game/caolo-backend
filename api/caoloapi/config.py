import os

try:
    from dotenv import load_dotenv

    load_dotenv()
except:
    pass

DB_URL = os.getenv("DATABASE_URL", "postgres://postgres:admin@localhost:5432/caolo")
REDIS_STR = os.getenv("REDIS_URL", "redis://localhost:6379/0")
QUEEN_URL = os.getenv("CAO_QUEEN_URL", "localhost:50051")

try:
    QUEEN_TAG = os.getenv("CAO_QUEEN_TAG")
    assert QUEEN_TAG is not None
except:
    logging.exception("Failed to find my queen :(")
    raise
