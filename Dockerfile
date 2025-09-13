# Stage 1: build via npm
FROM node:20 AS build
WORKDIR /app
COPY package*.json ./
COPY postcss.config.mjs ./
COPY tsconfig.json ./
RUN npm install
COPY web/src/ web/src/
RUN npm run build

# Stage 2: rust application
FROM rust:latest AS rust-build
WORKDIR /app
COPY --from=build /app/web/out ./web/out
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release
RUN strip target/release/indrive
RUN mv target/release/indrive ./indrive
RUN rm -rf src target
RUN chmod +x ./indrive
EXPOSE 8080
CMD ["./indrive"]