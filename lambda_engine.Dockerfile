FROM rust:1.55.0

WORKDIR /app
COPY . .

# RUN rm -rf environments

RUN cargo install --path .

# CMD RUST_LOG=INFO ENV=development ./target/release/luban ftx swap-mm-eth