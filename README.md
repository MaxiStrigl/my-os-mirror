# myOS (name tbd)

A custom operating system project targeting x86_64 UEFI systems.

## Project Structure

This project is organized as a Cargo workspace.

- **`bootloader/`**: A UEFI application that initializes the system, loads the kernel and runs it
-  **`kernel/`**: The main operating system kernel
- **`common/`**: Shared structures and constants used by both the bootloader and kernel
- **`ovmf/`**: (Ignored by Git) Directory for storing UEFI firmware for emulation.
- **`esp/`**: (Ignored by Git) The EFI System Partition directory where build artifacts are staged for execution.
- **`runner.py`**: A python script that manages the staging process and launches QEMU.
- **`target/`**: Build artifacts


## Prerequisites

- **Rust**: Nightly toolchain with `llvm-tools`
- **QEMU**: For emulating the hardware.
- **Make**: For building and executing commands.
- **Python 3**: For the runner.py script
- **OVMF**: UEFI firmware files (`OVMF_CODE.fd` and `OVMF_VARS.fd`) are required in the `ovmf/` directory
