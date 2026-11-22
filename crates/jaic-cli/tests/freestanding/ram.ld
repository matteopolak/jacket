ENTRY(_start)
MEMORY { RAM (rwx) : ORIGIN = 0x40000000, LENGTH = 16M }
SECTIONS {
  .text : { *(.text.boot) *(.text .text.*) } > RAM
  .rodata : { *(.rodata .rodata.*) } > RAM
  .data : { __data_start = .; *(.data .data.*) __data_end = .; } > RAM
  __data_load = __data_start;
  .bss : { __bss_start = .; *(.bss .bss.* COMMON) __bss_end = .; } > RAM
  __heap_start = .;
  __heap_end = ORIGIN(RAM) + LENGTH(RAM) - 64K;
  __stack_top = ORIGIN(RAM) + LENGTH(RAM);
}
