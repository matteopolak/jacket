deinit :: (using help: *Codex_Help) {
    assert(help != null);
    // Index arrays belong to the helper; records and strings belong to memory.
    if help.modules.data free(help.modules.data);
    if help.files.data free(help.files.data);
    if help.procedures.data free(help.procedures.data);

    help.* = .{};
}

#scope_file
#import "Basic";
