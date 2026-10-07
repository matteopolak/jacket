// init_core_graphics_runtime_constants fills every window-level global from CGWindowLevelForKey.
#import "Basic";

#if OS == .MACOS {
    #import "Objective_C";
    #import "Objective_C/AppKit";
}

main :: () {
    #if OS == .MACOS {
        init_core_graphics_runtime_constants();
        assert(kCGNormalWindowLevel == CGWindowLevelForKey(.kCGNormalWindowLevelKey));
        assert(kCGBackstopMenuLevel == CGWindowLevelForKey(.kCGBackstopMenuLevelKey));
        assert(kCGAssistiveTechHighWindowLevel == CGWindowLevelForKey(.kCGAssistiveTechHighWindowLevelKey));
        // Levels documented as fixed: normal is 0, floating sits above it, the minimum below all.
        assert(kCGNormalWindowLevel == 0);
        assert(kCGFloatingWindowLevel > kCGNormalWindowLevel);
        assert(kCGMinimumWindowLevel < kCGDesktopWindowLevel);
        assert(kCGMaximumWindowLevel > kCGScreenSaverWindowLevel);
    }
}
