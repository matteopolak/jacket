// Print_Color writes ANSI escapes around text, to stdout or stderr. On POSIX hosts the test
// points stdout and stderr at a file for a moment and checks the exact bytes.
#import "Basic";
#import "File";
#import "Print_Color";

#if OS == .MACOS || OS == .LINUX {
    POSIX :: #import "POSIX";
}

emit :: () {
    print_color("value %", 42, color = .RED);
    set_console_color(.HI_GREEN, .BOLD);
    write_string("bold");
    reset_console_color();
    with_console_color(.BG_BLUE, #code {
        write_string("inside");
    }, .UNDERLINE);
    print_color("to stderr", color = .YELLOW, to_standard_error = true);
    with_console_color(.CYAN, #code {
        write_string("err", true);
    }, to_standard_error = true);
}

main :: () {
    #if OS == .MACOS || OS == .LINUX {
        name :: ".scratch/print-color.txt";
        make_directory_if_it_does_not_exist(".scratch");
        file := POSIX.open(name, POSIX.O_WRONLY | POSIX.O_CREAT | POSIX.O_TRUNC, 420);
        assert(file >= 0);
        saved_out := POSIX.dup(1);
        saved_err := POSIX.dup(2);
        assert(POSIX.dup2(file, 1) >= 0 && POSIX.dup2(file, 2) >= 0);
        emit();
        POSIX.dup2(saved_out, 1);
        POSIX.dup2(saved_err, 2);
        POSIX.close(file);
        POSIX.close(saved_out);
        POSIX.close(saved_err);
        text, ok := read_entire_file(name);
        assert(ok);
        expected := "\e[0;31mvalue 42\e[0m\e[1;92mbold\e[0m\e[4;44minside\e[0m\e[0;33mto stderr\e[0m\e[0;36merr\e[0m";
        assert(text == expected, "escapes:\n%\nexpected\n%", text, expected);
        assert(file_delete(name));
    } else {
        emit();
        write_string("\n");
    }
    print("ok\n");
}
