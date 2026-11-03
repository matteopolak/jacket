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
//
// gl_create_context(window, 3, 3);
// glClearColor(0.1, 0.1, 0.2, 1);
// glClear(GL_COLOR_BUFFER_BIT);
// ```

// test: skip_on(.WASM, reason = "GL has no wasm target")
