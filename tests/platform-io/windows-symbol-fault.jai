// Jai callers must remain named and unwindable across an actual SDK fault.
#program_export "platform_io_fault_leaf"
fault_leaf :: no_inline () -> u64 #c_call {
    value := native_fault();
    return cast(u64) value + 1;
}

#program_export "platform_io_fault_middle"
fault_middle :: no_inline () -> u64 #c_call {
    value := fault_leaf();
    return value + 1;
}

main :: () {
    set_error_mode(get_error_mode() | 2);

    #if CPU == .X64 {
        addresses: [2] u64 = .[cast(u64) cast(*void) fault_leaf, cast(u64) cast(*void) fault_middle];
        for address: addresses {
            image_base: u64;
            assert(lookup_unwind(address, *image_base, null) != null,
                "native frame function is missing Windows unwind metadata");
        }
    }

    value := fault_middle();

    // If the actual fault disappears, the runner rejects this output and exit.
    print("fault unexpectedly returned %\n", value);
}

#scope_file
#import "Basic";

sdk :: #library "../../target/platform-io-tests/platform_io_fault_sdk";
native_fault :: () -> u32 #foreign sdk "platform_io_native_fault";

kernel32 :: #system_library "kernel32";
set_error_mode :: (mode: u32) -> u32 #foreign kernel32 "SetErrorMode";
get_error_mode :: () -> u32 #foreign kernel32 "GetErrorMode";
lookup_unwind :: (address: u64, image_base: *u64, history: *void) -> *void #foreign kernel32 "RtlLookupFunctionEntry";
