ENTRY(_start)

SECTIONS {

    . = 0x7c00;
    _stack_top = .;

    .boot :
    {
        *(.boot .boot.*)
    }

    .text :
    {
        *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }
    
    .data :
    {
        *(.rodata .rodata.*)
        *(.data .data.*)
        *(.got .got.*)
    }

    . = 0x7e00 - 2;

    .magic_number :
    {
        SHORT(0xaa55)
    }

    _stub_start = .;
}
