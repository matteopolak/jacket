ENTRY(_start)
MEMORY { RAM (rwx) : ORIGIN = 0x40000000, LENGTH = 16M }
SECTIONS {
  .text : { *(.text.boot) *(.text .text.*) } > RAM
  .rodata : { *(.rodata .rodata.*) } > RAM
  .data : { *(.data .data.*) } > RAM
  .bss : { *(.bss .bss.* COMMON) } > RAM
  __stack_top = ORIGIN(RAM) + LENGTH(RAM);
}
