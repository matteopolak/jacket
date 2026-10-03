#module_parameters(CHECK_BINDINGS := true);

get_plugin :: () -> *Metaprogram_Plugin {
    plugin.message = Engine.do_error_checking;
    plugin.init = null;
    plugin.before_intercept = null;
    plugin.add_source = null;
    plugin.finish = null;
    plugin.shutdown = null;
    plugin.log_help = null;
    plugin.communication_proc = null;

    return *plugin;
}

#if OS == .WINDOWS {
    DEBUG_BINDINGS :: false;
    C_Porting_Check_Settings :: Engine.C_Porting_Check_Settings;
    verify_elsewhere_binding :: Engine.verify_elsewhere_binding;
    print_binding_check_memory_stats :: Engine.print_binding_check_memory_stats;
    finish_error_checking :: Engine.finish_error_checking;
    check_c_porting :: Engine.check_c_porting;
    should_skip :: Engine.should_skip;
    is_of_fixed_array_type :: Engine.is_of_fixed_array_type;
    do_error_checking :: Engine.do_error_checking;
}

#scope_file
#import "Compiler";

plugin: Metaprogram_Plugin;

#if OS == .WINDOWS Engine :: #import "internal/developer-tools/check-engine"(CHECK_BINDINGS = CHECK_BINDINGS, DEBUG_BINDINGS = DEBUG_BINDINGS);
else Engine :: #import "internal/developer-tools/check-engine"(CHECK_BINDINGS = CHECK_BINDINGS);
