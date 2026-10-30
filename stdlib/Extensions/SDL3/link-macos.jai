// SDL3 static dependencies. Preserve SDL's optional weak framework linkage.
#library,system,link_always "Cocoa"; @native(external)
#library,system,link_always "Carbon"; @native(external)
#library,system,link_always "IOKit"; @native(external)
#library,system,link_always "CoreFoundation"; @native(external)
#library,system,link_always "CoreAudio"; @native(external)
#library,system,link_always "AudioToolbox"; @native(external)
#library,system,link_always "AVFoundation"; @native(external)
#library,system,link_always "CoreMedia"; @native(external)
#library,system,link_always "CoreVideo"; @native(external)
#library,system,link_always "Foundation"; @native(external)
#library,system,link_always "GameController"; @native(external)
#library,system,link_always "Metal"; @native(external)
#library,system,link_always "QuartzCore"; @native(external)
#library,system,link_always "ForceFeedback"; @native(external)
#library,system,link_always "objc"; @native(external)

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
