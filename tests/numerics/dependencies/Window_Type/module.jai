// Test-only borrowed handle fixture. WASM is an independent testing extension.
#if OS == .LINUX {
    Window_Type :: u64;
    INVALID_WINDOW: Window_Type : 0;
} else {
    Window_Type :: *void;
    INVALID_WINDOW: Window_Type : null;
}
