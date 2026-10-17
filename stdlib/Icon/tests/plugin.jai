#import "Basic";
I :: #import "Icon";
C :: #import "Compiler";

errors: int;
messages: int;

capture :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && data == null);
    messages += 1;
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

main :: () {
    plugin := I.get_plugin();

    assert(plugin != null && plugin.init == I.init && plugin.message == I.message);
    assert(plugin.shutdown == I.shutdown && plugin.log_help == I.log_help);
    defer I.shutdown(plugin);
    no_options: [0] string;

    assert(I.init(plugin, no_options));
    filename := copy_string("first.ico");

    defer free(filename);
    options: [4] string = .["-icon", filename, "-icon", "last.ico"];

    assert(plugin.init(plugin, options));
    unrelated := C.Message.{ kind = .FILE };

    plugin.message(plugin, *unrelated);
    assert(messages == 0);

    push_context {
        context.logger = capture;
        context.logger_data = null;
        assert(I.init(plugin, no_options));
        after_write := C.Message_Phase.{};
        after_write.kind = .PHASE;
        after_write.phase = .POST_WRITE_EXECUTABLE;
        I.message(plugin, cast(*C.Message) *after_write);
        missing: [1] string = .["-icon"];
        unknown: [1] string = .["-unknown"];

        assert(!I.init(plugin, missing));
        assert(!I.init(plugin, unknown));
        I.log_help(plugin);
    }

    assert(errors == 2 && messages == 4 && I.HELP_STRING.count > 0);
    print("ok\n");
}
