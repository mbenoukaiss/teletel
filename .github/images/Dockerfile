FROM rust:1.94-alpine AS base

ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN apk add --no-cache \
    alsa-lib-dev \
    eudev-dev \
    wayland-dev \
    pkgconf \
    musl-dev \
    gcc \
    openssl-dev \
    openssl-libs-static

RUN rustup component add clippy \
    && cargo install cargo-audit cargo-pants

FROM base AS ci-builder

WORKDIR /build

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p teletel/src \
             teletel-protocol/src \
             teletel-derive/src \
             minitel-emulator/src \
             examples/hello/src \
             examples/semigraphic/src \
             examples/load-vdt/src

COPY teletel/Cargo.toml teletel/
COPY teletel-protocol/Cargo.toml teletel-protocol/
COPY teletel-derive/Cargo.toml teletel-derive/
COPY minitel-emulator/Cargo.toml minitel-emulator/
COPY examples/hello/Cargo.toml examples/hello/
COPY examples/semigraphic/Cargo.toml examples/semigraphic/
COPY examples/load-vdt/Cargo.toml examples/load-vdt/

RUN echo "" > teletel/src/lib.rs \
    && echo "" > teletel-protocol/src/lib.rs \
    && echo "use proc_macro::TokenStream; #[proc_macro_derive(Dummy)] pub fn dummy(_: TokenStream) -> TokenStream { TokenStream::new() }" > teletel-derive/src/lib.rs \
    && echo "fn main() {}" > minitel-emulator/src/main.rs \
    && echo "fn main() {}" > examples/hello/src/main.rs \
    && echo "fn main() {}" > examples/semigraphic/src/main.rs \
    && echo "fn main() {}" > examples/load-vdt/src/main.rs

RUN cargo build --locked --workspace 2>&1 || true \
    && cargo build --locked --workspace --tests 2>&1 || true

RUN cargo clean -p teletel \
    -p teletel-protocol \
    -p teletel-derive \
    -p minitel-emulator \
    -p example-hello \
    -p example-semigraphic \
    -p example-load-vdt 2>/dev/null || true

RUN rm -rf teletel/ teletel-protocol/ teletel-derive/ minitel-emulator/ examples/ \
           Cargo.toml Cargo.lock

RUN tar -czf /target-cache.tar.gz -C /build target/

FROM base AS ci

COPY --from=ci-builder /target-cache.tar.gz /target-cache.tar.gz
