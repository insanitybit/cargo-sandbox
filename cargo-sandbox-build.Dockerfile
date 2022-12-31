# syntax=docker/dockerfile:1.4

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
        --home /home/cargo-sandbox-user \
        --shell /bin/bash \
        cargo-sandbox-user

RUN mkdir -p /usr/local/bin

USER cargo-sandbox-user
ENV USER=cargo-sandbox-user
WORKDIR /home/cargo-sandbox-user

# https://determinate.systems/posts/riff-rust-maintainers
RUN curl --proto '=https' --tlsv1.3 -Lo riff https://github.com/DeterminateSystems/riff/releases/download/v1.0.2/riff-x86_64-linux

USER root
RUN install -m +x ./riff /usr/local/bin/riff

USER cargo-sandbox-user
RUN rustup show
