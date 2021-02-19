  
# Build configuration
ARG project_name=maki
# Fill in name of crate^ here

# Set up rust build environment
FROM rust:latest as builder
ARG project_name

# install lib/s needed by Songbird
RUN apt-get update && apt-get install -y libopus-dev

# Create layer for the dependencies, so we don't have to rebuild them later
WORKDIR /usr/src
RUN USER=root cargo new $project_name
WORKDIR /usr/src/$project_name
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
RUN rm src/*.rs

# Build the actual source
COPY src ./src
COPY graphql ./graphql
COPY sqlx-data.json ./sqlx-data.json
RUN touch ./src/main.rs && cargo build --release

# Create a "minimal" docker file, using buster as we need it for voice
FROM debian:buster-slim
ARG project_name
RUN apt-get update \
    && apt-get install -y ca-certificates libopus-dev ffmpeg sudo python3-pip \
    && rm -rf /var/lib/apt/lists/*
RUN pip3 install youtube_dl
COPY --from=builder /usr/src/$project_name/target/release/$project_name ./app
USER 1000
CMD ["./app"]
