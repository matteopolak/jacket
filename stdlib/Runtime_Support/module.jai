#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool);

OS_IS_UNIX :: OS == .MACOS || OS == .LINUX || OS == .PS5 || OS == .IOS || OS == .ANDROID;

Context_Base :: struct {
    context_info: *Type_Info_Struct;
    thread_index: u32;
    allocator := default_allocator;
    logger := runtime_support_default_logger;
    logger_data: *void;
    log_source_identifier: u64;
    log_level: Log_Level;
    temporary_storage: *Temporary_Storage;
    stack_trace: *Stack_Trace_Node;
    assertion_failed := runtime_support_assertion_failed;
    handling_assertion_failure := false;
    default_allocator :: Allocator.{ runtime_support_default_allocator_proc, null };
}

Temporary_Storage :: struct {
    data: *u8;
    size: s64;
    total_bytes_occupied: s64;
    high_water_mark: s64;
    last_set_mark_location: Source_Code_Location;
    implementation_data: [256] u8;
}

synch_initted: s32 = 0;
__instrumentation_first :: () {}
__instrumentation_second :: () {}
__array_bounds_check_fail :: Impl.__array_bounds_check_fail;
__cast_bounds_check_fail :: Impl.__cast_bounds_check_fail;
__null_pointer_check_fail :: Impl.__null_pointer_check_fail;
__arithmetic_overflow :: Impl.__arithmetic_overflow;
one_time_init :: Impl.one_time_init;

#if DEFINE_INITIALIZATION {
    __jai_runtime_init :: Impl.__jai_runtime_init;
    __jai_runtime_fini :: Impl.__jai_runtime_fini;
}

__element_duplicate :: Impl.__element_duplicate;
c_style_strlen :: Impl.c_style_strlen;
to_string :: Impl.to_string;
write_string_unsynchronized :: Impl.write_string_unsynchronized;
write_string :: Impl.write_string;
write_strings :: Impl.write_strings;
#if OS == .ANDROID android_flush_write_string_buffer :: Impl.android_flush_write_string_buffer;
write_nonnegative_number :: Impl.write_nonnegative_number;
write_number :: Impl.write_number;
runtime_support_assertion_failed :: Impl.runtime_support_assertion_failed;
runtime_support_default_logger :: Impl.runtime_support_default_logger;
debug_break :: Impl.debug_break;
compile_time_debug_break :: Impl.compile_time_debug_break;
runtime_support_default_allocator_proc :: Default_Allocator.allocator_proc;

#scope_file
__jaic_check_failed :: Impl.report_native_check;
__jaic_interpreter_assertion :: Impl.runtime_support_assertion_failed;

Impl :: #import "internal/runtime";
Default_Allocator :: #import "Default_Allocator";

program_entry :: () -> s32 #entry_point;

#if OS == .WINDOWS Windows_Arguments :: #import "internal/runtime/windows-arguments";

#if DEFINE_SYSTEM_ENTRY_POINT && ENABLE_BACKTRACE_ON_CRASH {
    #if OS == .WINDOWS && CPU == .ARM64 Crash :: #import "internal/runtime/windows-arm64-crash";
    else Crash :: #import "Runtime_Support_Crash_Handler";
    #if OS == .WINDOWS && CPU == .X64 Crash_Cleanup :: #import "internal/runtime/windows-crash-cleanup";
}

#if DEFINE_SYSTEM_ENTRY_POINT {
    #program_export "main"
    runtime_entry :: (argc: s32, argv: **u8) -> s32 #c_call #no_debug {
        #if OS == .WINDOWS {
            arguments, acquired := Windows_Arguments.acquire();
            if !acquired {
                write_string("Windows command line conversion failed\n", true);
                return 1;
            }

            defer Windows_Arguments.release(*arguments);
            ctx := Impl.__jai_runtime_init(arguments.count, arguments.values);
            if !ctx {
                write_string("runtime initialization failed\n", true);
                return 1;
            }

            defer Impl.__jai_runtime_fini(ctx);

            push_context ctx.* {
                #if ENABLE_BACKTRACE_ON_CRASH Crash.init();
                #if ENABLE_BACKTRACE_ON_CRASH && CPU == .ARM64 defer Crash.fini();
                #if ENABLE_BACKTRACE_ON_CRASH && CPU == .X64 defer Crash_Cleanup.fini();
                __instrumentation_first();
                __instrumentation_second();

                return program_entry();
            }
        } else {
            ctx := Impl.__jai_runtime_init(argc, argv);
            if !ctx {
                write_string("runtime initialization failed\n", true);
                debug_break();
                return 1;
            }

            defer Impl.__jai_runtime_fini(ctx);

            push_context ctx.* {
                #if ENABLE_BACKTRACE_ON_CRASH Crash.init();
                __instrumentation_first();
                __instrumentation_second();
                return program_entry();
            }
        }
    }
}
