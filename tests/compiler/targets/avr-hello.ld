ENTRY(_start)

MEMORY {
  FLASH (rx) : ORIGIN = 0x0, LENGTH = 0x8000
  RAM (rwx)  : ORIGIN = 0x800100, LENGTH = 0x800
}

SECTIONS {
  .text : {
    KEEP(*(.vector_table))
    *(.text.boot)
    *(.text .text.*)
  } > FLASH
  .data : { __data_start = .; *(.rodata .rodata.*) *(.data .data.*) __data_end = .; } > RAM AT > FLASH
  __data_load = LOADADDR(.data);
  .bss (NOLOAD) : { __bss_start = .; *(.bss .bss.* COMMON) __bss_end = .; } > RAM
  __heap_start = ALIGN(16);
  __stack_top = ORIGIN(RAM) + LENGTH(RAM);
  __heap_end = __stack_top - 0x100;
  /DISCARD/ : { *(.comment) *(.eh_frame*) *(.note*) }
}
