BOOTLOADER_TARGET := x86_64-unknown-uefi

# Paths
BOOTLOADER_BIN := target/$(BOOTLOADER_TARGET)/$(PROFILE)/bootloader.efi


.PHONY: all build_bootloader run clean

all: build_bootloader

build_bootloader:
	cargo build -p bootloader --target $(BOOTLOADER_TARGET)

run: all
	python3 runner.py

clean:
	cargo clean
	rm -rf esp/
