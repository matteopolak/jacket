// The editor's settings, kept as JSON next to the user's other dotfiles. `JAI_EDITOR_CONFIG`
// names another file. A missing or invalid file gives the defaults; the file is written when the
// editor closes, so a hand edit survives only if made while the editor is not running.

Config :: struct @json(default) {
    font_size: float = 13;
    theme: string = "dark";
    tab_width: int = 4;
    line_numbers: bool = true;
    sidebar_width: float = 240;
    last_root: string;
}

config_path :: () -> string {
    override := environment_variable("JAI_EDITOR_CONFIG");
    if override.count return override;

    home := environment_variable("HOME");
    if !home.count home = environment_variable("USERPROFILE");
    if !home.count return "";

    return tprint("%/.jai-editor.json", home);
}

// Read the configuration from `path`; the defaults when it cannot be read or parsed.
load_config :: (path: string) -> Config {
    if !path.count || !file_exists(path) return .{};

    text, ok := read_entire_file(path);
    if !ok return .{};

    defer free(text.data);

    config, error := from_json(Config, text);

    if error.kind != .NONE {
        free_json(*error);
        return .{};
    }

    return config;
}

// Write the configuration to `path` as pretty JSON.
save_config :: (config: Config, path: string) -> bool {
    if !path.count return false;

    text := to_json(config, pretty = true);
    defer free(text.data);

    return write_entire_file(path, text);
}
