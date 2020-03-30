FROM rust:1.42

WORKDIR /usr/src/myapp

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY .env .env
COPY ./src ./src

RUN cargo build --release

CMD ["myapp"]