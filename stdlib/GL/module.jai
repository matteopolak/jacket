#module_parameters(DUMP_GL_ERRORS := false, ENABLE_ALL_EXTENSIONS := false, DEFAULT_MSAA: s32 = 4)();
#assert OS == .WINDOWS || OS == .LINUX || OS == .MACOS, "GL requires its documented native platform";

#load "declarations.jai";
#load "wrappers.jai";
