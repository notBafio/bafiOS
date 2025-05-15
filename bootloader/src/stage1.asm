.section .boot, "awx"
.global _start
.code16

_start:
    cli

    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax
    
    cld
    
    mov sp, 0x7c00 - 0x100
    sub sp, 0x100

    call _boot

end:
    hlt
    jmp end