FROM rust:latest as builder

RUN update-ca-certificates

# ENV USER=locus
# ENV UID=10001

# RUN adduser \
#     --disabled-password \
#     --gecos "" \
#     --home "/nonexistent" \
#     --shell "/sbin/nologin" \
#     --no-create-home \
#     --uid "${UID}" \
#     "${USER}"

WORKDIR /locus

COPY ./ ./

RUN cargo build --release

RUN strip -s /locus/target/release/locus

FROM debian:buster-slim

RUN apt-get update && apt-get install -y libssl-dev

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /locus

COPY --from=builder /locus/target/release/locus ./
COPY --from=builder /locus/content ./content

# USER locus:locus

EXPOSE 1965

CMD ["./locus"]
