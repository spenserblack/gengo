FROM rust:1-alpine3.19
WORKDIR app
COPY . .
RUN apk --update add cmake make musl-dev pkgconfig && \
    cargo build --release && \
    cp target/release/gengo /usr/local/bin/gengo && \
    rm -rf target

ENTRYPOINT ["gengo"]
