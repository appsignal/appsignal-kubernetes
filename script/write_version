#!/bin/sh

VERSION=$1

sed -i '' -e 's/^version = ".*"$/version = "'$VERSION'"/' Cargo.toml
sed -i '' -e 's|image: appsignal/appsignal-kubernetes:.*$|image: appsignal/appsignal-kubernetes:'$VERSION'|' deployment.yaml
