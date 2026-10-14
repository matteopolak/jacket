#import "Basic";

Capture :: struct {
    messages: [..] string;
    infos: [..] Log_Info;
    last_message: string;
}

capture :: (message: string, data: *void, info: Log_Info) {
    state := cast(*Capture) data;
    array_add(*state.messages, copy_string(message));
    array_add(*state.infos, info);
    state.last_message = message;
}

main :: () {
    state: Capture;

    defer {
        for state.messages free(it);
        array_free(state.messages);
        array_free(state.infos);
    }

    section := Log_Section.{ "network" };
    explicit := Source_Code_Location.{ fully_pathed_filename = "chosen.jai", line_number = 123, character_number = 7 };
    caller := #caller_location;
    backing: [2048] u8;
    storage: Temporary_Storage_Paged_Alloc;
    storage.data = backing.data;
    storage.size = backing.count;

    push_context {
        context.logger = capture;
        context.temporary_storage = *storage;
        context.logger_data = *state;
        context.log_source_identifier = 0x123456789abcdef0;
        log("value % / %", 17, "text");
        log(*section, "section %", true, loc = explicit, flags = .WARNING, user_flags = 0xa5);
        log_error("failure %", 9, loc = explicit, flags = .CONTENT, user_flags = 3, section = *section);
        log_error(*section, "section failure", flags = .WARNING);

        push_context {
            context.log_source_identifier = 42;
            log("inner", section = *section);
        }

        log("outer", flags = .TO_FILE_ONLY | .TOPIC_ONLY, user_flags = 0xffffffff);
        assert(state.last_message == "outer");
        assert(state.last_message.data >= backing.data && state.last_message.data < backing.data + backing.count);
    }

    assert(state.messages.count == 6 && state.infos.count == 6);
    assert(state.messages[0] == "value 17 / text");
    assert(state.messages[1] == "section true");
    assert(state.messages[2] == "failure 9");
    assert(state.messages[3] == "section failure");
    assert(state.messages[4] == "inner" && state.messages[5] == "outer");
    assert(state.infos[0].section == null && state.infos[0].common_flags == .NONE && state.infos[0].user_flags == 0);
    assert(state.infos[0].location.fully_pathed_filename == caller.fully_pathed_filename);
    assert(state.infos[0].location.line_number > 0);
    assert(state.infos[1].location.fully_pathed_filename == explicit.fully_pathed_filename);
    assert(state.infos[1].location.line_number == 123 && state.infos[1].location.character_number == 7);
    assert(state.infos[1].common_flags == .WARNING && state.infos[1].user_flags == 0xa5);
    assert(state.infos[2].common_flags == (.ERROR | .CONTENT) && state.infos[2].user_flags == 3);
    assert(state.infos[3].common_flags == (.ERROR | .WARNING));
    for index: 1..4 assert(state.infos[index].section == *section);
    for index: 0..3 assert(state.infos[index].source_identifier == 0x123456789abcdef0);

    assert(state.infos[4].source_identifier == 42 && state.infos[5].source_identifier == 0x123456789abcdef0);
    assert(state.infos[5].common_flags == (.TO_FILE_ONLY | .TOPIC_ONLY) && state.infos[5].user_flags == 0xffffffff);
    print("ok\n");
}
