// A point in a source file, as `#location` and `#caller_location` give it. Lines and characters
// count from 1.
Source_Code_Location :: struct {
    fully_pathed_filename: string;
    line_number: s64;
    character_number: s64;
}

// A span of a source file, from the start line and character to the end ones.
Source_Code_Range :: struct {
    fully_pathed_filename: string;
    line_number_start: s64;
    line_number_end: s64;
    character_number_start: s64;
    character_number_end: s64;
}

// A procedure that can appear in a stack trace. Each one is a global the compiler emits, so a
// pointer to it stays valid for the whole run.
Stack_Trace_Procedure_Info :: struct {
    name: string;
    location: Source_Code_Location; // Where the procedure is declared.

    procedure_address: *void;

}

// One call on the stack, newest first: `context.stack_trace` is the innermost call and `next` is
// its caller. Null in builds without stack traces.
Stack_Trace_Node :: struct {
    next: *Stack_Trace_Node;
    info: *Stack_Trace_Procedure_Info;
    hash: u64; // Mixes the caller's hash, this procedure and the calling line: equal call paths hash the same.
    call_depth: u32; // 1 for the outermost traced call.
    line_number: u32; // The line this procedure is on: where it made the call above it on the stack.
}

// How much a logger should print. `context.log_level` holds the current one.
Log_Level :: enum u8 {
    NORMAL :: 0;
    VERBOSE :: 1;
    VERY_VERBOSE :: 2;
}

// Flags on a log message, in [Log_Info.common_flags]. They are hints: the logger decides what
// each one does. The default logger only looks at `.ERROR`, which sends the message to standard
// error.
Log_Flags :: enum_flags u32 {
    NONE :: 0x0;
    ERROR :: 0x1;
    WARNING :: 0x2;
    CONTENT :: 0x4; // Output the program is for, as opposed to diagnostics.
    TO_FILE_ONLY :: 0x8; // Write to a log file but not the console.
    VERBOSE_ONLY :: 0x10; // Show only at `Log_Level.VERBOSE` or above.
    VERY_VERBOSE_ONLY :: 0x20; // Show only at `Log_Level.VERY_VERBOSE`.
    TOPIC_ONLY :: 0x40; // Show only when the message's [Log_Section] is enabled.
}

// A named group of log messages, so a logger can filter or label them. Pass a pointer to one as
// `section` in `log`.
Log_Section :: struct {
    name: string;
}

// What a [Logger] gets with each message.
Log_Info :: struct {
    source_identifier: u64; // `context.log_source_identifier` when the message was logged.
    location: Source_Code_Location; // Where `log` was called.
    common_flags: Log_Flags;
    user_flags: u32; // Bits the program defines; passed through unchanged.
    section: *Log_Section; // Null when the message has none.
}

// The procedure in `context.logger`. `data` is `context.logger_data`. `message` is only valid
// during the call.
Logger :: #type (message: string, data: *void, info: Log_Info);
