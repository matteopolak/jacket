/* Code and initial data in one region, the data's run-time address in another: init_runtime
   copies .data from its load address, as it does on a microcontroller's flash and RAM. */
ENTRY(_start)
MEMORY {
  FLASH (rx) : ORIGIN = 0x40000000, LENGTH = 1M
  RAM (rwx)  : ORIGIN = 0x40100000, LENGTH = 15M
}
SECTIONS {
  .text : { *(.text.boot) *(.text .text.*) } > FLASH
  .rodata : { *(.rodata .rodata.*) } > FLASH
  .data : { __data_start = .; *(.data .data.*) __data_end = .; } > RAM AT > FLASH
  __data_load = LOADADDR(.data);
  .bss : { __bss_start = .; *(.bss .bss.* COMMON) __bss_end = .; } > RAM
  __heap_start = .;
  __heap_end = ORIGIN(RAM) + LENGTH(RAM) - 64K;
  __stack_top = ORIGIN(RAM) + LENGTH(RAM);
}
