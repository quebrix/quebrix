# syntax=docker/dockerfile:1

# Create a stage for building the application.

ARG RUST_VERSION=1.80
FROM rust:${RUST_VERSION}-slim-bullseye AS build
WORKDIR /app

# Build the application.
# Leverage a cache mount to /usr/local/cargo/registry/
# for downloaded dependencies and a cache mount to /app/target/ for 
# compiled dependencies which will speed up subsequent builds.
# Leverage a bind mount to the src directory to avoid having to copy the
# source code into the container. Once built, copy the executable to an
# output directory before the cache mounted /app/target is unmounted.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=build.rs,target=build.rs \
    --mount=type=bind,source=config,target=config \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    /bin/bash -c "set -e && cargo build --locked --release && mkdir -p /bin/server/config && cp ./target/release/quebrix /bin/server && cp ./target/release/config/config.json /bin/server/config"

################################################################################
# Create a new stage for running the application that contains the minimal
# runtime dependencies for the application.
FROM debian:bullseye-slim AS final

# Copy the executable from the "build" stage.
COPY --from=build /bin/server /bin/server

# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/develop/develop-images/dockerfile_best-practices/   #user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

RUN mkdir /bin/server/data \
  && chown -R appuser:appuser /bin/server/data \
  && mkdir /bin/server/logs \
  && chown -R appuser:appuser /bin/server/logs \
  && mkdir /bin/server/creds \
  && chown -R appuser:appuser /bin/server/creds 

USER appuser

# Expose the port that the application listens on.
EXPOSE 6022

# What the container should run when it is started.
CMD ["/bin/server/quebrix"]