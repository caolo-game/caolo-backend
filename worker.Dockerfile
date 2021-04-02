FROM rust:1.51 AS deps

RUN apt-get update
RUN apt-get install lld clang libc-dev pkgconf -y

WORKDIR /caolo

COPY ./.cargo/ ./.cargo/
RUN cargo --version

# ============= cache dependencies ============================================================
WORKDIR /caolo
COPY ./worker/cao-storage-derive/ ./cao-storage-derive/
COPY ./worker/worker/Cargo.toml ./worker/Cargo.toml
COPY ./worker/simulation/Cargo.toml ./simulation/Cargo.toml
COPY ./worker/Cargo.toml ./Cargo.toml
COPY ./worker/Cargo.lock ./Cargo.lock

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
COPY ./.cargo/ ./.cargo/
RUN cargo --version

RUN apt-get update
RUN apt-get install lld clang libc-dev  pkgconf libpq-dev protobuf-compiler -y

WORKDIR /caolo
RUN protoc --version

RUN cargo install diesel_cli --no-default-features --features=postgres --root .

# copy the cache
COPY --from=deps $CARGO_HOME $CARGO_HOME

COPY ./protos/ ./protos/
COPY ./worker/ ./worker/

WORKDIR /caolo/worker
COPY --from=deps /caolo/target ./target
COPY --from=deps /caolo/Cargo.lock ./Cargo.lock

ENV SQLX_OFFLINE=true
RUN cargo build --release

# ========== Copy the built binary to a scratch container, to minimize the image size ==========

FROM ubuntu:18.04
WORKDIR /caolo

RUN apt-get update -y
RUN apt-get install bash libpq-dev openssl -y
# RUN apt-get install valgrind -y
# RUN apt-get install heaptrack -y

COPY ./migrations ./migrations
COPY --from=build /caolo/worker/target/release/caolo-worker ./caolo-worker
COPY --from=build /caolo/bin/diesel ./diesel

RUN ls -al

ENTRYPOINT [ "./caolo-worker" ]
