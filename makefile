# Variables
BINARY_NAME = r_tvui
TARGET ?= x86_64-unknown-linux-gnu

.PHONY: all build run test check clean fmt lint help coverage coverage-html

## all: Default target to format, lint, and build the project
all: fmt lint build

prepare: check fmt lint

## build: Build the release binary using cargo
build:
	echo "Building release binary..."
	cargo build --bin r_tvui

release:
	echo "Building release binary..."
	cargo build --release --target $(TARGET)

## run: Run the debug binary
run: prepare
	cargo run

## test: Run all tests
test: prepare
	cargo test --workspace --all-targets

# Pure I/O shells excluded from coverage: program bootstrap, the blocking
# terminal event loop, and the OS process-spawn wrapper. See docs/test.md.
COVERAGE_IGNORE = 'src/(main\.rs|events/app\.rs|os/mod\.rs)'

## coverage: Print a line/region coverage summary for the whole workspace
coverage:
	cargo llvm-cov --workspace --summary-only --ignore-filename-regex $(COVERAGE_IGNORE)

## coverage-html: Generate an HTML coverage report and open it
coverage-html:
	cargo llvm-cov --workspace --html --open --ignore-filename-regex $(COVERAGE_IGNORE)

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
