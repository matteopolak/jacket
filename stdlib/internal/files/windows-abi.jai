// Narrow native ABI supplements from Microsoft fileapi.h/winbase.h documentation.
Attribute_Data :: struct {
    attributes: u32;
    created: W.FILETIME;
    accessed: W.FILETIME;
    written: W.FILETIME;
    size_high: u32;
    size_low: u32;
}

File_Identity :: struct {
    volume: u64;
    id: [16] u8;
}

#assert size_of(Attribute_Data) == 36;
#assert size_of(File_Identity) == 24;

get_attributes :: (name: *u16, level: s32, data: *Attribute_Data) -> s32 #foreign kernel32 "GetFileAttributesExW";
replace_rename :: (source: *u16, dest: *u16, flags: u32) -> s32 #foreign kernel32 "MoveFileExW";

#scope_file
W :: #import "Windows";

kernel32 :: #system_library "kernel32";
