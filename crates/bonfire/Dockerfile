# Build Stage
FROM ghcr.io/onechatco/base:latest AS builder

# Bundle Stage
FROM debian:bullseye-slim
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    apt-get clean
COPY --from=builder /home/rust/src/target/release/onechatsocial-bonfire ./
EXPOSE 9000
CMD ["./onechatsocial-bonfire"]

