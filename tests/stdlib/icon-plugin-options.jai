// The Icon metaprogram plugin takes "-icon <file>" and ignores messages until an icon is set.
#import "Basic";
Icon :: #import "Icon";
#import "File";
#import "String";

#run {
    plugin := Icon.get_plugin();
    assert(plugin.handle_one_option != null && plugin.message != null && plugin.shutdown != null);
    options := string.["-other", "-icon", "app.ico", "-icon"];
    assert(plugin.handle_one_option(plugin, options, 0) == 0, "not ours");
    assert(plugin.handle_one_option(plugin, options, 1) == 3, "flag and its argument");
    assert(plugin.handle_one_option(plugin, options, 3) == 4, "missing argument still advances");
    assert(plugin.handle_one_option(plugin, options, 9) == 9, "out of range");
    plugin.message(plugin, null);
    plugin.shutdown(plugin);
}

main :: () {
    entries();
    print("ok\n");
}

// The desktop entry written for Linux builds: plain text and file output, so it runs everywhere.
entries :: () {
    text := Icon.desktop_entry_text("game", "/opt/game/game", "/opt/game/icon.png");
    assert(text == "[Desktop Entry]\nType=Application\nName=game\nExec=/opt/game/game\nIcon=/opt/game/icon.png\nTerminal=false\n", "entry text: %", text);

    make_directory_if_it_does_not_exist(".scratch/icon-plugin-options", recursive = true);
    path := Icon.write_desktop_entry(".scratch/icon-plugin-options/tool", "tool.png");
    assert(path == ".scratch/icon-plugin-options/tool.desktop", "path %", path);
    written, ok := read_entire_file(path);
    assert(ok && contains(written, "Name=tool\n") && contains(written, "Icon=tool.png\n"), "written entry %", written);
    assert(Icon.write_desktop_entry(".scratch/icon-plugin-options/missing/tool", "x.png") == "", "an unwritable directory fails cleanly");
    delete_directory(".scratch/icon-plugin-options");
}
