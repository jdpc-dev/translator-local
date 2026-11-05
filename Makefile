PREFIX ?= /usr/local
BIN := target/release/translate-cli

.PHONY: all build test check size bench install clean

all: build

build:
	cargo build --release

test:
	cargo test

check:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

size: build
	@bytes=$$(stat -c %s $(BIN)); \
	echo "binary size: $$bytes bytes"; \
	test $$bytes -lt 15728640 || { echo "error: binary exceeds 15 MB" >&2; exit 1; }

bench: build
	@python3 -c 'import subprocess, time; n = 50; t = time.perf_counter(); [subprocess.run(["./target/release/translate-cli", "El gato rápido salta"], stdout=subprocess.DEVNULL, check=True) for _ in range(n)]; print(f"average: {(time.perf_counter()-t)/n*1000:.2f} ms per run")'

install: build
	install -Dm755 $(BIN) $(DESTDIR)$(PREFIX)/bin/translate-cli

clean:
	cargo clean
