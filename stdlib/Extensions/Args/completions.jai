// Shell completion scripts, written from the same Command_Spec that help uses.

#scope_module

completion_script :: (root: *Command_Spec, shell: Shell) -> string {
    if shell == {
        case .BASH; return bash_script(root);
        case .ZSH; return zsh_script(root);
        case .FISH; return fish_script(root);
        case .POWERSHELL; return powershell_script(root);
    }
    return "";
}

// A name that is safe in a shell function name: `my-tool` -> `my_tool`.
identifier :: (name: string) -> string {
    result := copy_string(name,, temp);
    for 0..result.count - 1 {
        c := result[it];
        if !is_alnum_byte(c) result[it] = #char "_";
    }
    return result;
}

// Visits every command with the names leading to it ("tool", "build", ...).
Command_Visit :: struct {
    spec: *Command_Spec;
    path: [] string; // names after the program
    parents: [..] *Command_Spec;
}

collect_commands :: (root: *Command_Spec) -> [..] Command_Visit {
    visits: [..] Command_Visit;
    walk_commands(root, .[], *visits, .[]);
    return visits;
}

walk_commands :: (spec: *Command_Spec, path: [] string, visits: *[..] Command_Visit, parents: [] *Command_Spec) {
    visit: Command_Visit;
    visit.spec = spec;
    visit.path = path;
    for parents array_add(*visit.parents, it);
    array_add(visits, visit);
    inherited: [..] *Command_Spec;
    for parents array_add(*inherited, it);
    array_add(*inherited, spec);
    for *sub: spec.subcommands {
        child: [..] string;
        for path array_add(*child, it);
        array_add(*child, sub.name);
        walk_commands(sub, child, visits, inherited);
    }
}

// Options a command accepts: its own, plus the `@global` ones of the commands above it.
accepted_options :: (visit: Command_Visit) -> [..] Option_Spec {
    result: [..] Option_Spec;
    for visit.spec.options if !it.hidden array_add(*result, it);
    for parent: visit.parents {
        for parent.options if it.global && !it.hidden array_add(*result, it);
    }
    return result;
}

command_key :: (program: string, path: [] string) -> string {
    key := identifier(program);
    for path key = tprint("%__%", key, identifier(it));
    return key;
}

takes_value :: (option: Option_Spec) -> bool {
    return option.kind == .VALUE || option.kind == .LIST;
}

// Values the shell should offer after the option: its enum choices, else files when the
// value is named like a path, else nothing.
wants_files :: (option: Option_Spec) -> bool {
    if option.choices.count return false;
    name := to_lower_copy(value_label(option),, temp);
    return contains(name, "file") || contains(name, "path") || contains(name, "dir");
}

single_quoted :: (text: string) -> string {
    return replace(text, "'", "'\\''",, temp);
}

// ---- bash ----

