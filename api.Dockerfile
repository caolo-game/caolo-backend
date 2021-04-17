# ----------- Build image -----------

FROM python:3.9-slim AS build

RUN apt-get update
RUN apt-get install curl git build-essential -y
RUN pip install -U pip virtualenv

WORKDIR /caolo/api
RUN python -m venv .env
RUN .env/bin/pip install gunicorn poetry

WORKDIR /caolo
# install Rust compiler
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup update
RUN rustc --version
RUN cargo --version

RUN mkdir ./protos
ENV CAO_PROTOS_PATH=/caolo/protos

COPY ./api/pyproject.toml ./api/pyproject.toml
COPY ./api/poetry.lock ./api/poetry.lock
RUN mkdir ./api/caoloapi
RUN mkdir ./api/caoloapi/protos

WORKDIR /caolo/api

# Install deps
RUN .env/bin/poetry export -f requirements.txt -o requirements.txt
# split git requirements, because the --hash command shits the bed otherwise
RUN grep git requirements.txt > git-req.txt
RUN sed -i '/.*git.*/d' requirements.txt

RUN .env/bin/pip install -r requirements.txt
RUN .env/bin/pip install -r git-req.txt

# Actually install caoloapi
WORKDIR /caolo
COPY ./protos/ ./protos/
COPY ./api/ ./api/
WORKDIR /caolo/api
RUN .env/bin/pip install -e.

# ----------- Prod image -----------

FROM python:3.9-slim

WORKDIR /caolo/api

COPY --from=build /caolo/api/start.sh ./
COPY --from=build /caolo/api/ ./

ENV PATH="/caolo/api/.env/bin:$PATH"

RUN chmod +x start.sh

ENTRYPOINT [ "sh", "./start.sh"]
