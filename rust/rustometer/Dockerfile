FROM rust:1.59

WORKDIR /usr/src/rustometer
COPY . .

RUN cargo install --path .

CMD ["rustometer"]

