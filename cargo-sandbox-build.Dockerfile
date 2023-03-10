ARG RUST_VERSION=1.63

FROM rust:${RUST_VERSION}-slim-bullseye AS base

SHELL ["/bin/bash", "-o", "errexit", "-o", "nounset", "-o", "pipefail", "-c"]

# `curl` for riff
RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=rust-base-apt \
    apt-get update \
    && apt-get install --yes --no-install-recommends \
        curl \
        xz-utils \
        coreutils \
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
RUN mkdir -p /nix && chown -R cargo-sandbox-user /nix

USER cargo-sandbox-user

RUN sh <(curl --proto '=https' --tlsv1.2 -sSf -L https://nixos.org/nix/install) --no-daemon
RUN rustup show
#RUN echo -e '\nsource prefix/etc/profile.d/nix.sh' >> ~/.profile
