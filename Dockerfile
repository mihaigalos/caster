FROM rust:alpine3.14 as base
RUN apk update \
    && apk add \
        git \
        gcc \
        g++ \
        openssl \
        openssl-dev \
        pkgconfig

COPY . /src

RUN cd /src && \
    sed -i -e "s/openssl .*//" Cargo.toml && \
    RUSTFLAGS="-C target-feature=-crt-static" cargo build --release

FROM alpine:3.17 as tool

RUN apk update && apk add \
    curl \
    jq \
    libgcc

COPY --from=base /src/target/release/caster /usr/local/bin

ENTRYPOINT [ "caster" ]
CMD [ "--" ]
