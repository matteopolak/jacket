// Independently map the local logging protocol to the NDK text logger.
android_logger :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .TO_FILE_ONLY) != 0 return;
    priority := Log_Priority.INFO;
    if (info.common_flags & .ERROR) != 0 priority = .ERROR;
    else if (info.common_flags & .WARNING) != 0 priority = .WARN;
    else if (info.common_flags & .VERY_VERBOSE_ONLY) != 0 priority = .VERBOSE;
    else if (info.common_flags & .VERBOSE_ONLY) != 0 priority = .DEBUG;
    text := Basic.to_c_string(message);
    if text == null return;
    defer Basic.free(text);
    tag := Basic.to_c_string(LOG_TAG);
    if tag == null return;
    defer Basic.free(tag);
    __android_log_write(priority, tag, text);
}

#scope_file
Basic :: #import "Basic";
