// Target source regression; this is not a host GLX runtime test.
G :: #import "GL";
X :: #import "X11";

main :: () {
    window: X.Window;
    legacy, legacy_config, legacy_samples := G.glx_create_context(window);
    modern, modern_config, modern_samples := G.glx_create_context(window,
        major_version = 4, minor_version = 6, compatibility = false, debug = true);

    #assert type_of(legacy) == X.GLXContext;
    #assert type_of(legacy_config) == X.GLXFBConfig;
    #assert type_of(legacy_samples) == s32;
    #assert type_of(modern) == X.GLXContext;
    #assert type_of(modern_config) == X.GLXFBConfig;
    #assert type_of(modern_samples) == s32;
}
