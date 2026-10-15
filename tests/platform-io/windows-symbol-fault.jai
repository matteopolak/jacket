// Additional native trace-quality fixture; preserve the original windows-fault.
#program_export "platform_io_fault_leaf"
fault_leaf :: no_inline (address: *u8) -> u64 #c_call {
    address.* = 23;
    return cast(u64) address.*;
}

#program_export "platform_io_fault_middle"
fault_middle :: no_inline (address: *u8) -> u64 #c_call {
    value := fault_leaf(address);
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

    value := fault_middle(cast(*u8) 1);

    // If the actual fault disappears, the runner rejects this output and exit.
    print("fault unexpectedly returned %\n", value);
}

#scope_file
#import "Basic";

kernel32 :: #system_library "kernel32";
set_error_mode :: (mode: u32) -> u32 #foreign kernel32 "SetErrorMode";
get_error_mode :: () -> u32 #foreign kernel32 "GetErrorMode";
lookup_unwind :: (address: u64, image_base: *u64, history: *void) -> *void #foreign kernel32 "RtlLookupFunctionEntry";
