CRASH_HANDLER_USE_SEPARATE_THREAD :: false;

Crash_Handler :: #type (trace: [] string);
crash_handler: Crash_Handler;

Assert_Response :: enum {
    Ignore :: 0;
    Debug :: 1;
    Abort :: 2;
}

Exit_Code :: enum s32 {
    Success :: 0;
    Failure :: 1;
    Abort :: 2;
    Assert :: 3;
    Exception :: 4;
}

Minidump_State :: enum {
    None;
    Mini;
    Full;
}

#load "../developer-tools/debug-assert-storage.jai";

interactive_reporting: bool;

remember_ignored_assert :: (loc: Source_Code_Location) {
    push_context {
        context.allocator = context.default_allocator;
        // A retention allocation must not continue after a nonfatal assertion.
        context.assertion_failed = Runtime.runtime_support_assertion_failed;
        if ignored_asserts.allocator.proc != context.allocator.proc || ignored_asserts.allocator.data != null {
            replacement: type_of(ignored_asserts);
            replacement.allocator = context.allocator;

            for entry: ignored_asserts.entries {
                if entry.hash >= Tables.FIRST_VALID_HASH Tables.table_add(*replacement, entry.key, entry.value);
            }

            Tables.table_reset(*ignored_asserts);
            ignored_asserts = replacement;
        }

        stored := loc;
        stored.fully_pathed_filename = Basic.copy_string(loc.fully_pathed_filename);
        Basic.array_add(*owned_ignored_filenames, stored.fully_pathed_filename);
        Tables.table_set(*ignored_asserts, stored, true);
    }
}

#scope_file
Basic :: #import "Basic";
Tables :: #import "Hash_Table";
Runtime :: #import "Runtime_Support";

owned_ignored_filenames: [..] string;
