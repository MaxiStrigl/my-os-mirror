import os
import shutil
import subprocess
import sys

ESP_DIR = "esp"
BOOT_DIR = os.path.join(ESP_DIR, "EFI", "BOOT")
BOOTLOADER_SRC = "target/x86_64-unknown-uefi/debug/bootloader.efi"
KERNEL_SRC = "target/x86_64-my-os/debug/kernel"

OVMF_CODE = "ovmf/OVMF_CODE.4m.fd"
OVMF_VARS = "ovmf/OVMF_VARS.4m.fd"


def main():
    # Prepare the efi directory
    if os.path.exists(ESP_DIR):
        shutil.rmtree(ESP_DIR)
    os.makedirs(BOOT_DIR)

    # Copy the bootloader to the UEFI path
    shutil.copy(BOOTLOADER_SRC, os.path.join(BOOT_DIR, "BOOTX64.EFI"))
    shutil.copy(KERNEL_SRC, os.path.join(ESP_DIR, "kernel.elf"))

    # Build the QEMU command
    qemu_cmd = [
        "qemu-system-x86_64",
        "-drive",
        f"if=pflash,format=raw,readonly=on,file={OVMF_CODE}",
        "-drive",
        f"if=pflash,format=raw,readonly=on,file={OVMF_VARS}",
        "-drive",
        f"format=raw,file=fat:rw:{ESP_DIR}",
        "-serial",
        "stdio",
    ]

    print("--- LAUNCHING QEMU ---")
    try:
        subprocess.run(qemu_cmd)
    except KeyboardInterrupt:
        sys.exit(0)


if __name__ == "__main__":
    main()
