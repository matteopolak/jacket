// Private subsystem SDK unit; not loaded or exported by the public Simp facade.
#import "Basic";
Contexts :: #import "internal/rendering/window-contexts";

mark :: (calls: *s64) -> bool {
    calls.* += 1;
    return true;
}

main :: () {
    args := get_command_line_arguments();
    if args.count != 2 exit(2);

    calls: s64;

    assert(mark(*calls));

    if args[1] == "enabled" {
        if calls != 1 exit(3);
    } else if args[1] == "disabled" {
        if calls != 0 exit(4);
    } else exit(5);

    first := begin_context();
    if !first exit(6);

    defer end_context(first);
    selected := Contexts.capture();
    second := begin_context();

    if !second exit(7);

    defer end_context(second);
    if Contexts.current_native() == selected.native exit(8);

    Contexts.restore(selected);
    if Contexts.current_native() != selected.native exit(9);

    write_string("ok\n");
}

#scope_file
begin_context :: () -> *void #foreign GL_PROBE "window_gl_probe_begin";
end_context :: (context: *void) #foreign GL_PROBE "window_gl_probe_end";
