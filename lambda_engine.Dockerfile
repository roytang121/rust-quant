FROM rust:1.55.0

WORKDIR /app

RUN cargo install miniserve
CMD miniserve .
#COPY . .

# RUN rm -rf environments

#RUN cargo install --path .

# CMD RUST_LOG=INFO ENV=development ./target/release/luban ftx swap-mm-eth