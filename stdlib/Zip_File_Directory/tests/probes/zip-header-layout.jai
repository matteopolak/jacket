// Reviewed public field directives; sizes/alignments need oracle clarification.
#import "Basic";

Local_Header :: struct {
    signature: u32;
    version_needed_to_extract: u16;
    general_purpose_bit_flag: u16;
    compression_method: u16;
    last_mod_file_time: u16;
    last_mod_file_date: u16;
    crc_32: u32 #align 1;
    compressed_size: u32 #align 1;
    uncompressed_size: u32 #align 1;
    filename_length: u16 #align 1;
    extra_field_length: u16 #align 1;
}

End_Record :: struct {
    signature: u32;
    disk_number_this: u16;
    disk_number_start: u16;
    total_entries_this_disk: u16;
    total_entries: u16;
    central_directory_size: u32;
    central_directory_offset: u32;
    file_comment_length: u16;
}

main :: () {
    print("local=% end=%\n", size_of(Local_Header), size_of(End_Record));
}
