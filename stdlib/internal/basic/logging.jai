log :: (format_string: string, args: ..Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags: u32 = 0, section: *Log_Section = null) {
    dispatch_log(format_string, args, loc, flags, user_flags, section);
}

log :: (section: *Log_Section, format_string: string, args: ..Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags: u32 = 0) {
    dispatch_log(format_string, args, loc, flags, user_flags, section);
}

log_error :: (format_string: string, args: ..Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags: u32 = 0, section: *Log_Section = null) {
    dispatch_log(format_string, args, loc, flags | .ERROR, user_flags, section);
}

log_error :: (section: *Log_Section, format_string: string, args: ..Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags: u32 = 0) {
    dispatch_log(format_string, args, loc, flags | .ERROR, user_flags, section);
}

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
