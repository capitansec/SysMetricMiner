FROM rust:latest as builder

WORKDIR /usr/src/myapp

COPY Cargo.toml Cargo.lock ./

COPY src ./src

RUN cargo build --release

FROM rust:slim

RUN apt update && apt install -y glibc-source

WORKDIR /root/

COPY .env .

COPY --from=builder /usr/src/myapp/target/release/SysMetricMiner .

CMD ["./SysMetricMiner"]