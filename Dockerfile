FROM rust:1.75.0-alpine3.19 AS build

ARG TARGETPLATFORM

RUN apk add --no-cache clang libressl-dev

RUN --mount=type=cache,target=target,id=cache-${TARGETPLATFORM} \
    --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    cargo build --release && cp target/release/appsignal-kubernetes appsignal-kubernetes

FROM alpine:3.20.3

COPY --from=build /appsignal-kubernetes /appsignal-kubernetes

CMD ["/appsignal-kubernetes"]
