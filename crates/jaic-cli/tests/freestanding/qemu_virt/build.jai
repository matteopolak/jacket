#import "Basic";
#import "Compiler";
Board :: #import "Extensions/Embedded/Boards/Qemu_Virt_Aarch64";
Embedded :: #import "Extensions/Embedded";

#run {
    set_build_options_dc(.{ do_output = false });
    w := compiler_create_workspace("qemu-virt");
    options := get_build_options(w);
    options.output_executable_name = "prog.elf";
    Board.configure(*options);
    Embedded.link_for(*options, Board.memory_map);
    set_build_options(options, w);
    add_build_file("prog.jai", w);
}
