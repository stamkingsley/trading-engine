FROM rust:1.70-slim as builder

WORKDIR /app
COPY . .

# 安装FlatBuffers编译器
RUN apt-get update && apt-get install -y \
    flatbuffers-compiler \
    && rm -rf /var/lib/apt/lists/*

# 构建应用
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/trading-engine /usr/local/bin/

EXPOSE 8080 9090

CMD ["trading-engine"]