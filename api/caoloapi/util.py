import inspect
import logging
import asyncio


def aio_with_backoff(*, retries=None, max_sleep=None):
    def _decorator(func):
        assert inspect.iscoroutinefunction(func) or inspect.isasyncgenfunction(
            func
        ), "Only async context is supported"

        async def _wrapper(*args, **kwargs):
            i = 0
            while 1:
                try:
                    return await func(*args, **kwargs)
                except:
                    if retries is not None and i > retries:
                        raise
                    sleep_dur = 1 << i
                    if max_sleep is not None:
                        sleep_dur = min(sleep_dur, max_sleep)
                    i += 1
                    logging.debug("Sleeping for %d", sleep_dur)
                    await asyncio.sleep(sleep_dur)

        return _wrapper

    return _decorator
