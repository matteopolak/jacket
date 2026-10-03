#module_parameters()(USE_ANSI_CODES_ON_WINDOWS := false, SAVE_RESTORE_ON_WINDOWS := false);
#assert OS != .WINDOWS || USE_ANSI_CODES_ON_WINDOWS, "native Windows console color handling is not implemented";

#load "../internal/output/print-color.jai";
