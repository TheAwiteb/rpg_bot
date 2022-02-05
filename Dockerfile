FROM rust:latest

WORKDIR /usr/src/rpg_bot

COPY . .

RUN cargo build --release

CMD ["cargo", "run", "--release"]
