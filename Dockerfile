FROM frolvlad/alpine-rust as builder

RUN apk add --no-cache openssl-dev

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

RUN apk add --no-cache bash libssl3 libgcc

COPY --from=builder /chatgpt-telegram-bot/target/release/chatgpt-telegram-bot .
COPY log_config.yaml .

CMD ["/chatgpt-telegram-bot"]
