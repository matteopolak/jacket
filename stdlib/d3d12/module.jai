#module_parameters(
    // Includes the debug-layer interfaces.
    INCLUDE_DEBUG_BINDINGS := false
)();

#assert OS == .WINDOWS, "d3d12 requires its documented native platform";

#load "declarations.jai";
