# Variables
BINARY_NAME=r_tvui

.PHONY: all build run test check clean fmt lint help

## all: Default target to format, lint, and build the project
all: fmt lint build

prepare: check fmt lint

## build: Build the release binary using cargo
build: prepare test
	echo "Building release binary..."
	cargo build

release: prepare test
	echo "Building release binary..."
	cargo build --release

## run: Run the debug binary
run: prepare
	cargo run

## test: Run all tests
test: prepare
	cargo test --workspace --all-targets

## check: Fast check to verify code compiles without generating binaries
check:
	cargo check --workspace --all-targets

## clean: Remove build artifacts and target folder
clean:
	echo "Cleaning up..."
	cargo clean

## fmt: Automatically format code with rustfmt
fmt:
	cargo fmt --all

## lint: Check for code smells and lints using clippy
lint:
	cargo clippy --workspace --all-targets

## help: Show this help message
help:
	echo "Usage: make [target]"
	echo ""
	echo "Targets:"
	fgrep -h "##" $(MAKEFILE_LIST) | fgrep -v fgrep | sed -e 's/\\$$//' | sed -e 's/##//'
