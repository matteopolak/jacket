get_plugin :: () -> *C.Metaprogram_Plugin {
    return *instance;
}

#scope_file
C :: #import "Compiler";

instance: C.Metaprogram_Plugin;
