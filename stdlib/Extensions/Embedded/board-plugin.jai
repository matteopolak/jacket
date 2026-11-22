// The metaprogram plugin every board offers (`jaic app.jai +Extensions/Embedded/Boards/<Board>`):
// it configures the workspace for the board, and with the option `tests` also exports the reset
// function that runs `jacket test`'s entry point (`+Extensions/Testing/Discovery` adds it), so the
// tests need no `jai_start`. Runs in the metaprogram, on the host.

Board_Plugin :: struct {
    #as using base: Metaprogram_Plugin;
    // The board's module, as an import path: `Extensions/Embedded/Boards/Qemu_Virt_Aarch64`.
    board: string;
    // Makes the workspace build for the board (the board's `configure`).
    configure: (options: *Build_Options);
    tests: bool;
}

// The plugin of the board `board`, which `configure`s a workspace for itself and has a `start`
// that runs a program. A board's `get_plugin` returns this:
//
// ```jai,ignore
// get_plugin :: () -> *Metaprogram_Plugin {
//     return Embedded.board_plugin("Extensions/Embedded/Boards/Mps2_An386", configure);
// }
// ```
board_plugin :: (board: string, configure: (options: *Build_Options)) -> *Metaprogram_Plugin {
    plugin := New(Board_Plugin);
    plugin.board = board;
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
        set_build_options(options, plugin.workspace);

        if board_plugin.tests {
            add_build_string(tprint(TEST_START, board_plugin.board), plugin.workspace);
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
