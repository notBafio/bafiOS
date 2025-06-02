OSNAME := $(shell uname)

.PHONY: all
all: clean rust objcopy disk run
	@echo "bafiOS up and running"

.PHONY: rust
rust:
	@rustup component add rust-src --toolchain nightly-2025-01-01-x86_64-unknown-linux-gnu

	@sudo apt update
	@sudo apt -y install mtools
	@sudo apt -y install qemu-system-x86_64

	@cargo build --package=bootloader --target=bits16.json
	@cargo build --package=stage2 --target=bits16.json
	@cargo build --package=stage3 --target=bits32.json
	@cargo build --package=kernel --target=bits32.json

	@cargo build --package=userland --target=bits32-I.json --release
	@cargo build --package=proc1 --target=bits32-I.json --release
	@cargo build --package=terminal --target=bits32-I.json --release
	@cargo build --package=filemanager --target=bits32-I.json --release
	@cargo build --package=ide --target=bits32-I.json --release
	@cargo build --package=exec --target=bits32-I.json --release
	@cargo build --package=img --target=bits32-I.json --release
	@cargo build --package=login --target=bits32-I.json --release

.PHONY: objcopy
objcopy:
	@mkdir -p build
	@objcopy -I elf32-i386 -O binary target/bits16/debug/bootloader build/bootloader.bin
	@objcopy -I elf32-i386 -O binary target/bits16/debug/stage2 build/stage2.bin
	@objcopy -I elf32-i386 -O binary target/bits32/debug/stage3 build/stage3.bin
	@objcopy -I elf32-i386 -O binary target/bits32/debug/kernel build/kernel.bin

.PHONY: disk
disk:

	@dd if=/dev/zero of=build/disk.img bs=512 count=32768

	@dd if=/dev/zero of=build/fat16.img bs=512 count=524288
	@mkfs.fat -F 16 build/fat16.img

	@mmd -i build/fat16.img ::/icons
	@mmd -i build/fat16.img ::/lib

	@mmd -i build/fat16.img ::/sys
	@mmd -i build/fat16.img ::/sys/font

	@mmd -i build/fat16.img ::/user
	@mmd -i build/fat16.img ::/user/desktop
	@mmd -i build/fat16.img ::/user/temps
	@mmd -i build/fat16.img ::/user/downloads

	@dd if=build/bootloader.bin of=build/disk.img conv=notrunc
	@dd if=build/stage2.bin of=build/disk.img bs=512 seek=2048 conv=notrunc
	@dd if=build/stage3.bin of=build/disk.img bs=512 seek=3072 conv=notrunc
	@dd if=build/kernel.bin of=build/disk.img bs=512 seek=4096 conv=notrunc

	@mcopy -i build/fat16.img font.psf "::sys/font/default.psf"
	@mcopy -i build/fat16.img wallpaper.tga "::sys/bg.tga"

	@mcopy -i build/fat16.img icons.db "::sys/icons.db"
	@mcopy -i build/fat16.img exec.db "::sys/exec.db"
	@mcopy -i build/fat16.img users.db "::sys/users.db"

	@mcopy -i build/fat16.img icons/elf.tga "::icons/elf.tga"
	@mcopy -i build/fat16.img icons/file.tga "::icons/file.tga"
	@mcopy -i build/fat16.img icons/folder.tga "::icons/folder.tga"
	@mcopy -i build/fat16.img icons/folder2.tga "::icons/folder2.tga"
	@mcopy -i build/fat16.img icons/tga.tga "::icons/tga.tga"
	@mcopy -i build/fat16.img icons/cat0.tga "::icons/cat0.tga"
	@mcopy -i build/fat16.img icons/cat1.tga "::icons/cat1.tga"
	@mcopy -i build/fat16.img icons/cat2.tga "::icons/cat2.tga"

	@mcopy -i build/fat16.img font.psf "::/sys/font/font.psf"

	@mcopy -i build/fat16.img target/bits32-I/release/userland "::user/user.elf"
	@mcopy -i build/fat16.img target/bits32-I/release/proc1 "::user/desktop/proc1.elf"
	@mcopy -i build/fat16.img target/bits32-I/release/terminal "::user/desktop/csl.elf"
	@mcopy -i build/fat16.img target/bits32-I/release/filemanager "::user/desktop/files.elf"

	@mcopy -i build/fat16.img target/bits32-I/release/ide "::user/desktop/ide.elf"
	@mcopy -i build/fat16.img target/bits32-I/release/exec "::user/exec.elf"
	@mcopy -i build/fat16.img target/bits32-I/release/img "::user/img.elf"
	@mcopy -i build/fat16.img target/bits32-I/release/login "::user/login.elf"

	@dd if=build/fat16.img of=build/disk.img bs=512 seek=9216 conv=notrunc

	@rm -rf build/fat16.img

.PHONY: clean
clean:
	@cargo clean
	@rm -rf build/*

.PHONY: run
run:
	@qemu-system-x86_64 -drive file="build/disk.img",format=raw -m 1G -serial stdio -netdev user,id=n0 -device rtl8139,netdev=n0 -no-reboot
