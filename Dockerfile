FROM rust AS builder
WORKDIR /work
COPY ./ ./
RUN cargo build -r
FROM ubuntu
RUN apt-get update && apt-get install -y gzip ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /work/target/release/cc-dl /cc-dl
WORKDIR /
RUN chmod +x /cc-dl
CMD ["/cc-dl"]