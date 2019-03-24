FROM rust:1.33

WORKDIR /usr/src/app

RUN cargo install ficon

CMD ["ficon"]
