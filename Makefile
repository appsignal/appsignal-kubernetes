TAG=$(shell ./script/read_version)
PLATFORMS ?= "linux/amd64,linux/arm64"

.PHONY: build push setup

build:
	docker buildx build \
          --platform $(PLATFORMS) \
          --builder=appsignal-container \
          --load \
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
