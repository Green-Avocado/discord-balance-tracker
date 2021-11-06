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

ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --target x86_64-unknown-linux-musl

##
## Production
##
From scratch AS production-env

WORKDIR /application

COPY --from=build-env /build/target/x86_64-unknown-linux-musl/debug/discord-balance-tracker ./
COPY .env ./

USER 66534:66534

CMD ["./discord-balance-tracker"]
