#import "Basic";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

errors: int;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

Tracker :: struct {
    underlying: Allocator;
    live: int;
}

tracked :: (mode: Allocator_Mode, bytes: s64, old_bytes: s64, memory: *void, data: *void) -> *void {
    tracker := cast(*Tracker) data;
    result := tracker.underlying.proc(mode, bytes, old_bytes, memory, tracker.underlying.data);

    if (mode == .ALLOCATE || mode == .RESIZE) && !memory && result tracker.live += 1;
    if mode == .FREE && memory tracker.live -= 1;

    return result;
}

return_from_inner :: (generator: *B.Generator) {
    inner := S.open(generator, B.TARGET_OS);

    defer S.close(*inner);

    push_context {
        context.generator = inner.generator;
        context._bindings_generator_session = *inner;
        assert(S.current(B.TARGET_OS) == *inner);
        return;
    }
}

main :: () {
    assert(context._bindings_generator_session == null);
    assert(context._bindings_generator_options == null);
    first, second: B.Generator;
    outer := S.open(*first, B.TARGET_OS);
    inner := S.open(*second, B.TARGET_OS);

    push_context {
        options: B.Generate_Bindings_Options;

        S.activate(*first, *options, *outer);
        assert(S.active_generator() == *first && S.active_options() == *options);
        assert(S.current(B.TARGET_OS) == *outer);
        owned := S.new_owned(*outer, B.Declaration);
        owned.name = S.copy_text(*outer, "owned");
        assert(outer.allocations.count == 2);

        push_context {
            context.generator = inner.generator;
            context._bindings_generator_session = *inner;
            assert(S.current(B.TARGET_OS) == *inner);
        }

        assert(S.current(B.TARGET_OS) == *outer);

        push_context {
            context.logger = capture;
            assert(!S.current(.WINDOWS));
        }

        assert(errors == 1);
    }

    assert(context._bindings_generator_session == null);
    S.close(*inner);
    S.close(*outer);
    assert(!outer.live && !inner.live && outer.allocations.count == 0);

    push_context {
        context.logger = capture;
        assert(!S.current(B.TARGET_OS));
    }

    assert(errors == 2);
    tracker: Tracker;
    tracker.underlying = context.allocator;
    tracked_session: S;

    push_context {
        context.allocator = .{ tracked, *tracker };
        tracked_session = S.open(*first, B.TARGET_OS);
    }

    push_context {
        context.generator = tracked_session.generator;
        context._bindings_generator_session = *tracked_session;
        block := S.new_owned(*tracked_session, B.Block);
        child := S.new_owned(*tracked_session, B.Declaration);

        S.append_owned(*tracked_session, *block.members, child);
        assert(block.members.count == 1 && block.members[0] == child);
        return_from_inner(*second);
        assert(S.current(B.TARGET_OS) == *tracked_session);
    }

    assert(tracker.live > 0);
    S.close(*tracked_session);
    assert(tracker.live == 0);
    caller: Tracker;
    caller.underlying = context.allocator;
    replacement: [..] *B.Declaration;

    push_context {
        context.allocator = .{ tracked, *caller };
        array_resize(*replacement, 1);
    }

    push_context {
        context.allocator = .{ tracked, *tracker };
        tracked_session = S.open(*first, B.TARGET_OS);
    }

    push_context {
        context.generator = *first;
        context._bindings_generator_session = *tracked_session;
        block := S.new_owned(*tracked_session, B.Block);
        child := S.new_owned(*tracked_session, B.Declaration);

        S.append_owned(*tracked_session, *block.members, child);
        for i: 0..20 array_add(*block.members, child);

        // A caller replacement has independent ownership, even while its header
        // is installed in the public model for the remaining callback lifetime.
        block.members = replacement;
    }

    S.close(*tracked_session);
    assert(tracker.live == 0 && caller.live == 1);
    array_free(replacement);
    assert(caller.live == 0);
    print("ok\n");
}
