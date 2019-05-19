FROM rust:1.34.2 as build

WORKDIR /app

# create a new empty shell project
RUN USER=root cargo new --bin ficon
RUN mv ficon/* . && rm -rf ficon

# copy only manifest file and lock
COPY ./Cargo.lock .
COPY ./Cargo.toml .

# build and cache depedencies so if the depedencies doesn't change
# there's no need to rebuild them
RUN cargo build --release

# build the app
COPY ./src ./src
RUN cargo build --release

RUN rm src/*.rs

# our final base
FROM debian:stretch-slim

WORKDIR /app

# copy the build artifact from the build stage
COPY --from=build /app/target/release/ficon .

RUN mv ficon /usr/local/bin

CMD ["ficon"]
