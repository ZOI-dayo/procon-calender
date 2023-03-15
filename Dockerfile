FROM rust:slim-buster AS builder

WORKDIR /prod
COPY Cargo.toml .
RUN mkdir ~/.cargo
RUN touch ~/.catgo/config
RUN cargo vendor > ~/.cargo/config
COPY . .
RUN cargo build --release

FROM scratch AS runner
COPY --from=builder /prod/target/release/procon-calender /bin/

ENTRYPOINT [ "/bin/procon-calender" ]
