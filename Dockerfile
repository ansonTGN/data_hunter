FROM node:20-slim AS frontend-builder
WORKDIR /app/web
COPY web/package.json ./
RUN npm install
COPY web/ .
RUN npm run build

FROM rust:1.84-slim-bookworm AS backend-builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates
RUN cargo new --bin hunter_app
WORKDIR /app/hunter_app
COPY Cargo.toml .
RUN cargo build --release && rm src/*.rs
COPY --from=frontend-builder /app/web/dist ./ui_dist
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=backend-builder /app/hunter_app/target/release/hunter_app ./data_hunter
EXPOSE 3000
CMD ["./data_hunter"]
