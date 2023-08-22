FROM rust:1-alpine3.18
WORKDIR app
RUN apk --update add musl-dev openssl-dev pkgconfig zlib-dev

COPY . .
RUN cargo build --release
CMD ["/app/target/release/gengo"]
