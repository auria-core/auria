FROM rust:1.78-slim AS build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN useradd -m -u 10001 auria
WORKDIR /home/auria
COPY --from=build /app/target/release/auria /usr/local/bin/auria
USER auria
ENV RUST_LOG=info
EXPOSE 8787
CMD ["auria", "serve", "--bind", "0.0.0.0:8787"]
