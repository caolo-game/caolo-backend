#!/usr/bin/sh

gunicorn -w ${WEB_CONCURRENCY:-8} -k uvicorn.workers.UvicornWorker caoloapi.app:app --log-level=info --access-logfile=-
