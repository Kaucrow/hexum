# =======================
#   Builder
# =======================
FROM rust:1.81-alpine AS builder
WORKDIR /usr/src/app

RUN apk add --no-cache \
    musl-dev \
    build-base \
    postgresql-dev

COPY . .

RUN cargo build --release

# =======================
#   Runtime environment
# =======================
FROM alpine:latest
WORKDIR /app

RUN apk add --no-cache \
    libpq \
    redis \
    ca-certificates

COPY --from=builder /usr/src/app/target/release/hexum /app/hexum

COPY --from=builder /usr/src/app/config /app/config
COPY --from=builder /usr/src/app/postgres /app/postgres

CMD ["/app/hexum"]