FROM rust:bookworm

RUN apt-get update && \
	apt-get upgrade

RUN rustup update && \
	rustup target add thumbv6m-none-eabi && \
	rustup toolchain add nightly && \
	rustup component add \
	clippy \
	rustfmt

RUN cargo install \
	cargo-make \
	mdbook \
	mdbook-mermaid

