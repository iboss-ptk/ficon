FROM rust:1.34.2 as build

# create a new empty shell project
RUN USER=root cargo new --bin ficon
WORKDIR /app

COPY . /app

RUN cargo build --release
RUN rm src/*.rs

# our final base
FROM debian:stretch-slim

# copy the build artifact from the build stage
COPY --from=build /app/target/release/ficon .

# set the startup command to run your binary
CMD ["./ficon"]
