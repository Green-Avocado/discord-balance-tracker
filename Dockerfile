# syntax=docker/dockerfile:1

##
## Build
##
From rust:alpine AS build-env

RUN apk add --no-cache musl-dev

WORKDIR /build

COPY ./src ./src
COPY ./Cargo.toml ./
COPY ./Cargo.lock ./

RUN mkdir data

ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release --target x86_64-unknown-linux-musl

##
## Production
##
From scratch AS production-env

WORKDIR /application

COPY --from=build-env /build/target/x86_64-unknown-linux-musl/release/discord-balance-tracker ./
COPY --from=build-env --chown=66534:66534 /build/data ./data
COPY .env ./

USER 66534:66534

CMD ["./discord-balance-tracker"]
