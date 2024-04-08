.PHONY: build push

build:
	docker build --tag appsignal/appsignal-kubernetes:latest .

push: build
	docker push appsignal/appsignal-kubernetes:latest



