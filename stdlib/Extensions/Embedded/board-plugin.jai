// The metaprogram plugin every board offers (`jaic app.jai +Extensions/Embedded/Boards/<Board>`):
// it configures the workspace for the board and links it by the board's memory map. It writes the
// linker script when the executable is about to be written (`PRE_WRITE_EXECUTABLE`), so `-check`
// writes no script either. With the option `tests` it also exports the reset function that runs
// `jacket test`'s entry point (`+Extensions/Testing/Discovery` adds it), so the tests need no
// `jai_start`. Runs in the metaprogram, on the host.
Board_Plugin :: struct {
    #as using base: Metaprogram_Plugin;
    // The board's module, as an import path: `Extensions/Embedded/Boards/Qemu_Virt_Aarch64`.
    board: string;
    // Makes the workspace build for the board (the board's `configure`).
    configure: (options: *Build_Options);
    map: Memory_Map;
    // Where the linker script goes, set once the workspace is configured.
    script: string;
    tests: bool;
}

// The plugin of the board `board`, which `configure`s a workspace for itself, lays it out by
// `map` and has a `start` that runs a program. A board's `get_plugin` returns this:
//
// ```jai,ignore
// get_plugin :: () -> *Metaprogram_Plugin {
//     board :: "Extensions/Embedded/Boards/Mps2_An386";
//     return Embedded.board_plugin(board, memory_map, configure);
// }
// ```
board_plugin :: (
    board: string,
    map: Memory_Map,
    configure: (options: *Build_Options)
) -> *Metaprogram_Plugin {
    plugin := New(Board_Plugin);
    plugin.board = board;
    plugin.map = map;
    plugin.configure = configure;

    plugin.init = (plugin: *Metaprogram_Plugin, options: [] string) -> bool {
        board_plugin := cast(*Board_Plugin) plugin;

        for options {
            if it == "tests" {
                board_plugin.tests = true;
            } else {
                print("%: unknown option '%'\n", board_plugin.board, it);

                return false;
            }
        }

        return true;
    };

    plugin.add_source = (plugin: *Metaprogram_Plugin) {
        board_plugin := cast(*Board_Plugin) plugin;
        options := get_build_options(plugin.workspace);

        board_plugin.configure(*options);
        board_plugin.script = copy_string(linker_script_path(options));
        add_linker_script(*options, board_plugin.script);
        set_build_options(options, plugin.workspace);

        if board_plugin.tests {
            add_build_string(tprint(TEST_START, board_plugin.board), plugin.workspace);
        }
    };

    plugin.message = (plugin: *Metaprogram_Plugin, message: *Message) {
        board_plugin := cast(*Board_Plugin) plugin;

        if message.workspace != plugin.workspace || message.kind != .PHASE return;

        if (cast(*Message_Phase) message).phase == .PRE_WRITE_EXECUTABLE {
            write_linker_script(board_plugin.script, board_plugin.map);
        }
    };

    return plugin;
}

#scope_file

TEST_START :: #string START
Board :: #import "%";

#program_export "jai_start"
reset :: () #c_call #no_context {
    Board.start(__jacket_test_main);
}
START;
