FROM rust:1.64.0 as builder

WORKDIR /usr/src/placesloth
COPY . .
RUN cargo build --release

FROM debian:buster-slim

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

RUN mkdir /app/
COPY --from=builder /usr/src/placesloth/target/release/placesloth /app/placesloth
COPY --from=builder /usr/src/placesloth/Rocket.toml /app/Rocket.toml

EXPOSE 8000
CMD ["/app/placesloth"]
