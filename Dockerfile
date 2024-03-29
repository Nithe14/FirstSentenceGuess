FROM rust:1.77.0-slim as builder

WORKDIR /usr/src/fsg
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build  --release

FROM gcr.io/distroless/cc-debian12
WORKDIR /
COPY --from=builder /usr/src/fsg/target/release/fsg ./
COPY ./config.toml ./config.toml
COPY ./demo-db.json ./demo-db.json
COPY ./static ./static
COPY ./templates ./templates

CMD ["./fsg"]
