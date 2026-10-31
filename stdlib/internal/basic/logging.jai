// Formats a message like `print` and hands it to `context.logger`. By default that writes to standard error; swap the
// logger to send messages somewhere else.
//
// ```jai
// log("loaded % files", 3);
// ```
//
// - flags: a `Log_Flags` value, such as `.ERROR` or `.CONTENT`. Passed on to the logger.
// - user_flags: bits for your own logger. Nothing in the standard library reads them.
// - section: the `Log_Section` the message belongs to, or null. Passed on to the logger.
log :: (format_string: string, args: ..Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags: u32 = 0, section: *Log_Section = null) {
    dispatch_log(format_string, args, loc, flags, user_flags, section);
} @PrintLike

log :: (section: *Log_Section, format_string: string, args: ..Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags: u32 = 0) {
    dispatch_log(format_string, args, loc, flags, user_flags, section);
} @PrintLike

// Like `log`, with the `.ERROR` flag added.
//
// ```jai
// log_error("could not open %", "config.txt");
// ```
log_error :: (format_string: string, args: ..Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags: u32 = 0, section: *Log_Section = null) {
    dispatch_log(format_string, args, loc, flags | .ERROR, user_flags, section);
} @PrintLike

log_error :: (section: *Log_Section, format_string: string, args: ..Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags: u32 = 0) {
    dispatch_log(format_string, args, loc, flags | .ERROR, user_flags, section);
} @PrintLike

#scope_module
dispatch_log :: (format_string: string, args: [] Any, loc: Source_Code_Location, flags: Log_Flags, user_flags: u32, section: *Log_Section) {
    info := Log_Info.{
        source_identifier = context.log_source_identifier,
        location = loc,
        common_flags = flags,
        user_flags = user_flags,
        section = section,
    };

    message := tprint(format_string, ..args);

    if message.count != 0 context.logger(message, context.logger_data, info);
}