bash_script :: (root: *Command_Spec) -> string {
    out: String_Builder;
    function := identifier(root.name);
    visits := collect_commands(root);

    print(*out, "# bash completion for %\n# Save as /etc/bash_completion.d/% or source it from ~/.bashrc.\n\n", root.name, root.name);
    print(*out, "_%() {\n    local cur prev cmd i\n    cur=\"${COMP_WORDS[COMP_CWORD]}\"\n    prev=\"${COMP_WORDS[COMP_CWORD-1]}\"\n    cmd=\"%\"\n", function, function);
    print(*out, "    for ((i = 1; i < COMP_CWORD; i++)); do\n        case \"${cmd}:${COMP_WORDS[i]}\" in\n");
    for visit: visits {
        if !visit.path.count continue;
        parent_path: [..] string;
        for i: 0..visit.path.count - 2 array_add(*parent_path, visit.path[i]);
        print(*out, "            %:%) cmd=\"%\" ;;\n", command_key(root.name, parent_path), visit.path[visit.path.count - 1], command_key(root.name, visit.path));
    }
    append(*out, "        esac\n    done\n\n    case \"${cmd}:${prev}\" in\n");
    for visit: visits {
        for option: accepted_options(visit) {
            if !takes_value(option) continue;
            spellings: [..] string;
            array_add(*spellings, tprint("--%", option.long));
            if option.short array_add(*spellings, tprint("-%", short_text(option.short)));
            print(*out, "        %:%)\n", command_key(root.name, visit.path), join(..spellings, "|"));
            if option.choices.count {
                print(*out, "            COMPREPLY=($(compgen -W \"%\" -- \"$cur\"))\n", join(..option.choices, " "));
            } else if wants_files(option) {
                append(*out, "            COMPREPLY=($(compgen -f -- \"$cur\"))\n");
            } else {
                append(*out, "            COMPREPLY=()\n");
            }
            append(*out, "            return 0 ;;\n");
        }
    }
    append(*out, "    esac\n\n    local words=\"\" subcommands=\"\"\n    case \"$cmd\" in\n");
    for visit: visits {
        words: [..] string;
        for option: accepted_options(visit) {
            if option.short array_add(*words, tprint("-%", short_text(option.short)));
            array_add(*words, tprint("--%", option.long));
            if option.negatable array_add(*words, tprint("--no-%", option.long));
        }
        array_add(*words, "-h");
        array_add(*words, "--help");
        if root.version.count && !visit.path.count {
            array_add(*words, "-V");
            array_add(*words, "--version");
        }
        subcommands: [..] string;
        for visit.spec.subcommands array_add(*subcommands, it.name);
        print(*out, "        %) words=\"%\"; subcommands=\"%\" ;;\n", command_key(root.name, visit.path), join(..words, " "), join(..subcommands, " "));
    }
    append(*out, "    esac\n\n    if [[ \"$cur\" == -* ]]; then\n        COMPREPLY=($(compgen -W \"$words\" -- \"$cur\"))\n");
    append(*out, "    elif [[ -n \"$subcommands\" ]]; then\n        COMPREPLY=($(compgen -W \"$subcommands\" -- \"$cur\"))\n    else\n        COMPREPLY=($(compgen -f -- \"$cur\"))\n    fi\n}\n\n");
    print(*out, "complete -F _% -o bashdefault -o default %\n", function, root.name);
    return builder_to_string(*out);
}

// ---- zsh ----

zsh_text :: (text: string) -> string {
    // Inside '...[description]': quote ' and escape [ ] :.
    escaped := replace(text, "'", "'\\''",, temp);
    escaped = replace(escaped, "[", "\\[",, temp);
    escaped = replace(escaped, "]", "\\]",, temp);
    return escaped;
}

zsh_value_action :: (option: Option_Spec) -> string {
    if option.choices.count return tprint("(%)", join(..option.choices, " "));
    if wants_files(option) return "_files";
    return " ";
}

