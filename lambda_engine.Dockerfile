FROM rust:latest

WORKDIR /app
COPY . .

RUN rustup component add rustfmt
# RUN cargo install --path .

# RUN cargo install miniserve
# CMD miniserve .
#COPY . .

# RUN rm -rf environments

#RUN cargo install --path .

# CMD RUST_LOG=INFO ENV=development ./target/release/luban ftx swap-mm-eth