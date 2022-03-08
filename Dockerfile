FROM rust:1.59 as build

# Create a new project in order to avoid building dependencies every time
RUN USER=root cargo new --bin pub-sub-rust
WORKDIR /pub-sub-rust

# Copy cargo files to new project
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build step to cache dependencies, as long as the cargo files above have not changed
RUN cargo build --release
RUN rm src/*.rs

# Copy source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/pub-sub-rust*
RUN cargo build --release

FROM debian:buster-slim

# copy the build artifact from the build stage
COPY --from=build /pub-sub-rust/target/release/pub-sub-rust .

CMD ["./pub-sub-rust"]