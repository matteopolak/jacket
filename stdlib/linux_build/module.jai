get_plugin :: Impl.get_plugin;
before_intercept :: Impl.before_intercept;
add_source :: Impl.add_source;
message :: Impl.message;
finish :: Impl.finish;
shutdown :: Impl.shutdown;

#scope_file
WINDOWS_HOST :: #run,host -> bool {
    return OS == .WINDOWS;
};

#assert WINDOWS_HOST;

Impl :: #import "internal/build/linux-link";
