FROM rust:latest AS build

RUN apt-get update
RUN apt-get install clang lld capnproto -y

WORKDIR /caolo

COPY ./rust-toolchain ./rust-toolchain
COPY ./.cargo/ ./.cargo/
# cache the toolchain
RUN cargo --version

RUN cargo install diesel_cli --root . --no-default-features --features="postgres"

ENV SQLX_OFFLINE=1

# ============= cache dependencies =============
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN mkdir src/
RUN echo "fn main() {}" > ./src/dummy.rs
RUN sed -i 's/src\/main.rs/src\/dummy.rs/' Cargo.toml
RUN cargo build --release

COPY ./sqlx-data.json ./sqlx-data.json
COPY ./Cargo.lock ./Cargo.lock
COPY ./src/ ./src/
COPY ./Cargo.toml ./Cargo.toml
COPY ./migrations/ ./migrations/

RUN cargo install --path . --root .

# ---------- Copy the built binary to a scratch container, to minimize the image size ----------

FROM ubuntu:20.04
WORKDIR /caolo
RUN apt-get update -y
RUN apt-get install curl libpq-dev -y --fix-missing

COPY ./migrations/ ./migrations/
COPY ./release.sh ./
COPY --from=build /caolo/bin/ ./

RUN ls -al /caolo

ENTRYPOINT ["./caolo-web"]
