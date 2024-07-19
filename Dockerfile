# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.78

FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION}-slim-bookworm AS build

WORKDIR /app

RUN apt-get update -y && apt-get install -y libssl-dev pkg-config

COPY . .
RUN cargo build --locked --release --target-dir target && cp ./target/release/prodzilla /bin/prodzilla

FROM debian:bookworm-slim AS final

RUN apt-get update && apt-get install -y libssl-dev ca-certificates
# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/go/dockerfile-user-best-practices/
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser


# Copy the executable from the "build" stage.
COPY --from=build /bin/prodzilla /bin/

# Expose the port that the application listens on.
EXPOSE 3000

# What the container should run when it is started.
ENTRYPOINT ["/bin/prodzilla"]
