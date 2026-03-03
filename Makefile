.PHONY: build-backend build-lifekit run-backend test-backend test-lifekit test clean fmt

# === Build ===

build-backend:
	cd backend && cargo build

build-lifekit:
	cd lifekit && swift build

build: build-backend build-lifekit

# === Run ===

run-backend:
	cd backend && cargo run

# === Test ===

test-backend:
	cd backend && cargo test

test-lifekit:
	cd lifekit && swift test

test: test-backend test-lifekit

# === Clean ===

clean:
	cd backend && cargo clean
	cd lifekit && swift package clean

# === Format ===

fmt:
	cd backend && cargo fmt
