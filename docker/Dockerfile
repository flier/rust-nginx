# syntax=docker/dockerfile:1

ARG UBUNTU_VERSION=22.04
ARG NGINX_VERSION=1.24.0
ARG NGINX_OPTS="--with-compat --with-debug --with-http_ssl_module --with-stream"

FROM ubuntu:${UBUNTU_VERSION} as nginx

# Update and install required dependencies

ENV DEBIAN_FRONTEND=noninteractive

RUN rm -f /etc/apt/apt.conf.d/docker-clean; \
    echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update -yq && \
    apt-get install -yq \
    build-essential \
    ca-certificates \
    curl \
    pgpgpg \
    libpcre2-dev \
    libssl-dev \
    wget \
    zlib1g-dev

# Download and verify nginx source archive file

ARG NGINX_VERSION
ENV NGINX_VERSION=${NGINX_VERSION}

ADD https://nginx.org/download/nginx-${NGINX_VERSION}.tar.gz /
ADD https://nginx.org/download/nginx-${NGINX_VERSION}.tar.gz.asc /

ENV GPG_KEY_SERVER=keyserver.ubuntu.com
ENV GPG_KEY_ID=A0EA981B66B0D967

RUN gpg --keyserver ${GPG_KEY_SERVER} --recv-keys ${GPG_KEY_ID} && \
    gpg --list-packets /nginx-${NGINX_VERSION}.tar.gz.asc && \
    gpg --verify /nginx-${NGINX_VERSION}.tar.gz.asc /nginx-${NGINX_VERSION}.tar.gz

# Build nginx from source

WORKDIR /

RUN tar xzvf /nginx-${NGINX_VERSION}.tar.gz

WORKDIR /nginx-${NGINX_VERSION}

ARG NGINX_OPTS
ENV NGINX_OPTS=${NGINX_OPTS}

RUN ./configure ${NGINX_OPTS} && make && make install

FROM rust:1-bookworm as base

# Copy nginx from build stage

ARG NGINX_VERSION
ENV NGINX_VERSION=${NGINX_VERSION}

COPY --link --from=nginx /nginx-${NGINX_VERSION} /nginx-${NGINX_VERSION}

# Setup cargo to use mirror

ENV CARGO_HOME=/root/.cargo

COPY ./docker/cargo/config.toml ${CARGO_HOME}/

RUN cargo --version && rustc --version

# Build and test rust-nginx

ADD . /rust-nginx

ENV RUST_BACKTRACE=full
ENV RUST_LOG=debug
ENV NGINX_DIR=/nginx-${NGINX_VERSION}

# Build with static-link feature

WORKDIR /rust-nginx/

ENV CARGO_HOME=/root/.cargo

RUN --mount=type=cache,target=${CARGO_HOME}/registry,sharing=locked \
    --mount=type=cache,target=/rust-nginx/target,sharing=locked \
    cargo --version && rustc --version && \
    cargo build -v --features static-link && \
    cargo test -v --features static-link

# FROM base as examples

# Build examples

WORKDIR /rust-nginx/ngx-mod

ENV CARGO_HOME=/root/.cargo
ENV NGINX_PREFIX=/usr/local/nginx

COPY --link --from=nginx ${NGINX_PREFIX}  ${NGINX_PREFIX}

RUN --mount=type=cache,target=${CARGO_HOME}/registry,sharing=locked \
    --mount=type=cache,target=/rust-nginx/target,sharing=locked \
    cargo --version && rustc --version && \
    cargo build -v --examples && \
    mkdir -p ${NGINX_PREFIX}/modules && \
    cp /rust-nginx/target/debug/examples/*.so ${NGINX_PREFIX}/modules/ && \
    mkdir -p ${NGINX_PREFIX}/etc && \
    cp /rust-nginx/ngx-mod/examples/*.conf ${NGINX_PREFIX}/etc/

# FROM ubuntu:${UBUNTU_VERSION} as nginx

ENV NGINX_PREFIX=/usr/local/nginx

# COPY --link --from=build ${NGINX_PREFIX}  ${NGINX_PREFIX}

ADD docker/nginx/nginx.conf ${NGINX_PREFIX}/conf

CMD [ "${NGINX_PREFIX}/sbin/nginx" ]

FROM python:3-bookworm as integration-tests

ENV NGINX_PREFIX=/usr/local/nginx

COPY --link --from=base ${NGINX_PREFIX}  ${NGINX_PREFIX}

RUN --mount=type=cache,target=/root/.cache/pip \
    pip install --upgrade pip && \
    python --version && pip --version

COPY ngx-mod/examples/requirements.txt /

RUN --mount=type=cache,target=/root/.cache/pip \
    pip install -q -r /requirements.txt

WORKDIR /tests

COPY ngx-mod/examples/*.robot /tests/
COPY ngx-mod/examples/*.resource /tests/

CMD ["robot", "/tests"]
