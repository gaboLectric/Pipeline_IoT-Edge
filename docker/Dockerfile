FROM rust:alpine AS builder

# Instalamos las librerías estáticas de SSL y herramientas de compilación
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    build-base

WORKDIR /usr/src/app
COPY rust/ .

# Compilar el Workspace completo
RUN cargo build --release

FROM alpine:latest
# Instalamos dependencias mínimas de ejecución
RUN apk add --no-cache libgcc libstdc++ openssl ca-certificates

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/app/target/release/sensor .
COPY --from=builder /usr/src/app/target/release/edge .
COPY --from=builder /usr/src/app/target/release/coordinator .