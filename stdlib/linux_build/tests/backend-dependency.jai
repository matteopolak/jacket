// Public callback's real required backend. Expected current compiler dependency.
#load "../../internal/build/linux-link.jai";

run_backend_dependency :: () {
    w := C.compiler_create_workspace("required Linux plugin backend");

    assert(w >= 0);
    p := get_plugin();

    assert(p != null);
    p.workspace = w;
    defer shutdown(p);
    flags: C.Intercept_Flags;

    before_intercept(p, *flags);
    options := C.get_build_options(w);

    assert(options.os_target == .LINUX && options.backend == .X64 && options.use_custom_link_command);
    assert(flags == 0);
    C.set_build_options_dc(.{ do_output = false, write_added_strings = false });
}
