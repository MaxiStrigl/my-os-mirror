BOOTLOADER_TARGET := x86_64-unknown-uefi
KERNEL_TARGET := "kernel/x86_64-my-os.json"

# Paths
BOOTLOADER_BIN := target/$(BOOTLOADER_TARGET)/$(PROFILE)/bootloader.efi


.PHONY: all build_bootloader run clean

all: build_bootloader build_kernel

build_bootloader:
	cargo build -p bootloader --target $(BOOTLOADER_TARGET)

build_kernel:
	cargo build -p kernel --target $(KERNEL_TARGET)

run: all
	python3 runner.py

clean:
	cargo clean
	rm -rf esp/
