# Variables
BINARY_NAME = r_tvui
TARGET ?= x86_64-unknown-linux-gnu

.PHONY: all build run test check clean fmt fmt-fix lint help coverage coverage-html prepare

## all: Format (apply), lint, and build
all: fmt-fix lint build

## prepare: Check, format (apply), and lint before run/test
prepare: check fmt-fix lint

## build: Build the debug binary
build:
	echo "Building binary..."
	cargo build --bin r_tvui

## release: Build the release binary for TARGET
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

## check: Fast compile check
check:
	cargo check --workspace --all-targets

## clean: Remove build artifacts
clean:
	echo "Cleaning up..."
	cargo clean

## fmt: Check formatting (CI gate — does not write)
fmt:
	cargo fmt --all -- --check

## fmt-fix: Apply rustfmt
fmt-fix:
	cargo fmt --all

## lint: Clippy with warnings denied
lint:
	cargo clippy --workspace --all-targets -- -D warnings

## help: Show this help message
help:
	echo "Usage: make [target]"
	echo ""
	echo "Targets:"
	fgrep -h "##" $(MAKEFILE_LIST) | fgrep -v fgrep | sed -e 's/\\$$//' | sed -e 's/##//'
