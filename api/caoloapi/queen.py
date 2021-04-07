import grpc
from .config import QUEEN_URL

_QUEEN_CHANNEL = None


async def queen_channel(queen_url=QUEEN_URL):
    global _QUEEN_CHANNEL
    if not _QUEEN_CHANNEL:
        _QUEEN_CHANNEL = grpc.aio.insecure_channel(queen_url)
    return _QUEEN_CHANNEL
