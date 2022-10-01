FROM rust:1.64.0 as builder

WORKDIR /usr/src/placesloth
COPY . .
RUN cargo build --release

FROM debian:buster-slim

COPY --from=builder /usr/src/placesloth/target/release/placesloth /usr/local/bin/placesloth
EXPOSE 8000
CMD ROCKET_PORT=8000 /usr/local/bin/placesloth 
