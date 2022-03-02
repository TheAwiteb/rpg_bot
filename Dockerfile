FROM rust:latest

WORKDIR /usr/src/rpg_bot

RUN cargo install diesel_cli
RUN apt install -y libsqlite3-dev

COPY . .

RUN diesel migration run
RUN cargo build --release

CMD ["cargo", "run", "--release"]
