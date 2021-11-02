FROM rust:1.56-slim-bullseye as rust
WORKDIR /src
RUN apt-get update && apt-get install -y git pkg-config libssl-dev
RUN mkdir src && echo "fn main() {}" > src/main.rs
COPY Cargo.toml .
RUN sed -i '/.*build.rs.*/d' Cargo.toml
COPY Cargo.lock .
RUN cargo build --release
COPY . /src
RUN cargo build --release

FROM debian:bullseye-slim
RUN useradd -ms /bin/bash -u 1001 libmedium
RUN apt-get update && apt-get install -y ca-certificates
RUN mkdir /var/lib/libmedium && chown libmedium /var/lib/libmedium
COPY --from=rust /src/target/release/libmedium /usr/local/bin/
USER libmedium
ENTRYPOINT ["/usr/local/bin/libmedium"]
