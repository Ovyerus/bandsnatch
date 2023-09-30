FROM rust:1.70-alpine3.17 AS builder

ARG TARGETARCH
ENV TARGETARCH=${TARGETARCH}
WORKDIR /build

COPY docker-target.sh .

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV RUST_BACKTRACE=1

RUN apk add musl-dev git openssl-dev openssl perl make gcc
RUN ln -s "/usr/bin/$(sh ./docker-target.sh)-alpine-linux-musl-gcc" /usr/bin/musl-gcc
RUN rustup target add "$(sh ./docker-target.sh)-unknown-linux-musl"

COPY . .

ENV RUSTFLAGS=-Clinker=rust-lld
RUN cargo build --release --target "$(sh ./docker-target.sh)-unknown-linux-musl"

FROM alpine:3.17

WORKDIR /bs
RUN apk add dumb-init

COPY --from=builder /build/target/*-unknown-linux-musl/release/bandsnatch .

ENTRYPOINT [ "dumb-init", "--" ]
CMD [ "./bandsnatch", "--version" ]
