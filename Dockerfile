FROM rust:1.68-alpine3.17 AS builder

ARG BUILDARCH
ARG TARGETARCH=${BUILDARCH}
ENV TARGETARCH=${TARGETARCH}
WORKDIR /build

COPY docker-target.sh .

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV RUST_BACKTRACE=1
RUN apk add musl-dev git openssl-dev openssl perl make gcc
RUN ls /usr/bin
RUN ln -s "/usr/bin/$(sh ./docker-target.sh)-alpine-linux-musl-gcc" /usr/bin/musl-gcc
RUN rustup target add "$(sh ./docker-target.sh)-unknown-linux-musl"

# Dummy file exists to give Cargo something to compile while it downloads dependencies.
RUN echo "fn main() {}" > dummy.rs
COPY Cargo.toml Cargo.lock ./
# Tell Cargo to build the dummy file instead of the real app
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release --target "$(sh ./docker-target.sh)-unknown-linux-musl"

# Tell Cargo to build the real app now
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml
COPY . .
# RUN touch -a -m "./src/main.rs"

ENV RUSTFLAGS=-Clinker=rust-lld
RUN cargo build --release --target "$(sh ./docker-target.sh)-unknown-linux-musl"

FROM alpine:3.17

WORKDIR /bs
RUN apk add dumb-init

COPY --from=builder /build/target/*-unknown-linux-musl/release/bandsnatch .

ENTRYPOINT [ "dumb-init", "--" ]
CMD [ "./bandsnatch", "--version" ]
