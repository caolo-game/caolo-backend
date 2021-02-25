FROM rust:latest AS build

WORKDIR /caolo
RUN cargo install diesel_cli --root . --no-default-features --features="postgres"
# ---------- Copy the built binary to a scratch container, to minimize the image size ----------

FROM ubuntu:18.04
WORKDIR /caolo
RUN apt-get update
RUN apt-get install libpq-dev -y

COPY ./migrations/ ./migrations/
COPY ./release.sh ./
COPY --from=build /caolo/bin/ ./

ENTRYPOINT ["./release.sh"]
