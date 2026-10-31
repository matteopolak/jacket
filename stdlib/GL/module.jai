// OpenGL bindings with a loader.
//
// Open a window with `Window_Creation`, call `gl_create_context` to make a context current, then call
// the functions directly, such as `glClear` or `glDrawArrays`; they live in the global `gl`. Constants
// (`GL_COLOR_BUFFER_BIT`) and the `GL*` types are module level. Module parameters: `DUMP_GL_ERRORS` logs GL errors,
// `ENABLE_ALL_EXTENSIONS` loads every extension procedure, `DEFAULT_MSAA` is the sample count for new contexts.
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

#module_parameters(DUMP_GL_ERRORS := false, ENABLE_ALL_EXTENSIONS := false, DEFAULT_MSAA: s32 = 4)();
#assert OS == .WINDOWS || OS == .LINUX || OS == .MACOS, "GL requires its documented native platform";

#load "declarations.jai";
#load "wrappers.jai";
