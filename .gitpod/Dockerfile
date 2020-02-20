FROM gitpod/workspace-full:latest

USER root
RUN apt-get install -yq \
        musl \
        musl-dev \
        musl-tools \
        libssl-dev \
    && apt-get clean && rm -rf /var/lib/apt/lists/* /tmp/*

# Build a static OPENSSL from source to compile singular against it
# From: https://github.com/fornwall/rust-static-builder/blob/master/Dockerfile
# -DOPENSSL_NO_SECURE_MEMORY needed due to https://github.com/openssl/openssl/issues/7207
RUN cd /tmp && OPENSSL_VERSION=1.1.1a && \
    curl -LO "https://www.openssl.org/source/openssl-$OPENSSL_VERSION.tar.gz" && \
    tar xf "openssl-$OPENSSL_VERSION.tar.gz" && cd "openssl-$OPENSSL_VERSION" && \
    env CC=musl-gcc ./Configure \
        no-shared no-zlib no-engine no-unit-test -DOPENSSL_NO_SECURE_MEMORY \
        -fPIC --prefix=/usr/local/musl linux-x86_64 && \
    env C_INCLUDE_PATH=/usr/local/musl/include/ make depend && \
    make install_sw

ENV OPENSSL_DIR=/usr/local/musl/
ENV OPENSSL_INCLUDE_DIR=/usr/local/musl/include/
ENV OPENSSL_LIB_DIR=/usr/local/musl/lib/
ENV OPENSSL_STATIC=1

