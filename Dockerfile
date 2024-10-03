FROM alpine:3.20.3

ENV REFRESHED_AT=2024-09-25

ARG TARGETARCH
COPY release/$TARGETARCH-unknown-linux-musl/appsignal-kubernetes /appsignal-kubernetes

CMD ["/appsignal-kubernetes"]
