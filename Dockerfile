FROM rust:1.61

RUN cargo install .

CMD ["./target/release/nu-discord-bot"]
