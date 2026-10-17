// Procedure-table policy controls, not simulated GPU success or SDK evidence.
G :: #import "GL";

libc :: #library,system "c";
exit :: (status: s32) #foreign libc;

check :: (value: bool) {
    if !value exit(91);
}

callback: G.GLDEBUGPROC;
callback_data: *void;
core_calls: s32;
arb_calls: s32;
khr_calls: s32;
first_logs: s32;
second_logs: s32;
token: s32 = 713;

core_install :: (procedure: G.GLDEBUGPROC, data: *void) #c_call {
    core_calls += 1;
    callback = procedure;
    callback_data = data;
}

arb_install :: (procedure: G.GLDEBUGPROCARB, data: *void) #c_call {
    arb_calls += 1;
    callback = cast(G.GLDEBUGPROC) procedure;
    callback_data = data;
}

khr_install :: (procedure: G.GLDEBUGPROCKHR, data: *void) #c_call {
    khr_calls += 1;
}

control :: (source: G.GLenum, type: G.GLenum, severity: G.GLenum,
            count: G.GLsizei, ids: *G.GLuint, enabled: G.GLboolean) #c_call {}

observe_first :: (message: string, data: *void, info: Log_Info) {
    check(data == cast(*void) *token && message == "driver payload");
    first_logs += 1;
}

observe_second :: (message: string, data: *void, info: Log_Info) {
    check(data == cast(*void) *second_logs && message == "driver payload");
    second_logs += 1;
}

emit :: (source: G.GLenum, type: G.GLenum) {
    message := "driver payload";
    callback(source, type, 37, G.GL_DEBUG_SEVERITY_HIGH,
             cast(G.GLsizei) message.count, message.data, callback_data);
}

main :: () {
    previous := G.gl;
    previous_logger := context.logger;
    previous_data := context.logger_data;

    defer {
        G.gl = previous;
        context.logger = previous_logger;
        context.logger_data = previous_data;
    }

    G.gl = .{};
    G.gl.GL_VERSION_4_1 = true;
    // No supported capability, no procedure pointer needed, no registration.
    G.gl_enable_debug_output();
    G.gl_enable_debug_output(true);
    check(core_calls == 0 && arb_calls == 0 && khr_calls == 0);
    G.gl.GL_KHR_debug = true;
    G.gl.glDebugMessageCallbackKHR = khr_install;
    G.gl.glDebugMessageControlKHR = control;
    G.gl_enable_debug_output();
    check(khr_calls == 0);
    G.gl.GL_VERSION_4_3 = true;
    G.gl.GL_ARB_debug_output = true;
    G.gl.glDebugMessageCallback = core_install;
    G.gl.glDebugMessageControl = control;
    G.gl.glDebugMessageCallbackARB = arb_install;
    G.gl.glDebugMessageControlARB = control;
    context.logger = observe_first;
    context.logger_data = *token;
    G.gl_enable_debug_output();
    check(core_calls == 1 && arb_calls == 0 && callback != null);
    check(callback_data != null);
    old_callback := callback;
    old_data := callback_data;

    emit(G.GL_DEBUG_SOURCE_APPLICATION, G.GL_DEBUG_TYPE_ERROR);
    emit(G.GL_DEBUG_SOURCE_API, G.GL_DEBUG_TYPE_OTHER);
    emit(G.GL_DEBUG_SOURCE_API, G.GL_DEBUG_TYPE_UNDEFINED_BEHAVIOR);
    check(first_logs == 0);
    emit(G.GL_DEBUG_SOURCE_API, G.GL_DEBUG_TYPE_ERROR);
    emit(G.GL_DEBUG_SOURCE_SHADER_COMPILER, G.GL_DEBUG_TYPE_PERFORMANCE);
    check(first_logs == 2);
    G.gl.GL_VERSION_4_3 = false;
    context.logger = observe_second;
    context.logger_data = *second_logs;
    // glEnable is a constant SDK procedure. The true/synchronous path needs a
    // genuinely supported native context and is source-checked separately here.
    G.gl_enable_debug_output(false);
    check(core_calls == 1 && arb_calls == 1 && khr_calls == 0);
    check(old_data == callback_data && old_callback == callback);
    // A retained earlier callback observes the later logger/data, not TLS logger.
    context.logger = observe_first;
    context.logger_data = *token;
    message := "driver payload";

    old_callback(G.GL_DEBUG_SOURCE_API, G.GL_DEBUG_TYPE_PERFORMANCE, 41,
                 G.GL_DEBUG_SEVERITY_LOW, cast(G.GLsizei) message.count, message.data, old_data);
    check(first_logs == 2 && second_logs == 1);
    // Updating the flag back to false allows an error callback without a break.
    context.logger = observe_second;
    context.logger_data = *second_logs;
    G.gl_enable_debug_output(false);
    check(arb_calls == 2);
    emit(G.GL_DEBUG_SOURCE_API, G.GL_DEBUG_TYPE_ERROR);
    check(first_logs == 2 && second_logs == 2);
}

synchronous_source_control :: () {
    G.gl_enable_debug_output(true);
}
