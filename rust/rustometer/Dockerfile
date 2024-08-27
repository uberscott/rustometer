FROM rust:1.80.1

WORKDIR /usr/src/rustometer
COPY . .

RUN cargo install --path .

CMD ["rustometer"]


FROM rust:1.80.1 as build

# create a new empty shell project
RUN USER=root cargo new --bin rustometer
WORKDIR /rustometer

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/rustometer*
RUN cargo build --release

# our final base
FROM rust:1.80.1

# copy the build artifact from the build stage
COPY --from=build /rustometer/target/release/rustometer .

# set the startup command to run your binary
CMD ["./rustometer"]
