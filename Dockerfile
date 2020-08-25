  
# Build configuration
ARG project_name=maki
# Fill in name of crate^ here

# Set up rust build environment
FROM clux/muslrust:stable AS build
ARG project_name

# Create layer for the dependencies, so we don't have to rebuild them later
WORKDIR /usr/src
RUN USER=root cargo new $project_name
WORKDIR /usr/src/$project_name
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release --target x86_64-unknown-linux-musl

# Build the actual source
COPY src ./src
COPY graphql ./graphql
RUN touch ./src/main.rs && cargo build --release --target x86_64-unknown-linux-musl
RUN ls target/release
RUN find "$(pwd)"

# Create a minimal docker file with only the resulting binary
FROM alpine:latest
ARG project_name
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=build /usr/src/$project_name/target/x86_64-unknown-linux-musl/release/$project_name ./app
USER 1000
CMD ["./app"]