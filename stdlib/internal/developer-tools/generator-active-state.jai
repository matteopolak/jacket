// Single shared context slot. init_generator/run_generator owners assign this
// pointer; callers manage nested context normally. No stack or registry.
#scope_module
GP_Sessions :: #import "internal/developer-tools/generator-session";

gp_current_generator :: () -> *Generator {
    return cast(*Generator) GP_Sessions.active_generator();
}

gp_current_options :: () -> *Generate_Bindings_Options {
    return cast(*Generate_Bindings_Options) GP_Sessions.active_options();
}
