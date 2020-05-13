FROM ubuntu:18.04 AS rust_builder

ENV DEBIAN_FRONTEND=noninteractive

ENV RUST_ARCHIVE=rust-1.43.0-x86_64-unknown-linux-gnu.tar.gz
ENV RUST_DOWNLOAD_URL=https://static.rust-lang.org/dist/$RUST_ARCHIVE

RUN apt-get update && \
    apt-get install \
       ca-certificates \
       curl \
       g++ \
       -qqy \
       --no-install-recommends && \
    rm -rf /var/lib/apt/lists/* && \
    curl -fsOSL $RUST_DOWNLOAD_URL && \
    curl -s $RUST_DOWNLOAD_URL.sha256 | sha256sum -c - && \
    tar -xzf $RUST_ARCHIVE --strip-components=1 && \
    rm $RUST_ARCHIVE && \
    ./install.sh --prefix=/dist

FROM ubuntu:18.04 AS worker_builder

COPY --from=rust_builder /dist/ /usr/local/

ADD . /app

WORKDIR /app

RUN apt-get update && \
    apt-get install \
       git \
       ca-certificates \
       g++ \
       openssl \
       libssl-dev \
       pkg-config \
       -qqy \
       --no-install-recommends && \
    rm -rf /var/lib/apt/lists/* && \
    cargo build --verbose --release && \
    cargo install --path .

FROM ubuntu:18.04

COPY --from=worker_builder /root/.cargo/bin/command_line_worker /usr/bin

RUN apt-get update && \
    apt-get install \
       -qqy \
       libssl1.1 \
       ca-certificates

ENV AMQP_QUEUE job_command_line
CMD command_line_worker
