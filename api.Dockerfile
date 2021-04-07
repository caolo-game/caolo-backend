# ----------- Venv cache hack image -----------

FROM python:3.9-slim AS venv

WORKDIR /caolo/api
RUN python -m venv .env
RUN .env/bin/pip install gunicorn grpcio-tools

# ----------- Build image -----------

FROM python:3.9-slim AS build

RUN apt-get update
RUN apt-get install curl build-essential -y
RUN pip install -U setuptools pip virtualenv

WORKDIR /caolo
# install Rust compiler
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup update
RUN rustc --version
RUN cargo --version

RUN mkdir ./protos
ENV CAO_PROTOS_PATH=/caolo/protos

# Blind-bake dependencies by running setup with an empty caoloapi/ directory
COPY ./api/pyproject.toml ./api/pyproject.toml
COPY ./api/setup.py ./api/setup.py
RUN mkdir ./api/caoloapi
RUN mkdir ./api/caoloapi/protos

WORKDIR /caolo/api

# copy our cached virtualenv
COPY --from=venv /caolo/api/.env ./.env

# Install deps
RUN .env/bin/pip install . --no-cache-dir

# Actually install caoloapi
COPY ./protos/ ./protos/
COPY ./api/ ./
RUN .env/bin/pip install . --no-cache-dir

# ----------- Prod image -----------

FROM python:3.9-slim

WORKDIR /caolo/api

RUN apt-get update
RUN apt-get install libpq-dev -y

COPY --from=build /caolo/api/start.sh ./
COPY --from=build /caolo/api/ ./

ENV PATH="/caolo/api/.env/bin:$PATH"

RUN chmod +x start.sh

ENTRYPOINT [ "sh", "./start.sh"]
