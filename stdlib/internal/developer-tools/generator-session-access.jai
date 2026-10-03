#scope_module
GM_Sessions :: #import "internal/developer-tools/generator-session";

gm_active_options :: () -> *Generate_Bindings_Options {
    return cast(*Generate_Bindings_Options) GM_Sessions.active_options();
}

gm_current_session :: () -> *GM_Sessions.Session {
    return GM_Sessions.current(TARGET_OS);
}

gm_active_generator :: () -> *Generator {
    if !context.generator {
        log_error("Bindings_Generator: no active context.generator\n");
        return null;
    }

    session := context._bindings_generator_session;
    if session && (!session.live || session.generator != context.generator || session.target_os != TARGET_OS) {
        log_error("Bindings_Generator: generator does not match the active ownership ledger\n");
        return null;
    }

    return cast(*Generator) context.generator;
}
