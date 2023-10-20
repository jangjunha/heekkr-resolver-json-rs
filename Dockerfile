FROM rust:1-alpine as builder

WORKDIR /app/

RUN apk add musl-dev protoc

COPY . .
RUN cargo install --path .

FROM alpine:3
COPY --from=builder /usr/local/cargo/bin/heekkr-resolver-json-rs /usr/local/bin/heekkr-resolver-json-rs

CMD ["heekkr-resolver-json-rs", "serve", "0.0.0.0:50051"]
