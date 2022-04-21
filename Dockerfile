FROM rust:latest AS builder

RUN update-ca-certificates

# Create appuser
ENV USER=newsletter-signup-service
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /newsletter-signup-service

COPY ./ .

ENV SQLX_OFFLINE true
RUN cargo build --release

######################
FROM ubuntu:latest as newsletter-signup-service

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /newsletter-signup-service

# Copy our build
COPY --from=builder /newsletter-signup-service/target/release/newsletter-signup-service ./
COPY --from=builder /newsletter-signup-service/configuration ./configuration

# Use an unprivileged user.
USER newsletter-signup-service:newsletter-signup-service

EXPOSE 8000
ENV APP_ENVIRONMENT production

CMD ["/newsletter-signup-service/newsletter-signup-service"]
