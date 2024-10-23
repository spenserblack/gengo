FROM rust:1-alpine3.19 AS builder
WORKDIR /app
COPY . .
RUN apk --update add cmake make musl-dev pkgconfig && \
    cargo build --release

FROM alpine:3 AS runtime
COPY --from=builder /app/target/release/gengo /usr/local/bin/gengo

ENTRYPOINT ["/usr/local/bin/gengo"]
