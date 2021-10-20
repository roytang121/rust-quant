FROM rust:latest

WORKDIR /app
COPY . .

RUN rustup component add rustfmt