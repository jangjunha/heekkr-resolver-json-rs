FROM rust:1-alpine as builder

WORKDIR /app/

RUN apk add musl-dev protoc

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo install --path .

FROM alpine:3
COPY --from=builder /usr/local/cargo/bin/heekkr-resolver-json-rs /usr/local/bin/heekkr-resolver-json-rs

CMD ["heekkr-resolver-json-rs", "serve", "0.0.0.0:50051"]
