# Kernel

This directory contains the kernel of myOS. (name tbd)

## Overview
The kernel is designed to be loaded by a UEFI bootloader. It runs in 64bit long mode and currently implements a very basic framebuffer to verify execution.

## Entry Point

The entry point is defined in `src/main.rs` as _start 

```rust
extern "win64" fn _start(info: &'static BootInfo) -> !
```

- **ABI:** It uses `extern win64` to match the calling convention used by the UEFI bootloader. (common in UEFI environments)
- **Parameters:** The function receives a `static` reference to `BootInfo`, which contains critical system information like framebuffer data.

## Dependencies 

- `common`: A shared library containing structures like `BootInfo` that are used by both the bootloader and the kernel to ensure type-safe handover.

## Building

The kernel is build using the custom target.
```bash
cargo build --target kernel/x86_64-my-os.json
#! or in the project root
make build_kernel
```
