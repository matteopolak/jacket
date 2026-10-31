// One file in a zip archive, as its central directory describes it. Most fields are the zip
// format's own; `filename` points into the directory's `memory_block`.
Zip_Entry :: struct {
    filename: string;
    position_in_file: s64;
    version_made_by: u16;
    version_needed_to_extract: u16;
    bit_flag: u16;
    crc: u32;
    compressed_len: u32;
    chunk_len: u32;
    name_len: u16;
    extra_field_len: u16;
    file_comment_len: u16;
    internal_attributes: u16;
    external_attributes: u32;
    chunk_offset: u32;
}

// The list of files in a zip archive, read without extracting anything.
Zip_Directory :: struct {
    name := "";
    entries: [..] Zip_Entry;
    memory_block: string;
    file: File;
}

#scope_module
#import "File";
