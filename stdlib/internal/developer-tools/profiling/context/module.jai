#add_context _independent_iprof_runtime_active: bool;
#add_context _independent_iprof_program_running: bool;

is_active :: () -> bool #expand {
    return context._independent_iprof_runtime_active;
}

set_active :: (active: bool) -> bool #expand {
    prior := context._independent_iprof_runtime_active;
    context._independent_iprof_runtime_active = active;
    return prior;
}
