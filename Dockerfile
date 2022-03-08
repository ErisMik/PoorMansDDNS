# Mirror All The Things Dockerfile

#### Rust Builder Image ####
FROM rust:latest AS rustbuild

RUN mkdir -p /matt
WORKDIR /matt

COPY Cargo.toml .
COPY src/ src/

RUN cargo build --release


#### Runtime Image ####
FROM nginx:latest

RUN mkdir -p /matt
WORKDIR /matt

RUN apt-get update && apt-get install -y supervisor rsync

RUN rm -rf /usr/share/nginx/html/index.html
COPY nginx.conf /etc/nginx/
COPY supervisord.conf .
COPY --from=rustbuild /matt/target/release/matt .

VOLUME /usr/share/nginx/html/mirrors
VOLUME /matt/config
EXPOSE 80

CMD ["supervisord", "-c", "/matt/supervisord.conf"]
