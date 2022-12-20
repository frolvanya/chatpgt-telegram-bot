FROM frolvlad/alpine-rust as builder

# create a new empty shell project
WORKDIR /chatgpt-telegram-bot

# copy over manifests
COPY Cargo.toml .
COPY Cargo.lock .

RUN mkdir src
RUN touch src/main.rs
RUN echo "fn main() {}" > src/main.rs

RUN cargo test
RUN cargo build --release

COPY . .
RUN touch src/main.rs

RUN cargo test
RUN cargo build --release

RUN strip target/release/chatgpt-telegram-bot

# start building the final image
FROM alpine:3.17

COPY --from=builder /chatgpt-telegram-bot/target/release/chatgpt-telegram-bot .
COPY log_config.yaml .

ENTRYPOINT ["/entrypoint.sh"]
