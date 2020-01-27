FROM rust:1.40-buster as builder

ADD . ./

RUN apt-get update && \
    apt-get install -y libssl-dev && \
    cargo build --verbose --release && \
    cargo install --path .

FROM debian:buster
COPY --from=builder /usr/local/cargo/bin/command_line_worker /usr/bin

RUN apt update && apt install -y libssl1.1 ca-certificates

ENV AMQP_QUEUE job_command_line
CMD command_line_worker
