FROM clux/muslrust:nightly-2022-04-16 AS environment

ENV CHANNEL=nightly-2022-04-16

RUN curl "https://static.rust-lang.org/rustup/archive/${RUSTUP_VER}/${RUST_ARCH}/rustup-init" -o rustup-init \
   && chmod +x rustup-init \
   && ./rustup-init -y --default-toolchain ${CHANNEL} --profile minimal \
   && rm rustup-init \
   && ~/.cargo/bin/rustup target add x86_64-unknown-linux-musl \
   && echo "[build]\ntarget = \"x86_64-unknown-linux-musl\"" > ~/.cargo/config

FROM environment as builder

WORKDIR /usr/src

RUN cargo new locus

WORKDIR /usr/src/locus

COPY Cargo.* .

RUN cargo build --release

COPY . .

RUN --mount=type=cache,target=/usr/src/locus/target \
    --mount=type=cache,target=/root/.cargo/registry \
    cargo build --release --bin locus \
    && strip -s /usr/src/locus/target/x86_64-unknown-linux-musl/release/locus \
    && mv /usr/src/locus/target/x86_64-unknown-linux-musl/release/locus .

FROM gcr.io/distroless/static:nonroot

WORKDIR /locus

COPY --from=builder --chown=nonroot:nonroot /usr/src/locus/locus .

COPY --from=builder --chown=nonroot:nonroot /usr/src/locus/content ./content

EXPOSE 1965

ENTRYPOINT ["/locus/locus"]
