// Help text, usage lines, error reports and the colour rules.

#scope_module

Option_Kind :: enum u8 {
    FLAG;
    COUNT;
    VALUE;
    LIST;
}

// One option or positional as help and completions see it.
Option_Spec :: struct {
    field: string;
    long: string; // no dashes; for a positional, the name shown in usage
    short: u8; // 0: none
    env: string;
    value_name: string; // `FILE` in `--config <FILE>`
    help: string;
    kind: Option_Kind;
    required: bool;
    hidden: bool;
    global: bool;
    negatable: bool; // a bool flag also has `--no-<long>`
    choices: [] string;
    default_text: string;
    range_text: string;
}

Command_Spec :: struct {
    name: string;
    about: string;
    version: string;
    options: [..] Option_Spec;
    positionals: [..] Option_Spec;
    subcommands: [..] Command_Spec;
    subcommand_required: bool;
}

// ---- colour ----

// `--color auto` decides as jaic and jailint do: NO_COLOR turns colour off, a FORCE_COLOR or
// CLICOLOR_FORCE that is not empty or "0" on, and otherwise it is on for a terminal that is not
// "dumb" (on Windows, a console known to understand ANSI codes).
#scope_export

Stream :: enum u8 {
    OUTPUT;
    ERROR;
}

color_enabled :: (stream: Stream) -> bool {
    if color_mode == .ALWAYS return true;
    if color_mode == .NEVER return false;
    if environment_set("NO_COLOR") return false;
    if forced("FORCE_COLOR") || forced("CLICOLOR_FORCE") return true;
    if environment_value("TERM") == "dumb" return false;
    return is_terminal(stream);
}

#scope_module

ANSI_RESET :: "\e[0m";
ANSI_ERROR :: "\e[1;31m";
ANSI_HINT :: "\e[1;36m";
ANSI_HEADING :: "\e[1;4m";
ANSI_LITERAL :: "\e[1m";
ANSI_PLACEHOLDER :: "\e[36m";

paint :: (builder: *String_Builder, on: bool, code: string, text: string) {
    if on append(builder, code);
    append(builder, text);
    if on append(builder, ANSI_RESET);
}

// ---- usage ----

positional_label :: (positional: Option_Spec) -> string {
    name := ifx positional.value_name.count then positional.value_name else to_upper_copy(positional.long,, temp);
    if positional.kind == .LIST {
        if positional.required return tprint("<%>...", name);
        return tprint("[%]...", name);
    }
    if positional.required return tprint("<%>", name);
    return tprint("[%]", name);
}

// `jaifmt [OPTIONS] [PATHS]...` for the command at the end of `path`.
usage_line :: (root: *Command_Spec, path: [] string) -> string {
    spec := root;
    builder: String_Builder;
    append(*builder, root.name);
    for path {
        for *sub: spec.subcommands if sub.name == it {
            spec = sub;
            break;
        }
        append(*builder, " ");
        append(*builder, it);
    }
    if has_visible_options(spec) append(*builder, " [OPTIONS]");
    for spec.positionals {
        append(*builder, " ");
        append(*builder, positional_label(it));
    }
    if spec.subcommands.count {
        append(*builder, ifx spec.subcommand_required then " <COMMAND>" else " [COMMAND]");
    }
    return builder_to_string(*builder,, temp);
}

// Always true: every command has --help.
has_visible_options :: (spec: *Command_Spec) -> bool {
    return true;
}

find_command :: (root: *Command_Spec, path: [] string) -> *Command_Spec {
    spec := root;
    for path {
        found: *Command_Spec;
        for *sub: spec.subcommands if sub.name == it {
            found = sub;
            break;
        }
        if !found break;
        spec = found;
    }
    return spec;
}

// ---- help ----

option_label :: (option: Option_Spec, color: bool) -> string {
    text: String_Builder;
    if option.short {
        paint(*text, color, ANSI_LITERAL, tprint("-%", short_text(option.short)));
        append(*text, ", ");
    } else {
        append(*text, "    ");
    }
    long := ifx option.negatable then tprint("--[no-]%", option.long) else tprint("--%", option.long);
    paint(*text, color, ANSI_LITERAL, long);
    if option.kind == .VALUE || option.kind == .LIST {
        append(*text, " ");
        paint(*text, color, ANSI_PLACEHOLDER, tprint("<%>", value_label(option)));
    }
    return builder_to_string(*text,, temp);
}

short_text :: (letter: u8) -> string {
    return copy_string(string.{ 1, *letter },, temp);
}

value_label :: (option: Option_Spec) -> string {
    if option.value_name.count return option.value_name;
    return to_upper_copy(option.long,, temp);
}

