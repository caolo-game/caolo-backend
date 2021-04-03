# ----------- Venv cache hack image -----------

FROM python:3.9-alpine AS venv
WORKDIR /caolo/api
RUN python -m venv .env
RUN .env/bin/pip install gunicorn

RUN pip install -U setuptools pip virtualenv

# ----------- Build image -----------

FROM python:3.9-alpine AS build

RUN apk add curl gcc libpq protobuf build-base libffi-dev
RUN protoc --version

RUN pip install -U setuptools pip virtualenv

WORKDIR /caolo
# install Rust compiler
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup update
RUN rustc --version
RUN cargo --version

COPY ./protos/ ./protos/
ENV CAO_PROTOS_PATH=/caolo/protos

# Blind-bake dependencies by running setup with an empty caoloapi/ directory
COPY ./api/setup.py ./api/setup.py
RUN mkdir ./api/caoloapi
RUN mkdir ./api/caoloapi/protos

WORKDIR /caolo/api

# copy our cached virtualenv
COPY --from=venv /caolo/api/.env ./.env

# Install deps
RUN .env/bin/pip install . --no-cache-dir

# Actually install caoloapi
COPY ./api/ ./
RUN .env/bin/pip install . --no-cache-dir

# ----------- Prod image -----------

FROM python:3.9-alpine

WORKDIR /caolo/api

RUN apk add gcc libpq

COPY --from=build /caolo/api/start.sh ./
COPY --from=build /caolo/api/ ./

ENV PATH="/caolo/api/.env/bin:$PATH"

RUN chmod +x start.sh

ENTRYPOINT [ "sh", "./start.sh"]
