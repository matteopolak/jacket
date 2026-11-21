// A memory map and the linker script made from it. Runs in the metaprogram, on the host.

#import "Basic";
#import "Compiler";
#import "File";

// A span of the address space.
Region :: struct {
    origin: u64;
    length: u64;
}

// Where the program lives. The image is laid out as
//
//     flash:  .vector_table  .text  .rodata  (initial values of .data)
//     ram:    .data  .bss  heap ...  stack
//
// With no `flash` (length 0) code, constants and data all go in `ram`, as when a loader copies the
// program into memory.
//
// The script defines the symbols `init_runtime` and the heap read: `__data_start`, `__data_end`,
// `__data_load`, `__bss_start`, `__bss_end`, `__heap_start`, `__heap_end` and `__stack_top`. The
// heap is the RAM between `.bss` and the stack.
Memory_Map :: struct {
    ram: Region;
    flash: Region;
    stack_size: u64 = 8192;
    entry := "_start";
}

// The linker script for `map`.
linker_script :: (map: Memory_Map) -> string {
    builder: String_Builder;
    code := ifx map.flash.length != 0 then "FLASH" else "RAM";

    print_to_builder(*builder, "ENTRY(%)\n\n", map.entry);
    append(*builder, "MEMORY {\n");
    if map.flash.length != 0 print_to_builder(*builder, "  FLASH (rx) : ORIGIN = 0x%, LENGTH = 0x%\n", formatInt(map.flash.origin, base = 16), formatInt(map.flash.length, base = 16));

    print_to_builder(*builder, "  RAM (rwx)  : ORIGIN = 0x%, LENGTH = 0x%\n", formatInt(map.ram.origin, base = 16), formatInt(map.ram.length, base = 16));
    append(*builder, "}\n\nSECTIONS {\n");

    print_to_builder(*builder, "  .text : {\n    KEEP(*(.vector_table))\n    *(.text.boot)\n    *(.text .text.*)\n  } > %\n", code);
    print_to_builder(*builder, "  .rodata : { *(.rodata .rodata.*) } > %\n", code);

    if map.flash.length != 0 {
        append(*builder, "  .data : { __data_start = .; *(.data .data.*) __data_end = .; } > RAM AT > FLASH\n");
        append(*builder, "  __data_load = LOADADDR(.data);\n");
    } else {
        append(*builder, "  .data : { __data_start = .; *(.data .data.*) __data_end = .; } > RAM\n");
        append(*builder, "  __data_load = __data_start;\n");
    }

    append(*builder, "  .bss (NOLOAD) : { __bss_start = .; *(.bss .bss.* COMMON) __bss_end = .; } > RAM\n");
    append(*builder, "  __heap_start = ALIGN(16);\n");
    append(*builder, "  __stack_top = ORIGIN(RAM) + LENGTH(RAM);\n");
    print_to_builder(*builder, "  __heap_end = __stack_top - 0x%;\n", formatInt(map.stack_size, base = 16));
    append(*builder, "  /DISCARD/ : { *(.comment) *(.eh_frame*) *(.note*) }\n}\n");

    return builder_to_string(*builder);
}

// Writes the linker script for `map` and adds it to `options`' link line (`-T`), for a workspace
// whose `os_target` is `.NONE`. The script goes in `script_path`, or beside the executable.
//
// ```jai,ignore
// options := get_build_options(w);
// link_for(*options, Memory_Map.{ ram = .{ 0x4000_0000, 0x100_0000 } });
// set_build_options(options, w);
// ```
link_for :: (options: *Build_Options, map: Memory_Map, script_path := "") {
    path := script_path;
    if !path {
        directory := ifx options.output_path then tprint("%/", options.output_path) else "";
        path = tprint("%%.ld", directory, options.output_executable_name);
    }

    written := write_entire_file(path, linker_script(map));

    assert(written, "could not write the linker script %", path);

    count := options.additional_linker_arguments.count;
    arguments := NewArray(count + 2, string);

    memcpy(arguments.data, options.additional_linker_arguments.data, count * size_of(string));

    arguments[count] = "-T";
    arguments[count + 1] = path;
    options.additional_linker_arguments = arguments;
}
