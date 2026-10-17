// Reduction: the SDK link name is an import library, not a physical DLL name.
#assert OS == .WINDOWS;
provider :: #system_library,link_always "d3dcompiler";
main :: () {}
