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
// Source-built jaic workspace primitives; append without replacing caller linker options.
__jaic_current_workspace :: () -> s64 #compiler;
__jaic_workspace_set_option :: (w: s64, key: string, value: string) #compiler;
#run {
    w := __jaic_current_workspace();
    __jaic_workspace_set_option(w, "additional_linker_argument", "-Wl,-weak_framework,CoreHaptics");
    __jaic_workspace_set_option(w, "additional_linker_argument", "-Wl,-weak_framework,UniformTypeIdentifiers");
}
