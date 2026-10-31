// OpenGL bindings with a loader.
//
// Open a window with `Window_Creation`, call `gl_create_context` to make a context current, then call
// the functions directly, such as `glClear` or `glDrawArrays`; they live in the global `gl`. Constants
// (`GL_COLOR_BUFFER_BIT`) and the `GL*` types are module level.
//
// ```jai,no_run
// #import "Window_Creation";
// #import "GL";
//
// window := create_window(800, 600, "gl");
// gl_create_context(window, 3, 3);
// glClearColor(0.1, 0.1, 0.2, 1);
// glClear(GL_COLOR_BUFFER_BIT);
// ```

#module_parameters(
    // Logs GL errors.
    DUMP_GL_ERRORS := false,
    // Loads every extension procedure.
    ENABLE_ALL_EXTENSIONS := false,
    // The sample count of new contexts.
    DEFAULT_MSAA: s32 = 4
)();

#assert OS == .WINDOWS || OS == .LINUX || OS == .MACOS, "GL requires its documented native platform";

#load "declarations.jai";
#load "wrappers.jai";
