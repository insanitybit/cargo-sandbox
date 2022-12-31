# syntax=docker/dockerfile:1.4
# We use the above syntax for here documents:
# https://github.com/moby/buildkit/blob/master/frontend/dockerfile/docs/syntax.md#user-content-here-documents

ARG RUST_VERSION=1.63

FROM rust:${RUST_VERSION}-slim-bullseye AS base

SHELL ["/bin/bash", "-o", "errexit", "-o", "nounset", "-o", "pipefail", "-c"]

# `curl` for riff
RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=rust-base-apt \
    apt-get update \
    && apt-get install --yes --no-install-recommends \
        curl=7.74.0-1.3+deb11u3 \
    && adduser \
        --disabled-password \
        --gecos '' \
        --home /home/grapl \
        --shell /bin/bash \
        cargo-sandbox-user

USER cargo-sandbox-user
ENV USER=cargo-sandbox-user
WORKDIR /home/cargo-sandbox-user

# https://determinate.systems/posts/riff-rust-maintainers
RUN curl --proto '=https' --tlsv1.3 -Lo riff https://github.com/DeterminateSystems/riff/releases/download/v1.0.2/riff-x86_64-linux
RUN mkdir -p /usr/local/bin
RUN install -m +x ./riff /usr/local/bin/riff
RUN rustup show
