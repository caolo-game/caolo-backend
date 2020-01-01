import redis
import os


def get_redis_client():
    return redis.Redis.from_url(
        os.getenv("REDIS_URL", "redis://localhost:6379/0"))
