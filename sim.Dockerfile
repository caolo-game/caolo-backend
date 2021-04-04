# ============= cache dependencies ============================================================
FROM rust:1.51 AS deps

RUN apt-get update
RUN apt-get install lld clang libc-dev pkgconf -y

WORKDIR /caolo

COPY ./.cargo/ ./.cargo/
RUN cargo --version

WORKDIR /caolo
COPY ./sim/cao-storage-derive/ ./cao-storage-derive/
COPY ./sim/worker/Cargo.toml ./worker/Cargo.toml
COPY ./sim/simulation/Cargo.toml ./simulation/Cargo.toml
COPY ./sim/Cargo.toml ./Cargo.toml
COPY ./sim/Cargo.lock ./Cargo.lock

RUN mkdir worker/src/
RUN mkdir simulation/src/
RUN echo "fn main() {println!(\"If you see this the build did a doo doo\");}" > ./worker/src/main.rs
RUN touch ./simulation/src/lib.rs

# Delete the build script
RUN sed -i '/build\s*=\s*\"build\.rs\"/d' worker/Cargo.toml
RUN sed -i '/build\s*=\s*\"build\.rs\"/d' simulation/Cargo.toml
# Delete the bench section
RUN sed -i '/\[\[bench/,+2d' simulation/Cargo.toml


ENV SQLX_OFFLINE=true
RUN cargo build --release
RUN rm -f target/release/deps/caolo_*

# ==============================================================================================

FROM rust:1.51 AS build

RUN apt-get update
RUN apt-get install lld clang libc-dev  pkgconf libpq-dev protobuf-compiler -y

WORKDIR /caolo

RUN cargo install diesel_cli --no-default-features --features=postgres --root .

# copy the cache
COPY --from=deps $CARGO_HOME $CARGO_HOME
COPY --from=deps /caolo/target ./sim/target
COPY --from=deps /caolo/Cargo.lock ./sim/Cargo.lock

COPY ./.cargo/ ./.cargo/
RUN cargo --version
RUN protoc --version

COPY ./protos/ ./protos/
COPY ./sim/ ./sim/
WORKDIR /caolo/sim

ENV SQLX_OFFLINE=true
RUN cargo build --release

# ========== Copy the built binary to a scratch container, to minimize the image size ==========

FROM ubuntu:18.04
WORKDIR /caolo

RUN apt-get update -y
RUN apt-get install bash libpq-dev openssl -y

COPY ./migrations ./migrations
COPY --from=build /caolo/sim/target/release/caolo-worker ./caolo-worker
COPY --from=build /caolo/bin/diesel ./diesel

RUN ls -al

ENTRYPOINT [ "./caolo-worker" ]
