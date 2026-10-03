package_name :: (name: string) -> string {
    result := name;
    if result.count >= 2 && result[0] == #char "." && result[1] == #char "/" {
        result.data += 2;
        result.count -= 2;
    }

    return result;
}

log_priority :: (flags: Log_Flags) -> s32 {
    if (flags & .ERROR) != 0 return 6;
    if (flags & .WARNING) != 0 return 5;
    if (flags & .VERBOSE_ONLY) != 0 return 3;
    if (flags & .VERY_VERBOSE_ONLY) != 0 return 2;

    return 4;
}

log_tag :: (info: Log_Info) -> string {
    if info.section return info.section.name;
    return "Jai";
}
