#scope_module
GM_Sessions :: #import "internal/developer-tools/generator-session";

gm_active_options :: () -> *Generate_Bindings_Options {
    return cast(*Generate_Bindings_Options) GM_Sessions.active_options();
}

gm_current_session :: () -> *GM_Sessions.Session {
    return GM_Sessions.current(TARGET_OS);
}

// Why `context.generator` cannot be used here, or "" when it can: none is active, or the session that owns the running
// generator has ended or belongs to another instance of this module (one compiled for another `TARGET_OS`).
gm_generator_problem :: () -> string {
    if !context.generator return "no active context.generator";

    session := context._bindings_generator_session;

    if session && (!session.live || session.generator != context.generator || session.target_os != TARGET_OS) {
        return "generator does not match the active ownership ledger";
    }

    return "";
}

gm_active_generator :: () -> *Generator {
    problem := gm_generator_problem();

    if problem.count {
        log_error("Bindings_Generator: %\n", problem);

        return null;
    }

    return cast(*Generator) context.generator;
}
