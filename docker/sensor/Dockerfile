FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static build-base
WORKDIR /usr/src/app
COPY rust/ .
RUN cargo build --release --bin sensor

FROM alpine:latest
RUN apk add --no-cache libgcc libstdc++ openssl ca-certificates bash
WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/sensor .
COPY docker/sensor/entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/entrypoint.sh

ENTRYPOINT ["entrypoint.sh"]