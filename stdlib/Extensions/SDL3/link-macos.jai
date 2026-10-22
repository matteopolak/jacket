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
C :: #import "Compiler";
Basic :: #import "Basic";

// Preserve all settings, including output controls set during compilation.
#run {
    options := C.get_build_options();
    arguments: [..] string;

    defer Basic.array_free(arguments);

    for arg: options.additional_linker_arguments Basic.array_add(*arguments, arg);

    Basic.array_add(*arguments, "-Wl,-weak_framework,CoreHaptics");
    Basic.array_add(*arguments, "-Wl,-weak_framework,UniformTypeIdentifiers");
    options.additional_linker_arguments = arguments;
    C.set_build_options(options);
}
