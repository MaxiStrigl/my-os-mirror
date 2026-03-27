# Bootloader

The bootloader is an UEFI application responsible for loading the kernel ELF, setting up the system and finally handing over control.

## Overview

It's primary tasks are:
1. Initialize UEFI services.
2. Locate and reading the `kernel.elf` from the file system
3. Parse the ELF file and load its segments into memory
4. Retrieve the Graphics Output Protocol for framebuffer access
5. Exit boot services to take full control of hardware
6. Jumping to the kernel entry point with the `BootInfo` structure

## Dependencies

- `common`: A shared library containing structures like `BootInfo` that are used by both the bootloader and the kernel to ensure type-safe handover.
- `uefi`: Rusty wrapper for the Unified Extensible Firmware Interface.

## Building

The kernel is build using the custom target.
```bash
cargo build --target x86_64-unknown-uefi
#! or in the project root
make build_bootloader
```
