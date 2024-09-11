FROM rust:alpine

RUN apk add --no-cache clang libressl-dev

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=protocol,target=protocol \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    cargo build --release

CMD ["/target/release/appsignal-kubernetes"]
