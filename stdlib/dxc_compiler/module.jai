#module_parameters(
    // Nintendo platform support; no declaration depends on it yet.
    NINTENDO_SUPPORT := false
)();

#assert OS == .WINDOWS, "dxc_compiler requires its documented native platform";

#load "declarations.jai";
#load "wrappers.jai";
