#/bin/sh

docker build -t cargo-sandbox-build - < cargo-sandbox-build.Dockerfile && \
docker build -t cargo-sandbox-publish - < cargo-sandbox-publish.Dockerfile