zsh_script :: (root: *Command_Spec) -> string {
    out: String_Builder;
    function := identifier(root.name);
    visits := collect_commands(root);

    print(*out, "#compdef %\n\n# zsh completion for %. Save as _% in a directory on $fpath.\n\n", root.name, root.name, root.name);
    for visit: visits {
        key := command_key(root.name, visit.path);
        print(*out, "_%() {\n    local context state state_descr line\n    typeset -A opt_args\n    _arguments -C \\\n", key);
        for option: accepted_options(visit) {
            if takes_value(option) {
                action := ifx option.kind == .LIST then "*" else "";
                if option.short print(*out, "        '%-%+[%]:%:%' \\\n", action, short_text(option.short), zsh_text(option.help), value_label(option), zsh_value_action(option));
                print(*out, "        '%--%=[%]:%:%' \\\n", action, option.long, zsh_text(option.help), value_label(option), zsh_value_action(option));
            } else {
                action := ifx option.kind == .COUNT then "*" else "";
                if option.short print(*out, "        '%-%[%]' \\\n", action, short_text(option.short), zsh_text(option.help));
                print(*out, "        '%--%[%]' \\\n", action, option.long, zsh_text(option.help));
                if option.negatable print(*out, "        '--no-%[%]' \\\n", option.long, zsh_text(option.help));
            }
        }
        print(*out, "        '(-h --help)'{-h,--help}'[Print help]' \\\n");
        if root.version.count && !visit.path.count print(*out, "        '(-V --version)'{-V,--version}'[Print version]' \\\n");
        for visit.spec.positionals {
            spec_action := ifx wants_files(it) || !it.choices.count then "_files" else zsh_value_action(it);
            print(*out, "        '%:%:%' \\\n", ifx it.kind == .LIST then "*" else "", value_label(it), spec_action);
        }
        if visit.spec.subcommands.count {
            append(*out, "        '1: :->command' \\\n        '*:: :->arguments' && return\n    case $state in\n        command)\n            local -a commands\n            commands=(\n");
            for visit.spec.subcommands print(*out, "                '%:%'\n", it.name, replace(zsh_text(it.about), ":", "\\:",, temp));
            append(*out, "            )\n            _describe -t commands 'command' commands ;;\n        arguments)\n            case $line[1] in\n");
            for sub: visit.spec.subcommands {
                child: [..] string;
                for visit.path array_add(*child, it);
                array_add(*child, sub.name);
                print(*out, "                %) _% ;;\n", sub.name, command_key(root.name, child));
            }
            append(*out, "            esac ;;\n    esac\n}\n\n");
        } else {
            append(*out, "        && return\n}\n\n");
        }
    }
    print(*out, "if [ \"$funcstack[1]\" = \"_%\" ]; then\n    _% \"$@\"\nelse\n    compdef _% %\nfi\n", function, function, function, root.name);
    return builder_to_string(*out);
}

// ---- fish ----

fish_script :: (root: *Command_Spec) -> string {
    out: String_Builder;
    visits := collect_commands(root);
    print(*out, "# fish completion for %. Save as ~/.config/fish/completions/%.fish.\n\n", root.name, root.name);
    print(*out, "complete -c % -f\n", root.name);
    for visit: visits {
        condition := "";
        if visit.path.count {
            condition = tprint(" -n '__fish_seen_subcommand_from %'", visit.path[visit.path.count - 1]);
        } else if root.subcommands.count {
            condition = "";
        }
        for option: accepted_options(visit) {
            line: String_Builder;
            print(*line, "complete -c %%", root.name, condition);
            if option.short print(*line, " -s %", short_text(option.short));
            print(*line, " -l %", option.long);
            if takes_value(option) {
                append(*line, " -r");
                if option.choices.count print(*line, " -f -a '%'", join(..option.choices, " "));
                else if !wants_files(option) append(*line, " -f");
            }
            if option.help.count print(*line, " -d '%'", single_quoted(option.help));
            print(*out, "%\n", builder_to_string(*line,, temp));
            if option.negatable {
                print(*out, "complete -c %% -l no-% -d '%'\n", root.name, condition, option.long, single_quoted(option.help));
            }
        }
        print(*out, "complete -c %% -s h -l help -d 'Print help'\n", root.name, condition);
        if root.version.count && !visit.path.count print(*out, "complete -c % -s V -l version -d 'Print version'\n", root.name);
        for visit.spec.positionals {
            if wants_files(it) || !it.choices.count print(*out, "complete -c %% -F\n", root.name, condition);
            else print(*out, "complete -c %% -f -a '%'\n", root.name, condition, join(..it.choices, " "));
        }
        if visit.spec.subcommands.count {
            parent_condition := ifx visit.path.count then tprint(" -n '__fish_seen_subcommand_from %'", visit.path[visit.path.count - 1]) else " -n '__fish_use_subcommand'";
            for visit.spec.subcommands {
                print(*out, "complete -c %% -a '%' -d '%'\n", root.name, parent_condition, it.name, single_quoted(it.about));
            }
        }
    }
    return builder_to_string(*out);
}

// ---- PowerShell ----

ps_text :: (text: string) -> string {
    return replace(text, "'", "''",, temp);
}

