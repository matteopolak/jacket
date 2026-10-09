// The texts the golden tests compare: help for each fixture and each shell's completion script.
// Loaded by help.jai, completions.jai and the generator of golden.jai (not a test itself).

#load "fixtures.jai";

// Escape bytes are shown as <ESC> so the golden files stay readable.
visible :: (text: string) -> string {
    return replace(text, "\e", "<ESC>");
}

Named_Text :: struct {
    name: string;
    text: string;
}

// What `tool <args>` answers with: the help page of the command it ends at.
tool_help_page :: (args: ..string) -> string {
    argv: [..] string;
    array_add(*argv, "tool");
    for args array_add(*argv, it);
    _, error := Args.try_parse(Tool, TOOL_HELP, argv);
    return error.text;
}

help_outputs :: () -> [] Named_Text {
    Args.set_color_mode(.NEVER); // --help pages in tests must not depend on the terminal
    list: [..] Named_Text;
    array_add(*list, .{ "CLI_HELP", Args.help_text(Cli, HELP, "jaifmt") });
    array_add(*list, .{ "CLI_HELP_COLOR", visible(Args.help_text(Cli, HELP, "jaifmt", color = true)) });
    array_add(*list, .{ "KINDS_HELP_TEXT", Args.help_text(Kinds, KINDS_HELP, "kinds") });
    array_add(*list, .{ "COPY_HELP_TEXT", Args.help_text(Copy, COPY_HELP, "cp") });
    array_add(*list, .{ "TOOL_HELP_TEXT", Args.help_text(Tool, TOOL_HELP, "tool") });
    array_add(*list, .{ "OPTIONAL_HELP_TEXT", Args.help_text(Optional_Tool, OPTIONAL_HELP, "opt") });
    array_add(*list, .{ "TOOL_BUILD_HELP", tool_help_page("build", "--help") });
    array_add(*list, .{ "TOOL_REMOTE_HELP", tool_help_page("remote-add", "-h") });
    return list;
}

completion_outputs :: () -> [] Named_Text {
    list: [..] Named_Text;
    array_add(*list, .{ "CLI_BASH", Args.completions(Cli, HELP, .BASH, "jaifmt") });
    array_add(*list, .{ "CLI_ZSH", Args.completions(Cli, HELP, .ZSH, "jaifmt") });
    array_add(*list, .{ "CLI_FISH", Args.completions(Cli, HELP, .FISH, "jaifmt") });
    array_add(*list, .{ "CLI_POWERSHELL", Args.completions(Cli, HELP, .POWERSHELL, "jaifmt") });
    array_add(*list, .{ "TOOL_BASH", Args.completions(Tool, TOOL_HELP, .BASH, "tool") });
    array_add(*list, .{ "TOOL_ZSH", Args.completions(Tool, TOOL_HELP, .ZSH, "tool") });
    array_add(*list, .{ "TOOL_FISH", Args.completions(Tool, TOOL_HELP, .FISH, "tool") });
    array_add(*list, .{ "TOOL_POWERSHELL", Args.completions(Tool, TOOL_HELP, .POWERSHELL, "tool") });
    return list;
}
