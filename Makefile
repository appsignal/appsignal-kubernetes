TAG=$(shell git describe --tags --abbrev=0 | tr --delete a-z)

.PHONY: build push setup protocol

build:
	docker buildx build \
          --builder=appsignal-container \
          --tag appsignal/appsignal-kubernetes:$(TAG) \
          --tag appsignal/appsignal-kubernetes:latest \
          .

push:
	docker buildx build \
          --platform linux/amd64,linux/arm64 \
          --builder=appsignal-container \
          --push \
          --tag appsignal/appsignal-kubernetes:$(TAG) \
          --tag appsignal/appsignal-kubernetes:latest \
          .

setup:
	docker buildx create --name appsignal-container --driver=docker-container

protocol:
	mkdir -p protocol
	protoc -I ../appsignal-protocol --rust_out=protocol ../appsignal-protocol/kubernetes.proto
