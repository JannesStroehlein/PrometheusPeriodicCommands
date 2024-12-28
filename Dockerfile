FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Notice that we are specifying the --target flag!
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin PrometheusPeriodicCommands

FROM alpine AS runtime
RUN addgroup -S ppc && adduser -S ppc -G ppc
RUN apk add openrc openssh
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/PrometheusPeriodicCommands /usr/local/bin/
USER ppc

LABEL org.opencontainers.image.title="Prometheus Periodic Commands"
LABEL org.opencontainers.image.url="https://github.com/JannesStroehlein/PrometheusPeriodicCommands"
LABEL org.opencontainers.image.description="This container can be configured to run specific commands in an intervall, parse their output and expose the parsed values as numeric gauges to Prometheus."
LABEL org.opencontainers.image.documentation="https://github.com/JannesStroehlein/PrometheusPeriodicCommands/README.md"
LABEL org.opencontainers.image.source="https://github.com/JannesStroehlein/PrometheusPeriodicCommands"
LABEL org.opencontainers.image.authors="jannes@j3s.dev"
ENTRYPOINT ["/usr/local/bin/PrometheusPeriodicCommands"]