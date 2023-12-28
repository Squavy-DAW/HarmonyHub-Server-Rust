FROM rust:alpine
WORKDIR /app
COPY . .

RUN apk add --no-cache musl-dev
RUN cargo build --release
COPY .env.production /app/target/release/.env.production

WORKDIR /app/target/release
EXPOSE 8000
CMD [ "./harmony_hub_server" ]