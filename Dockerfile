FROM rust:1.75.0-alpine3.19 AS build

RUN apk add --no-cache clang libressl-dev

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    cargo build --release

FROM alpine:3.20.3

COPY --from=build /target/release/appsignal-kubernetes /target/release/appsignal-kubernetes

CMD ["/target/release/appsignal-kubernetes"]
