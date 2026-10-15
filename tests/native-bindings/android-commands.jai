// Reviewed android.md transitions, using actual host pthread/pipe allocation.
// This is private algorithm evidence, not Android ABI or device execution.
C :: #import "internal/bindings/android-commands";
P :: #import "POSIX";
N :: #import "internal/bindings/native";
Policy :: #import "internal/bindings/android-policy";
#import "Basic";

State :: struct {
    mutex: P.pthread_mutex_t;
    cond: P.pthread_cond_t;
    msgread: s32;
    window: *void;
    pendingWindow: *void;
    savedState: *void;
    savedStateSize: u64;
    activityState: s32;
    destroyRequested: s32;
    stateSaved: s32;
}

main :: () {
    assert(Policy.package_name("./asset") == "asset");
    assert(Policy.package_name("././asset") == "./asset");
    assert(Policy.package_name(".") == ".");
    assert(Policy.package_name("") == "");
    assert(Policy.log_priority(.ERROR | .WARNING | .VERBOSE_ONLY) == 6);
    assert(Policy.log_priority(.WARNING | .VERBOSE_ONLY) == 5);
    assert(Policy.log_priority(.VERBOSE_ONLY | .VERY_VERBOSE_ONLY) == 3);
    assert(Policy.log_priority(.VERY_VERBOSE_ONLY) == 2);
    assert(Policy.log_priority(.CONTENT) == 4);
    info: Log_Info;

    assert(Policy.log_tag(info) == "Jai");
    section: Log_Section;
    section.name = "native-test";
    info.section = *section;
    assert(Policy.log_tag(info) == "native-test");

    guarded: struct {
        before: u64 = 0x12345678;
        state: State;
        after: u64 = 0x87654321;
    };

    app := *guarded.state;

    assert(P.pthread_mutex_init(*app.mutex, null) == 0);
    assert(P.pthread_cond_init(*app.cond, null) == 0);
    defer assert(P.pthread_mutex_destroy(*app.mutex) == 0);
    defer assert(P.pthread_cond_destroy(*app.cond) == 0);
    descriptors: [2] s32;

    assert(P.pipe(*descriptors) == 0);
    defer P.close(descriptors[0]);
    app.msgread = descriptors[0];
    value: u64 = 51;

    app.pendingWindow = *value;
    C.pre_state(app, 1);
    assert(app.window == *value);
    C.pre_state(app, 2);
    assert(app.window == *value); // Termination stays visible to caller processing.
    C.post_state(app, 2);
    assert(app.window == null);

    for command: s8.[10, 11, 13, 14] {
        C.pre_state(app, command);
        assert(app.activityState == command);
    }

    app.savedState = N.allocate(37);
    app.savedStateSize = 37;
    cmd: s8 = 12;

    assert(P.write(descriptors[1], *cmd, 1) == 1);
    assert(C.read_command(app) == 12);
    assert(app.savedState == null && app.savedStateSize == 0);
    C.post_state(app, 12);
    assert(app.stateSaved == 1);
    app.savedState = N.allocate(19);
    app.savedStateSize = 19;
    C.post_state(app, 11);
    assert(app.savedState == null && app.savedStateSize == 0);
    C.pre_state(app, 15);
    assert(app.destroyRequested == 1);
    assert(P.close(descriptors[1]) == 0);
    assert(C.read_command(app) == -1); // Real EOF, no fabricated read result.
    app.msgread = -1;
    assert(C.read_command(app) == -1); // Real EBADF.
    assert(guarded.before == 0x12345678 && guarded.after == 0x87654321);
}
