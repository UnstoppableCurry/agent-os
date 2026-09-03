.PHONY: build-backend build-lifekit run-backend test-backend test-lifekit test-pages test clean fmt

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

test-pages:
	python3 scripts/check_pages_site.py

test: test-backend test-lifekit test-pages

# === Clean ===

clean:
	cd backend && cargo clean
	cd lifekit && swift package clean

# === Format ===

fmt:
	cd backend && cargo fmt
