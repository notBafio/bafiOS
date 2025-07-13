wsl rm -rf build/*

cargo build --package=bootloader --target=bits16.json
cargo build --package=stage2 --target=bits16.json
cargo build --package=stage3 --target=bits32.json
cargo build --package=kernel --target=bits32.json
cargo build --package=userland --target=bits32-I.json --release

cargo build --package=proc1 --target=bits32-I.json --release
cargo build --package=terminal --target=bits32-I.json --release
cargo build --package=filemanager --target=bits32-I.json --release
cargo build --package=ide --target=bits32-I.json --release
cargo build --package=exec --target=bits32-I.json --release
cargo build --package=img --target=bits32-I.json --release
cargo build --package=login --target=bits32-I.json --release

wsl sh -c "objcopy -I elf32-i386 -O binary target/bits16/debug/bootloader build/bootloader.bin"
wsl sh -c "objcopy -I elf32-i386 -O binary target/bits16/debug/stage2 build/stage2.bin"
wsl sh -c "objcopy -I elf32-i386 -O binary target/bits32/debug/stage3 build/stage3.bin"
wsl sh -c "objcopy -I elf32-i386 -O binary target/bits32/debug/kernel build/kernel.bin"

wsl dd if=/dev/zero of=build/disk.img bs=512 count=32768
wsl dd if=/dev/zero of=build/fat16.img bs=512 count=524288
wsl mkfs.fat -F 16 build/fat16.img

wsl mmd -i build/fat16.img ::/icons
wsl mmd -i build/fat16.img ::/lib

wsl mmd -i build/fat16.img ::/sys
wsl mmd -i build/fat16.img ::/sys/font

wsl mmd -i build/fat16.img ::/user
wsl mmd -i build/fat16.img ::/user/desktop
wsl mmd -i build/fat16.img ::/user/temps
wsl mmd -i build/fat16.img ::/user/downloads

wsl dd if=build/bootloader.bin of=build/disk.img conv=notrunc
wsl dd if=build/stage2.bin of=build/disk.img bs=512 seek=2048 conv=notrunc
wsl dd if=build/stage3.bin of=build/disk.img bs=512 seek=3072 conv=notrunc
wsl dd if=build/kernel.bin of=build/disk.img bs=512 seek=4096 conv=notrunc

wsl mcopy -i build/fat16.img font.psf "::sys/font/default.psf"
wsl mcopy -i build/fat16.img wallpaper.tga "::sys/bg.tga"

wsl mcopy -i build/fat16.img icons.db "::sys/icons.db"
wsl mcopy -i build/fat16.img exec.db "::sys/exec.db"
wsl mcopy -i build/fat16.img users.db "::sys/users.db"

wsl mcopy -i build/fat16.img icons/elf.tga "::icons/elf.tga"
wsl mcopy -i build/fat16.img icons/file.tga "::icons/file.tga"
wsl mcopy -i build/fat16.img icons/folder.tga "::icons/folder.tga"
wsl mcopy -i build/fat16.img icons/folder2.tga "::icons/folder2.tga"
wsl mcopy -i build/fat16.img icons/tga.tga "::icons/tga.tga"
wsl mcopy -i build/fat16.img icons/cat0.tga "::icons/cat0.tga"
wsl mcopy -i build/fat16.img icons/cat1.tga "::icons/cat1.tga"
wsl mcopy -i build/fat16.img icons/cat2.tga "::icons/cat2.tga"

wsl mcopy -i build/fat16.img font.psf "::/sys/font/font.psf"

wsl mcopy -i build/fat16.img target/bits32-I/release/userland "::user/user.elf"
wsl mcopy -i build/fat16.img target/bits32-I/release/proc1 "::user/desktop/proc1.elf"
wsl mcopy -i build/fat16.img target/bits32-I/release/terminal "::user/desktop/csl.elf"
wsl mcopy -i build/fat16.img target/bits32-I/release/filemanager "::user/desktop/files.elf"

wsl mcopy -i build/fat16.img target/bits32-I/release/ide "::user/desktop/ide.elf"
wsl mcopy -i build/fat16.img target/bits32-I/release/exec "::user/exec.elf"
wsl mcopy -i build/fat16.img target/bits32-I/release/img "::user/img.elf"
wsl mcopy -i build/fat16.img target/bits32-I/release/login "::user/login.elf"

wsl dd if=build/fat16.img of=build/disk.img bs=512 seek=9216 conv=notrunc

wsl rm -rf build/fat16.img

qemu-system-x86_64 -drive file=".\build\disk.img",format=raw -m 1G -serial stdio -netdev user,id=n0 -device rtl8139,netdev=n0 -no-reboot -object filter-dump,id=d0,netdev=n0,file=net.pcap
 
pause