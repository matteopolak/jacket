// C numeric conversion support for the compiler's WASI command runtime.
#assert OS == .WASM "Wasi_Numeric_Runtime is only for wasm targets";

#scope_module
#import "internal/wasi-numeric";