powershell_script :: (root: *Command_Spec) -> string {
    out: String_Builder;
    visits := collect_commands(root);
    print(*out, "# PowerShell completion for %. Add this to your $PROFILE.\n\n", root.name);
    print(*out, "Register-ArgumentCompleter -Native -CommandName '%' -ScriptBlock {\n    param($wordToComplete, $commandAst, $cursorPosition)\n", root.name);
    append(*out, "    $elements = @($commandAst.CommandElements | ForEach-Object { $_.ToString() })\n");
    print(*out, "    $command = '%'\n    $previous = ''\n    foreach ($element in $elements | Select-Object -Skip 1) {\n        switch (\"$command;$element\") {\n", root.name);
    for visit: visits {
        if !visit.path.count continue;
        parent_path: [..] string;
        for i: 0..visit.path.count - 2 array_add(*parent_path, visit.path[i]);
        parent_key := join(..parent_path, ";", before_first = true);
        print(*out, "            '%%;%' { $command = '%;%' }\n", root.name, parent_key, visit.path[visit.path.count - 1], root.name, join(..visit.path, ";"));
    }
    append(*out, "        }\n    }\n\n    $previous = if ($elements.Count -gt 1) { $elements[$elements.Count - 1] } else { '' }\n");
    append(*out, "    if ($wordToComplete -eq '' -and $elements.Count -gt 1) { $previous = $elements[$elements.Count - 1] }\n    elseif ($elements.Count -gt 2) { $previous = $elements[$elements.Count - 2] }\n\n");
    append(*out, "    $values = switch (\"$command;$previous\") {\n");
    for visit: visits {
        for option: accepted_options(visit) {
            if !takes_value(option) || !option.choices.count continue;
            names: [..] string;
            array_add(*names, tprint("--%", option.long));
            if option.short array_add(*names, tprint("-%", short_text(option.short)));
            for name: names {
                print(*out, "        '%%;%' { @(", root.name, join(..visit.path, ";", before_first = true), name);
                for choice: option.choices {
                    if it_index append(*out, ", ");
                    print(*out, "'%'", ps_text(choice));
                }
                append(*out, ") }\n");
            }
        }
    }
    append(*out, "        default { $null }\n    }\n");
    append(*out, "    if ($null -ne $values) {\n        $values | Where-Object { $_ -like \"$wordToComplete*\" } | ForEach-Object {\n            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)\n        }\n        return\n    }\n\n");
    append(*out, "    $completions = switch ($command) {\n");
    for visit: visits {
        print(*out, "        '%%' {\n            @(\n", root.name, join(..visit.path, ";", before_first = true));
        for option: accepted_options(visit) {
            if option.short print(*out, "                [System.Management.Automation.CompletionResult]::new('-%', '-%', 'ParameterName', '%')\n", short_text(option.short), short_text(option.short), ps_text(option.help));
            print(*out, "                [System.Management.Automation.CompletionResult]::new('--%', '--%', 'ParameterName', '%')\n", option.long, option.long, ps_text(option.help));
            if option.negatable print(*out, "                [System.Management.Automation.CompletionResult]::new('--no-%', '--no-%', 'ParameterName', '%')\n", option.long, option.long, ps_text(option.help));
        }
        append(*out, "                [System.Management.Automation.CompletionResult]::new('-h', '-h', 'ParameterName', 'Print help')\n");
        append(*out, "                [System.Management.Automation.CompletionResult]::new('--help', '--help', 'ParameterName', 'Print help')\n");
        if root.version.count && !visit.path.count {
            append(*out, "                [System.Management.Automation.CompletionResult]::new('-V', '-V', 'ParameterName', 'Print version')\n");
            append(*out, "                [System.Management.Automation.CompletionResult]::new('--version', '--version', 'ParameterName', 'Print version')\n");
        }
        for visit.spec.subcommands {
            print(*out, "                [System.Management.Automation.CompletionResult]::new('%', '%', 'ParameterValue', '%')\n", it.name, it.name, ps_text(it.about));
        }
        append(*out, "            )\n        }\n");
    }
    append(*out, "    }\n\n    $completions | Where-Object { $_.CompletionText -like \"$wordToComplete*\" }\n}\n");
    return builder_to_string(*out);
}
