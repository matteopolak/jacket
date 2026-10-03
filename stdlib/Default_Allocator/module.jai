#module_parameters(ENABLE_ASSERTS := true, ENABLE_VALIDATE_ARGS := true);

#if (CPU == .X64 && OS != .IOS) || OS == .NN_SWITCH2 {
    #load "../internal/allocation/default-records.jai";
    #load "../internal/allocation/default-heaps.jai";
} else {
    #load "../internal/allocation/default-c.jai";
}
