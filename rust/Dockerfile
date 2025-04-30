FROM clux/muslrust:stable AS build

WORKDIR /usr/share/app

COPY Cargo.lock Cargo.toml ./

RUN mkdir src &&  \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release || true && \
    rm -r src

COPY . .

RUN cargo build --release

FROM scratch

COPY --from=build /usr/share/app/target/x86_64-unknown-linux-musl/release/speer-test ./web

ENTRYPOINT ["./web"]