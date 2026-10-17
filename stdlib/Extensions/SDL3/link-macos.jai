// SDL3 static dependencies. Preserve SDL's optional weak framework linkage.
#library,system,link_always "Cocoa";
#library,system,link_always "Carbon";
#library,system,link_always "IOKit";
#library,system,link_always "CoreFoundation";
#library,system,link_always "CoreAudio";
#library,system,link_always "AudioToolbox";
#library,system,link_always "AVFoundation";
#library,system,link_always "CoreMedia";
#library,system,link_always "CoreVideo";
#library,system,link_always "Foundation";
#library,system,link_always "GameController";
#library,system,link_always "Metal";
#library,system,link_always "QuartzCore";
#library,system,link_always "ForceFeedback";
#library,system,link_always "objc";

#scope_file
Workspaces :: #import "internal/compiler-api";

// Append without replacing caller linker options.
#run {
    w := Workspaces.__jaic_current_workspace();
    Workspaces.append_linker_argument(w, "-Wl,-weak_framework,CoreHaptics");
    Workspaces.append_linker_argument(w, "-Wl,-weak_framework,UniformTypeIdentifiers");
}
