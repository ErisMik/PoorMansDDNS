# Poorman's DDNS Dockerfile

#### Rust Builder Image ####
FROM rust:latest AS rustbuild

RUN mkdir -p /pmddns
WORKDIR /pmddns

COPY Cargo.toml .
COPY src/ src/

RUN cargo build --release


#### Runtime Image ####
FROM debian:11-slim

RUN mkdir -p /pmddns
WORKDIR /pmddns

RUN apt-get update && apt-get install -y supervisor

COPY supervisord.conf .
COPY --from=rustbuild /pmddns/target/release/poormans-ddns .

VOLUME /pmddns/config
CMD ["supervisord", "-c", "/pmddns/supervisord.conf"]
