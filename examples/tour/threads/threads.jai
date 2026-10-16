// The browser/WASI tour uses the private compiler runtime provider.
// Other targets retain the original native Thread consumer.
#if OS == .WASM {
    #load "cooperative.jai";
} else {
    #load "native.jai";
}
