# Dockerfile for creating a statically-linked Rust application using docker's
# multi-stage build feature.
COPY ./ ./
RUN rustup target install x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN mkdir -p /build-out && cp target/x86_64-unknown-linux-musl

FROM scratch
WORKDIR /opt/maki-bot
COPY --from=build /buildout/maki-bot /

ENTRYPOINT ["/maki-bot"]
