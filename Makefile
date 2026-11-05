PREFIX ?= /usr/local
BIN := target/release/translate-cli

.PHONY: all build test check size bench bench-check install clean

all: build

build:
	cargo build --release --locked

test:
	cargo test --locked

check:
	cargo fmt --check
	cargo clippy --all-targets --locked -- -D warnings

size: build
	@bytes=$$(wc -c < $(BIN) | tr -d ' '); \
	echo "binary size: $$bytes bytes"; \
	test $$bytes -lt 15728640 || { echo "error: binary exceeds 15 MB" >&2; exit 1; }

bench: build
	@python3 -c 'import subprocess, time; n = 50; t = time.perf_counter(); [subprocess.run(["./target/release/translate-cli", "El gato rápido salta"], stdout=subprocess.DEVNULL, check=True) for _ in range(n)]; print(f"average: {(time.perf_counter()-t)/n*1000:.2f} ms per run")'

bench-check: build
	@python3 -c 'import subprocess, sys, time; n = 50; budget = 50.0; t = time.perf_counter(); [subprocess.run(["./target/release/translate-cli", "El gato rápido salta"], stdout=subprocess.DEVNULL, check=True) for _ in range(n)]; average = (time.perf_counter()-t)/n*1000; print(f"average: {average:.2f} ms per run (budget: {budget:.0f} ms)"); sys.exit(0 if average < budget else f"error: average {average:.2f} ms exceeds the {budget:.0f} ms budget")'

install: build
	install -d $(DESTDIR)$(PREFIX)/bin
	install -m755 $(BIN) $(DESTDIR)$(PREFIX)/bin/translate-cli

clean:
	cargo clean
