FROM rustlang/rust:nightly AS builder
MAINTAINER Kyle Cotton <kyle@cybersaur.io>
WORKDIR /usr/src/sdp-backend
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian10
MAINTAINER Kyle Cotton <kyle@cybersaur.io>
EXPOSE 8000
COPY --from=builder /usr/src/sdp-backend/target/release/sdp-backend /
COPY --from=builder /usr/src/sdp-backend/static /static

CMD ["./sdp-backend"]
