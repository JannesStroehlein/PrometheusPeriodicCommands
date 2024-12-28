####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=ppc
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /PrometheusPeriodicCommands

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /PrometheusPeriodicCommands

# Copy our build
COPY --from=builder /PrometheusPeriodicCommands/target/x86_64-unknown-linux-musl/release/PrometheusPeriodicCommands ./

# Use an unprivileged user.
USER ppc:ppc

CMD ["/PrometheusPeriodicCommands/PrometheusPeriodicCommands"]