// The visible width of an option's label, without colour codes.
label_width :: (option: Option_Spec) -> s64 {
    width := 4 + 2 + option.long.count; // "-c, " or 4 spaces, then "--long"
    if option.negatable width += 5; // "[no-]"
    if option.kind == .VALUE || option.kind == .LIST width += 3 + value_label(option).count; // " <NAME>"
    return width;
}

// Everything after the description: [default: 4] [env: X] ...
notes_for :: (option: Option_Spec) -> string {
    notes: String_Builder;
    if option.default_text.count print(*notes, " [default: %]", option.default_text);
    if option.choices.count && option.kind != .FLAG print(*notes, " [possible values: %]", join(..option.choices, ", "));
    if option.range_text.count print(*notes, " [range: %]", option.range_text);
    if option.env.count print(*notes, " [env: %]", option.env);
    return builder_to_string(*notes,, temp);
}

render_help :: (root: *Command_Spec, path: [] string, color: bool) -> string {
    spec := find_command(root, path);
    out: String_Builder;

    if spec.about.count {
        append(*out, spec.about);
        append(*out, "\n\n");
    }
    paint(*out, color, ANSI_HEADING, "Usage:");
    print(*out, " %\n", usage_line(root, path));

    // Left column entries, in the order shown.
    labels: [..] string;
    widths: [..] s64;
    descriptions: [..] string;
    sections: [..] s64; // 0 arguments, 1 options, 2 commands

    for spec.positionals {
        if it.hidden continue;
        array_add(*labels, positional_label(it));
        array_add(*widths, positional_label(it).count);
        array_add(*descriptions, tprint("%%", it.help, notes_for(it)));
        array_add(*sections, 0);
    }
    shown: [..] Option_Spec;
    for spec.options array_add(*shown, it);
    // The `@global` options of the commands above are accepted here too.
    ancestor := root;
    for step: path {
        for ancestor.options if it.global array_add(*shown, it);
        ancestor = find_command(ancestor, .[step]);
    }
    for shown {
        if it.hidden continue;
        array_add(*labels, option_label(it, color));
        array_add(*widths, label_width(it));
        array_add(*descriptions, tprint("%%", it.help, notes_for(it)));
        array_add(*sections, 1);
    }
    help_option := Option_Spec.{ long = "help", short = #char "h", help = "Print help" };
    version_option := Option_Spec.{ long = "version", short = #char "V", help = "Print version" };
    {
        array_add(*labels, option_label(help_option, color));
        array_add(*widths, label_width(help_option));
        array_add(*descriptions, help_option.help);
        array_add(*sections, 1);
    }
    if root.version.count && !path.count {
        array_add(*labels, option_label(version_option, color));
        array_add(*widths, label_width(version_option));
        array_add(*descriptions, version_option.help);
        array_add(*sections, 1);
    }
    for spec.subcommands {
        array_add(*labels, ifx color then tprint("%1%2%3", ANSI_LITERAL, it.name, ANSI_RESET) else it.name);
        array_add(*widths, it.name.count);
        array_add(*descriptions, it.about);
        array_add(*sections, 2);
    }

    column := 0;
    for widths if it > column column = it;

    titles := string.["Arguments:", "Options:", "Commands:"];
    for section: 0..2 {
        first := true;
        for labels {
            if sections[it_index] != section continue;
            if first {
                append(*out, "\n");
                paint(*out, color, ANSI_HEADING, titles[section]);
                append(*out, "\n");
                first = false;
            }
            append(*out, "  ");
            // Positionals and commands carry no colour codes of their own: colour them here.
            if section == 0 && color paint(*out, true, ANSI_PLACEHOLDER, it);
            else append(*out, it);
            for 1..column - widths[it_index] + 2 append(*out, " ");
            append(*out, descriptions[it_index]);
            append(*out, "\n");
        }
    }
    return builder_to_string(*out);
}

// ---- errors ----

#scope_export

// Prints a parse error the way the compilers print theirs:
//
//     error: unknown option `--chek`
//     help: did you mean `--check`?
//     usage: jaifmt [OPTIONS] [PATHS]...
//     For more information, run `jaifmt --help`.
report :: (error: Error) {
    if error.kind == .NONE return;
    color := color_enabled(.ERROR);
    out: String_Builder;
    paint(*out, color, ANSI_ERROR, "error");
    print(*out, ": %\n", error.message);
    if error.suggestion.count {
        paint(*out, color, ANSI_HINT, "help");
        print(*out, ": did you mean `%`?\n", error.suggestion);
    }
    if error.usage.count {
        paint(*out, color, ANSI_HINT, "usage");
        print(*out, ": %\n", error.usage);
    }
    if error.command.count print(*out, "For more information, run `% --help`.\n", error.command);
    print("%", builder_to_string(*out,, temp), to_standard_error = true);
}
