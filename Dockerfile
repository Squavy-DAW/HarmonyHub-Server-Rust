FROM rust:alpine
WORKDIR /app
COPY . .

RUN cargo build --release

WORKDIR /app/target/release

EXPOSE 8000

CMD [ "./harmony_hub_server" ]